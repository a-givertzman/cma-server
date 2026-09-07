use chrono::{DateTime, Utc};
use sal_core::error::Error;
use sal_sync::services::entity::{
    Cot, Point, PointConf, PointConfAddress, PointType, PointHlr, Status,
};
use crate::{domain::filter::filter::{Filter, FilterEmpty}, services::slmp_client::slmp::ParsePoint};
///
/// Used for parsing configured point from slice of bytes read from device
#[derive(Debug)]
pub struct SlmpParseReal {
    id: String,
    typ: PointType,
    txid: usize,
    name: String,
    value: Box<dyn Filter<Item = f32> + Send>,
    status: Box<dyn Filter<Item = Status> + Send>,
    offset: Option<u32>,
    // history: PointConfHistory,
    // alarm: Option<u8>,
    // comment: Option<String>,
}
//
//
impl SlmpParseReal {
    ///
    /// Size in the bytes in the Device address area
    const SIZE: usize = 4;
    ///
    ///
    pub fn new(
        txid: usize,
        name: String,
        config: &PointConf,
        filter: Box<dyn Filter<Item = f32> + Send>,
    ) -> SlmpParseReal {
        SlmpParseReal {
            id: format!("SlmpParseReal"),
            typ: config.type_.clone(),
            txid,
            value: filter,
            status: Box::new(FilterEmpty::<Status>::new(Some(Status::Invalid))),
            name,
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
    ) -> Result<f32, Error> {
        let value = bytes.get(start..(start + Self::SIZE))
            .and_then(|bytes| bytes.try_into().ok())
            .map(f32::from_le_bytes)
            .ok_or_else(|| Error::new(&self.name, "convert").err("Wrong bytes length"))?;
        if value.is_nan() {
            return Err(Error::new(&self.name, "convert").err("NAN parsed"));
        }
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
    fn to_point(&mut self, value: Option<f32>, status: Status, ts: DateTime<Utc>) -> Option<Point> {
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
        Some(Point::Real(PointHlr::new(
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
        let result = self.convert(bytes, self.offset.unwrap() as usize, 0);
        match result {
            Ok(value) => self.to_point(Some(value), Status::Ok, ts),
            Err(e) => {
                log::warn!("SlmpParseReal.add_raw | convertion error: {:?}", e);
                self.to_point(None, Status::Invalid, ts)
            }
        }
    }
}
//
//
impl ParsePoint for SlmpParseReal {
    // //
    // //
    // fn type_(&self) -> PointType {
    //     self.type_.clone()
    // }
    //
    //
    fn next(&mut self, bytes: &[u8], ts: DateTime<Utc>) -> Option<Point> {
        self.add_raw(bytes, ts)
    }
    //
    //
    fn next_status(&mut self, status: Status, ts: DateTime<Utc>) -> Option<Point> {
        self.to_point(None, status, ts)
    }
    //
    //
    fn address(&self) -> PointConfAddress {
        PointConfAddress { offset: self.offset, bit: None }
    }
    //
    //
    fn size(&self) -> usize {
        Self::SIZE
    }
    //
    //
    fn to_bytes(&self, point: &Point) -> Result<Vec<u8>, Error> {
        match point {
            Point::Real(point) => Ok(point.value.to_le_bytes().to_vec()),
            Point::Double(_) => Ok(point.to_real().as_real().value.to_le_bytes().to_vec()),
            _ => {
                let err = Error::new(&self.name, "to_bytes").err(format!("Can't convert f32 '{}': {:?} into bytes, incompatible input type", point.name(), point.value()));
                log::warn!("{}", err);
                Err(err)
            }
        }
    }    
}
///
/// Tests
#[cfg(test)]
mod slmp_parse_real_test {
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
    /// Testing [SlmpParseReal].to_point
    ///
    #[test]
    fn to_point() {
        DebugSession::new().filter(LogLevel::Debug).init().unwrap();
        init_once();
        init_each();
        log::debug!("");
        let dbg = Dbg::own("SlmpParseReal-to_point");
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
        test_duration.run().unwrap();
        // f32 boundary values
        let f320 = 0.0f32;
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
            (03, Some(f320), ok, Some((f320, ok))),
            // ------------------------------------------------------------
            // value unchanged -> None
            // ------------------------------------------------------------
            (04, Some(f320), ok, None),
            // ------------------------------------------------------------
            // status changed -> emit last value
            // ------------------------------------------------------------
            (05, None, invalid, Some((f320, invalid))),
            // ------------------------------------------------------------
            // value changed
            // ------------------------------------------------------------
            (06, Some(1.0), invalid, Some((1.0, invalid))),
            // ------------------------------------------------------------
            // value unchanged + status unchanged -> None
            // ------------------------------------------------------------
            (07, Some(1.0), invalid, None),
            // ------------------------------------------------------------
            // value unchanged + status changed
            // ------------------------------------------------------------
            (08, Some(1.0), ok, Some((1.0, ok))),
            // ------------------------------------------------------------
            // переход через 0
            // ------------------------------------------------------------
            (09, Some(-1.0), ok, Some((-1.0, ok))),
            (10, Some(1.0), ok, Some((1.0, ok))),
            // ------------------------------------------------------------
            // границы f32
            // ------------------------------------------------------------
            (11, Some(f32::MIN), ok, Some((f32::MIN, ok))),
            (12, Some(f32::MAX), ok, Some((f32::MAX, ok))),
            (13, Some(-f32::MIN), ok, None),
            (14, Some(-f32::MAX), ok, Some((-f32::MAX, ok))),
            // ------------------------------------------------------------
            // одинаковые boundary -> None
            // ------------------------------------------------------------
            (15, Some(-f32::MAX), ok, None),
        ];
        let mut parse = SlmpParseReal::new(0, Name::new(&dbg, "SlmpParseReal").join(),
            &PointConf {
                id: 0,
                name: Name::new(&dbg, "SlmpParseReal").join(),
                type_: PointType::Real,
                history: Default::default(), alarm: Default::default(),
                address: Some(PointConfAddress {offset: Some(0), bit: None}),
                filters: None, comment: None,
            },
            Box::new(FilterEmpty::<f32>::new(None)),
        );
        let ts = Utc::now();
        for (step, input_value, input_status, target) in test_data {
            log::debug!("{dbg} | step {step} | input: {:?} {:?}", input_value, input_status);
            let t = Instant::now();
            let result = parse.to_point(input_value, input_status, ts);
            log::debug!("{dbg} | step {step} | elapsed {:?}", t.elapsed());
            match (&result, &target) {
                (None, None) => {},
                (Some(Point::Real(p)), Some((target_value, target_status))) => {
                    assert!(
                        (p.value - target_value).abs() < f32::EPSILON,
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
    /// Testing [SlmpParseReal].add_raw
    ///
    #[test]
    fn add_raw() {
        DebugSession::new().filter(LogLevel::Debug).init().unwrap();
        init_once();
        init_each();
        log::debug!("");
        let dbg = Dbg::own("SlmpParseReal-add_raw");
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
                &f32::to_le_bytes(0.0)[..],
                Some((0.0, Status::Ok)),
                "first value",
            ),
            (
                2,
                &f32::to_le_bytes(0.0)[..],
                None,
                "value not changed",
            ),
            (
                3,
                &f32::to_le_bytes(1.5)[..],
                Some((1.5, Status::Ok)),
                "value changed",
            ),
            // ------------------------------------------------------------
            // граничные значения f32
            // ------------------------------------------------------------
            (
                4,
                &f32::to_le_bytes(f32::MAX)[..],
                Some((f32::MAX, Status::Ok)),
                "f32 MAX",
            ),
            (
                5,
                &f32::to_le_bytes(f32::MIN)[..],
                Some((f32::MIN, Status::Ok)),
                "f32 MIN",
            ),
            (
                6,
                &f32::to_le_bytes(f32::INFINITY)[..],
                Some((f32::INFINITY, Status::Ok)),
                "f32 INF",
            ),
            (
                7,
                &f32::to_le_bytes(f32::NEG_INFINITY)[..],
                Some((f32::NEG_INFINITY, Status::Ok)),
                "f32 NEG_INF",
            ),
            // ------------------------------------------------------------
            // ошибка парсинга
            // ------------------------------------------------------------
            (
                8,
                &[0x00, 0x01],
                Some((f32::NEG_INFINITY, Status::Invalid)),
                "slice too short -> error",
            ),
            (
                9,
                &f32::to_le_bytes(f32::NEG_INFINITY)[..],
                Some((f32::NEG_INFINITY, Status::Ok)),
                "normal value -> f32::NEG_INFINITY",
            ),
            (
                10,
                &[],
                Some((f32::NEG_INFINITY, Status::Invalid)),
                "empty slice -> error",
            ),
            (
                11,
                &f32::to_le_bytes(f32::NAN)[..],
                None,
                // Some((f32::NEG_INFINITY, Status::Ok)),
                "NaN value",
            ),
        ];
        let mut parse = SlmpParseReal::new(
            0,
            Name::new(&dbg, "SlmpParseReal").join(),
            &PointConf {
                id: 0,
                name: Name::new(&dbg, "SlmpParseReal").join(),
                type_: PointType::Real,
                history: Default::default(),
                alarm: Default::default(),
                address: Some(PointConfAddress {
                    offset: Some(0),
                    bit: None,
                }),
                filters: None,
                comment: None,
            },
            Box::new(FilterEmpty::<f32>::new(None)),
        );
        let ts = Utc::now();
        for (step, bytes, target, description) in test_data {
            log::debug!("{dbg} | step {step} | {description} | bytes: {:?}", bytes);
            let t = Instant::now();
            let result = parse.add_raw(bytes, ts);
            log::debug!("{dbg} | step {step} | elapsed {:?}", t.elapsed());
            match (&result, &target) {
                (None, None) => {}
                (Some(Point::Real(p)), Some((target_value, target_status))) => {
                    if target_value.is_finite() {
                        assert!(
                            (p.value - target_value).abs() < f32::EPSILON,
                            "{dbg} | step {step} | value result: {:?}, target: {:?}",
                            p.value,
                            target_value
                        );
                    } else if target_value.is_nan() {
                        assert!(
                            p.value.is_nan(),
                            "{dbg} | step {step} | value result: {:?}, target: {:?}",
                            p.value,
                            target_value
                        );
                    } else {
                        assert!(
                            p.value.is_infinite() == target_value.is_infinite(),
                            "{dbg} | step {step} | value result: {:?}, target: {:?}",
                            p.value,
                            target_value
                        );
                    }
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
