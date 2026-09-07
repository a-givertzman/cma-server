use chrono::{DateTime, Utc};
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::services::entity::{
    Cot, Point, PointConf, PointConfAddress, PointType, PointHlr, Status,
};
use crate::{domain::filter::filter::{Filter, FilterEmpty}, err, services::slmp_client::slmp::ParsePoint};
///
/// Used for parsing configured point from slice of bytes read from device
#[derive(Debug)]
pub struct SlmpParseInt {
    id: String,
    typ: PointType,
    txid: usize,
    name: String,
    value: Box<dyn Filter<Item = i64> + Send>,
    status: Box<dyn Filter<Item = Status> + Send>,
    offset: u32,
    // history: PointConfHistory,
    // alarm: Option<u8>,
    // comment: Option<String>,
}
//
//
impl SlmpParseInt {
    ///
    /// Size in the bytes in the Device address area
    const SIZE: usize = 2;
    ///
    /// ### Creates `SlmpParseInt` new instance
    /// - `parent` - Идентификатор родительского сервиса (для отладки)
    /// - `txid` - Идентификатор сервиса-отправителя сигнала (например родительский ProfinetClient)
    /// - `name` - Ниаменование сигнала
    /// - `conf` - Конфигурация сигнала
    /// - `filter` - Фильтр значения сигнала
    #[named]
    pub fn new(
        parent: impl Into<String>,
        tx_id: usize,
        name: String,
        conf: &PointConf,
        filter: Box<dyn Filter<Item = i64> + Send>,
    ) -> Result<SlmpParseInt, Error> {
        let dbg = Dbg::new(parent, crate::domain::me::<Self>());
        let addr = conf.address.clone().ok_or_else(|| err!(dbg, "Address is empty in '{name}'"))?;
        let offset = addr.offset.ok_or_else(|| err!(dbg, "Address offset is empty in '{name}'"))?;
        Ok(SlmpParseInt {
            id: format!("SlmpParseInt({})", name),
            typ: conf.type_.clone(),
            txid: tx_id,
            name,
            value: filter,
            status: Box::new(FilterEmpty::<Status>::new(Some(Status::Invalid))),
            offset,
            // history: config.history.clone(),
            // alarm: config.alarm,
            // comment: config.comment.clone(),
        })
    }
    //
    //
    fn convert(
        &self,
        bytes: &[u8],
        start: usize,
    ) -> Result<i16, Error> {
        let value = bytes.get(start..(start + Self::SIZE))
            .and_then(|bytes| bytes.try_into().ok())
            .map(i16::from_le_bytes)
            .ok_or_else(|| Error::new(&self.name, "convert").err("Wrong bytes length"))?;
        Ok(value)
    }
    ///
    /// Логика фильтра входных евентов
    /// 
    /// - Изменилось value или изменился status - возвращаем новый эвент
    /// 
    /// - Если value не изменилось или его нет (значит status пришел)
    ///     - Если status изменился
    ///         - Если есть сохраненное последнее value - возвращаем новый эвент
    ///         - Если есть нет сохраненного value - эвента нет (изменение статуса игнорируется)
    ///     - Если status прежний - эвента нет
    /// 
    /// Все варианты:
    /// ```
    /// | Input value | Value changed | Status changed | Last value exists | Output value | Output status | Result |
    /// |-------------|-------------- |--------------- |-------------------|--------------|---------------|--------|
    /// | None        | No            | No             | -                 | -            | -             | None   |
    /// | None        | No            | Yes            | No                | -            | -             | None   |
    /// | None        | No            | Yes            | Yes               | last         | new           | Point  |
    /// | Some(v)     | No            | No             | Yes               | -            | -             | None   |
    /// | Some(v)     | No            | Yes            | Yes               | last         | new           | Point  |
    /// | Some(v)     | Yes           | No             | Yes               | v            | last status   | Point  |
    /// | Some(v)     | Yes           | Yes            | Yes               | v            | new           | Point  |
    /// ```
    fn to_point(&mut self, value: Option<i64>, status: Status, ts: DateTime<Utc>) -> Option<Point> {
        let value_changed = value.and_then(|v| self.value.add(v));
        let status_changed = self.status.add(status);
        // log::trace!("{}.to_point | value_changed: {:?}  |  status_changed {:?}", self.dbg, value_changed, status_changed);
        let (value, status) = match status_changed {
            Some(status_changed) => (
                value_changed.or_else(|| self.value.last())?,
                status_changed,
            ),
            None => (
                value_changed?,
                self.status.last().unwrap_or(Status::Ok)
            )
        };
        Some(Point::Int(PointHlr::new(
            self.txid,
            &self.name,
            value,
            status,
            Cot::Inf,
            ts,
        )))
    }
    //
    //
    fn add_raw(&mut self, bytes: &[u8], ts: DateTime<Utc>) -> Option<Point> {
        let result = self.convert(bytes, self.offset as usize);
        match result {
            Ok(value) => self.to_point(Some(value as i64), Status::Ok, ts),
            Err(e) => {
                log::warn!("{}.add_raw | convertion error: {:?}", self.name, e);
                self.to_point(None, Status::Invalid, ts)
            }
        }
    }
}
//
//
impl ParsePoint for SlmpParseInt {
    //
    fn next(&mut self, bytes: &[u8], ts: DateTime<Utc>) -> Option<Point> {
        self.add_raw(bytes, ts)
    }
    //
    fn next_status(&mut self, status: Status, ts: DateTime<Utc>) -> Option<Point> {
        self.to_point(None, status, ts)
    }
    //
    fn address(&self) -> PointConfAddress {
        PointConfAddress { offset: Some(self.offset), bit: None }
    }
    //
    fn size(&self) -> usize {
        Self::SIZE
    }
    //
    fn to_bytes(&self, point: &Point) -> Result<Vec<u8>, Error> {
        match point.try_as_int() {
            Ok(point) => {
                log::debug!("{}.write | converting '{}' into i16...", self.id, point.value);
                match i16::try_from(point.value) {
                    Ok(value) => {
                        Ok(value.to_le_bytes().to_vec())
                    }
                    Err(err) => {
                        let err = Error::new(&self.name, "to_bytes").pass_with(format!("Can't convert i16 '{}': {:?} into bytes", point.name, point.value), err.to_string());
                        log::warn!("{}", err);
                        Err(err)
                    }
                }
            }
            Err(err) => {
                let err = Error::new(&self.name, "to_bytes").pass_with(format!("Can't convert i16 '{}': {:?} into bytes", point.name(), point.value()), err.to_string());
                log::warn!("{}", err);
                Err(err)
            }
        }
    }
}
///
/// Tests
#[cfg(test)]
mod slmp_parse_int_test {
    use std::{sync::Once, time::{Duration, Instant}};
    use super::*;
    use chrono::Utc;
    use debugging::session::{DebugSession, LogLevel};
    use sal_core::dbg::Dbg;
    use sal_sync::services::entity::{Name, PointConf, PointConfAddress, PointType, Status};
    use testing::stuff::max_test_duration::TestDuration;
        
    ///
    ///
    static INIT: Once = Once::new();
    ///
    /// once called initialisation
    fn init_once() {
        INIT.call_once(|| {
            // implement your initialisation code to be called only once for current test file
        })
    }
    ///
    /// returns:
    ///  - ...
    fn init_each() -> () {}
    ///
    /// Testing [SlmpParseInt].to_point
    ///
    #[test]
    fn to_point() {
        DebugSession::new().filter(LogLevel::Debug).init().unwrap();
        init_once();
        init_each();
        log::debug!("");
        let dbg = Dbg::own("SlmpParseInt-to_point");
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
        test_duration.run().unwrap();
        // status helpers
        let ok = Status::Ok;
        let invalid = Status::Invalid;
        // ----------------------------------------------------------------
        // Таблица тестов (повторяет truth-table метода)
        // ----------------------------------------------------------------
        let test_data = [
            // ------------------------------------------------------------
            // нет value, статус не изменился
            // ------------------------------------------------------------
            (01, None, ok, None),
            // ------------------------------------------------------------
            // status change без value и без last value -> None
            // ------------------------------------------------------------
            (02, None, invalid, None),
            // ------------------------------------------------------------
            // первое value
            // ------------------------------------------------------------
            (03, Some(0), ok, Some((0, ok))),
            // ------------------------------------------------------------
            // value unchanged -> None
            // ------------------------------------------------------------
            (04, Some(0), ok, None),
            // ------------------------------------------------------------
            // status changed -> emit last value
            // ------------------------------------------------------------
            (05, None, invalid, Some((0, invalid))),
            // ------------------------------------------------------------
            // value changed
            // ------------------------------------------------------------
            (06, Some(1), invalid, Some((1, invalid))),
            // ------------------------------------------------------------
            // value unchanged + status unchanged -> None
            // ------------------------------------------------------------
            (07, Some(1), invalid, None),
            // ------------------------------------------------------------
            // value unchanged + status changed
            // ------------------------------------------------------------
            (08, Some(1), ok, Some((1, ok))),
            // ------------------------------------------------------------
            // переход через 0
            // ------------------------------------------------------------
            (09, Some(-1), ok, Some((-1, ok))),
            (10, Some(1), ok, Some((1, ok))),
            // ------------------------------------------------------------
            // границы f32
            // ------------------------------------------------------------
            (11, Some(i64::MIN), ok, Some((i64::MIN, ok))),
            (12, Some(i64::MAX), ok, Some((i64::MAX, ok))),
            (13, Some(i64::MIN), ok, Some((i64::MIN, ok))),
            (14, Some(-i64::MAX), ok, Some((-i64::MAX, ok))),
            // ------------------------------------------------------------
            // одинаковые boundary -> None
            // ------------------------------------------------------------
            (15, Some(-i64::MAX), ok, None),
        ];
        let mut parse = SlmpParseInt::new(&dbg, 0, Name::new(&dbg, "SlmpParseInt").join(),
            &PointConf {
                id: 0,
                name: Name::new(&dbg, "SlmpParseInt").join(),
                type_: PointType::Int,
                history: Default::default(), alarm: Default::default(),
                address: Some(PointConfAddress {offset: Some(0), bit: None}),
                filters: None, comment: None,
            },
            Box::new(FilterEmpty::<i64>::new(None)),
        ).unwrap();
        let ts = Utc::now();
        for (step, input_value, input_status, target) in test_data {
            log::debug!("{dbg} | step {step} | input: {:?} {:?}", input_value, input_status);
            let t = Instant::now();
            let result = parse.to_point(input_value, input_status, ts);
            log::debug!("{dbg} | step {step} | elapsed {:?}", t.elapsed());
            match (&result, &target) {
                (None, None) => {},
                (Some(Point::Int(p)), Some((target_value, target_status))) => {
                    assert!(
                        p.value == *target_value,
                        "{dbg} | step {step} | \nvalue result: {:?}\ntarget: {:?}",
                        p.value,
                        target_value
                    );
                    assert!(
                        p.status == *target_status,
                        "{dbg} | step {step} | \nstatus result: {:?}\ntarget: {:?}",
                        p.status,
                        target_status
                    );
                }
                _ => panic!(
                    "{dbg} | step {step} | \nresult: {:?}\ntarget: {:?}",
                    result,
                    target
                ),
            };
        }
        test_duration.exit();
    }
    ///
    /// Testing [SlmpParseInt].add_raw
    ///
    #[test]
    fn add_raw() {
        DebugSession::new().filter(LogLevel::Debug).init().unwrap();
        init_once();
        init_each();
        log::debug!("");
        let dbg = Dbg::own("SlmpParseInt-add_raw");
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
        test_duration.run().unwrap();
        // ----------------------------------------------------------------
        // Таблица тестов
        // ----------------------------------------------------------------
        let test_data = [
            // ------------------------------------------------------------
            // нормальные значения
            // ------------------------------------------------------------
            (
                0,
                &[][..],
                None,
                "first packet broken -> no event",
            ),            (
                1,
                & i16::to_le_bytes(0)[..],
                Some((0, Status::Ok)),
                "first value",
            ),
            (
                2,
                & i16::to_le_bytes(0)[..],
                None,
                "value not changed",
            ),
            (
                3,
                & i16::to_le_bytes(2)[..],
                Some((2, Status::Ok)),
                "value changed",
            ),
            // ------------------------------------------------------------
            // граничные значения  i16
            // ------------------------------------------------------------
            (
                4,
                & i16::to_le_bytes( i16::MAX)[..],
                Some(( i16::MAX, Status::Ok)),
                " i16 MAX",
            ),
            (
                5,
                & i16::to_le_bytes( i16::MIN)[..],
                Some(( i16::MIN, Status::Ok)),
                " i16 MIN",
            ),
            (
                6,
                & i16::to_le_bytes( i16::MAX)[..],
                Some(( i16::MAX, Status::Ok)),
                " i16 INF",
            ),
            (
                7,
                & i16::to_le_bytes( i16::MIN)[..],
                Some(( i16::MIN, Status::Ok)),
                " i16 NEG_INF",
            ),
            // ------------------------------------------------------------
            // ошибка парсинга
            // ------------------------------------------------------------
            (
                8,
                &[0x0],
                Some(( i16::MIN, Status::Invalid)),
                "slice too short -> error",
            ),
            (
                9,
                & i16::to_le_bytes( i16::MIN)[..],
                Some(( i16::MIN, Status::Ok)),
                "normal value ->  i16::MIN",
            ),
            (
                10,
                &[],
                Some(( i16::MIN, Status::Invalid)),
                "empty slice -> error",
            ),
        ];
        let mut parse = SlmpParseInt::new(
            &dbg,
            0,
            Name::new(&dbg, "SlmpParseInt").join(),
            &PointConf {
                id: 0,
                name: Name::new(&dbg, "SlmpParseInt").join(),
                type_: PointType::Int,
                history: Default::default(),
                alarm: Default::default(),
                address: Some(PointConfAddress {
                    offset: Some(0),
                    bit: None,
                }),
                filters: None,
                comment: None,
            },
            Box::new(FilterEmpty::<i64>::new(None)),
        ).unwrap();
        let ts = Utc::now();
        for (step, bytes, target, description) in test_data {
            log::debug!("{dbg} | step {step} | {description} | bytes: {:?}", bytes);
            let t = Instant::now();
            let result = parse.add_raw(bytes, ts);
            log::debug!("{dbg} | step {step} | elapsed {:?}", t.elapsed());
            match (&result, &target) {
                (None, None) => {}
                (Some(Point::Int(p)), Some((target_value, target_status))) => {
                    assert!(
                        p.value as i16 == *target_value,
                        "{dbg} | step {step} | value result: {:?}, target: {:?}",
                        p.value,
                        target_value
                    );
                    assert!(
                        p.status == *target_status,
                        "{dbg} | step {step} | status result: {:?}, target: {:?}",
                        p.status,
                        target_status
                    );
                }
                _ => panic!("{dbg} | step {step} | \nresult: {:?}\ntarget: {:?}", result, target),
            }
        }
        test_duration.exit();
    }
}
