use sal_core::error::Error;
use sal_sync::services::entity::{
    Cot, Point, PointConf, PointConfAddress, PointHlr, Status
};
use std::array::TryFromSliceError;
use chrono::{DateTime, Utc};
use crate::{domain::filter::filter::{Filter, FilterEmpty}, services::profinet_client::parse_point::ParsePoint};
///
///
#[derive(Debug)]
pub struct S7ParseInt {
    txid: usize,
    name: String,
    value: Box<dyn Filter<Item = i64>>,
    status: Box<dyn Filter<Item = Status>>,
    offset: Option<u32>,
    // pub history: PointConfHistory,
    // pub alarm: Option<u8>,
    // pub comment: Option<String>,
}
//
//
impl S7ParseInt {
    ///
    ///
    pub fn new(
        txid: usize,
        name: String,
        config: &PointConf,
        filter: Box<dyn Filter<Item = i64>>,
    ) -> S7ParseInt {
        S7ParseInt {
            txid,
            name,
            value: filter,
            status: Box::new(FilterEmpty::<Status>::new(Some(Status::Invalid))),
            offset: config.clone().address.unwrap_or(PointConfAddress::empty()).offset,
            // history: config.history.clone(),
            // alarm: config.alarm,
            // comment: config.comment.clone(),
        }
    }
    //
    //
    fn convert(
        &self,
        bytes: &[u8],
        start: usize,
        _bit: usize,
    ) -> Result<i16, Error> {
        let value = bytes.get(start..(start + 2))
            .and_then(|bytes| bytes.try_into().ok())
            .map(i16::from_be_bytes)
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
    fn to_point(&mut self, value: Option<i64>, status: Status, timestamp: DateTime<Utc>) -> Option<Point> {
        let value_changed = value.and_then(|v| self.value.add(v));
        let status_changed = self.status.add(status);
        // log::trace!("{}.to_point | value_changed: {:?}  |  status_changed {:?}", self.name, value_changed, status_changed);
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
            timestamp,
        )))
    }
    //
    //
    fn add_raw(&mut self, bytes: &[u8], timestamp: DateTime<Utc>) -> Option<Point>{
        let result = self.convert(bytes, self.offset.unwrap() as usize, 0);
        match result {
            Ok(value) => self.to_point(Some(value as i64), Status::Ok, timestamp),
            Err(e) => {
                log::warn!("{}.add_raw | convertion error: {:?}", self.name, e);
                self.to_point(None, Status::Invalid, timestamp)
            }
        }
    }
}
//
//
impl ParsePoint for S7ParseInt {
    //
    fn next(&mut self, bytes: &[u8], timestamp: DateTime<Utc>) -> Option<Point> {
        self.add_raw(bytes, timestamp)
    }
    //
    fn next_status(&mut self, status: Status) -> Option<Point> {
        self.to_point(None, status, Utc::now())
    }
    //
    fn address(&self) -> PointConfAddress {
        PointConfAddress { offset: self.offset, bit: None }
    }
}
///
/// Tests
#[cfg(test)]
mod s7_parse_int_test {
    use std::{sync::Once, time::{Duration, Instant}};
    use super::*;
    use chrono::Utc;
    use debugging::session::debug_session::{DebugSession, LogLevel};
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
    /// Testing [S7ParseInt].to_point
    ///
    #[test]
    fn to_point() {
        DebugSession::new().filter(LogLevel::Debug).init();
        init_once();
        init_each();
        log::debug!("");
        let dbg = Dbg::own("S7ParseInt-to_point");
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
        let mut parse = S7ParseInt::new(0, Name::new(&dbg, "S7ParseInt").join(),
            &PointConf {
                id: 0,
                name: Name::new(&dbg, "S7ParseInt").join(),
                type_: PointType::Int,
                history: Default::default(), alarm: Default::default(),
                address: Some(PointConfAddress {offset: Some(0), bit: None}),
                filters: None, comment: None,
            },
            Box::new(FilterEmpty::<i64>::new(None)),
        );
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
    /// Testing [S7ParseInt].add_raw
    ///
    #[test]
    fn add_raw() {
        DebugSession::new().filter(LogLevel::Debug).init();
        init_once();
        init_each();
        log::debug!("");
        let dbg = Dbg::own("S7ParseInt-add_raw");
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
                & i16::to_be_bytes(0)[..],
                Some((0, Status::Ok)),
                "first value",
            ),
            (
                2,
                & i16::to_be_bytes(0)[..],
                None,
                "value not changed",
            ),
            (
                3,
                & i16::to_be_bytes(2)[..],
                Some((2, Status::Ok)),
                "value changed",
            ),
            // ------------------------------------------------------------
            // граничные значения  i16
            // ------------------------------------------------------------
            (
                4,
                & i16::to_be_bytes( i16::MAX)[..],
                Some(( i16::MAX, Status::Ok)),
                " i16 MAX",
            ),
            (
                5,
                & i16::to_be_bytes( i16::MIN)[..],
                Some(( i16::MIN, Status::Ok)),
                " i16 MIN",
            ),
            (
                6,
                & i16::to_be_bytes( i16::MAX)[..],
                Some(( i16::MAX, Status::Ok)),
                " i16 INF",
            ),
            (
                7,
                & i16::to_be_bytes( i16::MIN)[..],
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
                & i16::to_be_bytes( i16::MIN)[..],
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
        let mut parse = S7ParseInt::new(
            0,
            Name::new(&dbg, "S7ParseInt").join(),
            &PointConf {
                id: 0,
                name: Name::new(&dbg, "S7ParseInt").join(),
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
        );
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
                _ => {
                    panic!(
                        "{dbg} | step {step} | \nresult: {:?}\ntarget: {:?}",
                        result,
                        target
                    );
                }
            }
        }
        test_duration.exit();
    }
}
