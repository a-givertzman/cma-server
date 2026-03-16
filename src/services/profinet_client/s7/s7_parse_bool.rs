use sal_core::error::Error;
use sal_sync::services::{
    entity::{Cot, Point, PointConf, PointConfAddress, PointHlr, Status},
    types::Bool,
};
use chrono::{DateTime, Utc};
use crate::{domain::filter::filter::{Filter, FilterEmpty}, services::profinet_client::parse_point::ParsePoint};

///
///
#[derive(Debug)]
pub struct S7ParseBool {
    txid: usize,
    name: String,
    value: Box<dyn Filter<Item = bool>>,
    status: Box<dyn Filter<Item = Status>>,
    offset: Option<u32>,
    bit: Option<u8>,
    // pub history: PointConfHistory,
    // pub alarm: Option<u8>,
    // pub comment: Option<String>,
}
impl S7ParseBool {
    ///
    ///
    pub fn new(
        txid: usize,
        name: String,
        config: &PointConf,
    ) -> S7ParseBool {
        S7ParseBool {
            txid,
            name,
            value: Box::new(FilterEmpty::<bool>::new(None)),
            status: Box::new(FilterEmpty::<Status>::new(Some(Status::Invalid))),
            offset: config.clone().address.unwrap_or(PointConfAddress::empty()).offset,
            bit: config.clone().address.unwrap_or(PointConfAddress::empty()).bit,
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
        bit: usize,
    ) -> Result<bool, Error> {
        let value = bytes.get(start..(start + 2))
            .and_then(|bytes| bytes.try_into().ok())
            .map(|bytes| (i16::from_be_bytes(bytes) >> bit) & 1)
            .ok_or_else(|| Error::new(&self.name, "convert").err("Wrong bytes length"))?;
        Ok(value > 0)
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
    fn to_point(&mut self, value: Option<bool>, status: Status, timestamp: DateTime<Utc>) -> Option<Point> {
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
        Some(Point::Bool(PointHlr::new(
            self.txid,
            &self.name,
            Bool(value),
            status,
            Cot::Inf,
            timestamp,
        )))
    }
    //
    //
    fn add_raw(&mut self, bytes: &[u8], timestamp: DateTime<Utc>) -> Option<Point> {
        let result = self.convert(
            bytes,
            self.offset.unwrap() as usize,
            self.bit.unwrap() as usize,
        );
        match result {
            Ok(value) => self.to_point(Some(value), Status::Ok, timestamp),
            Err(e) => {
                log::warn!("{}.add_raw | convertion error: {:?}", self.name, e);
                self.to_point(None, Status::Invalid, timestamp)
            }
        }
    }
}
///
impl ParsePoint for S7ParseBool {
    //
    //
    fn next(&mut self, bytes: &[u8], timestamp: DateTime<Utc>) -> Option<Point> {
        self.add_raw(bytes, timestamp)
    }
    //
    //
    fn next_status(&mut self, status: Status) -> Option<Point> {
        self.to_point(None, status, Utc::now())
    }
    //
    //
    fn address(&self) -> PointConfAddress {
        PointConfAddress { offset: self.offset, bit: self.bit }
    }
}
///
/// Tests
#[cfg(test)]
mod s7_parse_bool_test {
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
    /// Testing [S7ParseBool].to_point
    ///
    #[test]
    fn to_point() {
        DebugSession::new().filter(LogLevel::Debug).init();
        init_once();
        init_each();
        log::debug!("");
        let dbg = Dbg::own("S7ParseBool-to_point");
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
            (03, Some(false), ok, Some((false, ok))),
            // ------------------------------------------------------------
            // value unchanged -> None
            // ------------------------------------------------------------
            (04, Some(false), ok, None),
            // ------------------------------------------------------------
            // status changed -> emit last value
            // ------------------------------------------------------------
            (05, None, invalid, Some((false, invalid))),
            // ------------------------------------------------------------
            // value changed
            // ------------------------------------------------------------
            (06, Some(true), invalid, Some((true, invalid))),
            // ------------------------------------------------------------
            // value unchanged + status unchanged -> None
            // ------------------------------------------------------------
            (07, Some(true), invalid, None),
            // ------------------------------------------------------------
            // value unchanged + status changed
            // ------------------------------------------------------------
            (08, Some(true), ok, Some((true, ok))),
            // ------------------------------------------------------------
            // переход через 0
            // ------------------------------------------------------------
            (09, Some(false), ok, Some((false, ok))),
            (10, Some(true), ok, Some((true, ok))),
        ];
        let mut parse = S7ParseBool::new(0, Name::new(&dbg, "S7ParseBool").join(),
            &PointConf {
                id: 0,
                name: Name::new(&dbg, "S7ParseBool").join(),
                type_: PointType::Bool,
                history: Default::default(), alarm: Default::default(),
                address: Some(PointConfAddress {offset: Some(0), bit: None}),
                filters: None, comment: None,
            },
        );
        let ts = Utc::now();
        for (step, input_value, input_status, target) in test_data {
            log::debug!("{dbg} | step {step} | input: {:?} {:?}", input_value, input_status);
            let t = Instant::now();
            let result = parse.to_point(input_value, input_status, ts);
            log::debug!("{dbg} | step {step} | elapsed {:?}", t.elapsed());
            match (&result, &target) {
                (None, None) => {},
                (Some(Point::Bool(p)), Some((target_value, target_status))) => {
                    assert!(
                        p.value.0 == *target_value,
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
    /// Testing [S7ParseBool].add_raw
    ///
    #[test]
    fn add_raw() {
        fn to_be_bytes(bit: u8, v: bool) -> [u8; 2] {
            let mut x: i16 = 0;
            if v {
                x |= 1 << bit; // Установить в true (1)
            }
            x.to_be_bytes()
        }
        DebugSession::new().filter(LogLevel::Debug).init();
        init_once();
        init_each();
        log::debug!("");
        let dbg = Dbg::own("S7ParseBool-add_raw");
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
        test_duration.run().unwrap();
        // ----------------------------------------------------------------
        // Таблица тестов
        // ----------------------------------------------------------------
        // Бит в котом режит наш bool
        let bit = 3;
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
                &to_be_bytes(bit, false)[..],
                Some((false, Status::Ok)),
                "first value",
            ),
            (
                2,
                &to_be_bytes(bit, false)[..],
                None,
                "value not changed",
            ),
            (
                3,
                &to_be_bytes(bit, true)[..],
                Some((true, Status::Ok)),
                "value changed",
            ),
            (
                4,
                &to_be_bytes(bit, false)[..],
                Some(( false, Status::Ok)),
                "value changed",
            ),
            // ------------------------------------------------------------
            // ошибка парсинга
            // ------------------------------------------------------------
            (
                8,
                &[0x0],
                Some(( false, Status::Invalid)),
                "slice too short -> Invalid",
            ),
            (
                9,
                &to_be_bytes(bit, false)[..],
                Some(( false, Status::Ok)),
                "normal value ->  false",
            ),
            (
                10,
                &[],
                Some(( false, Status::Invalid)),
                "empty slice -> error",
            ),
        ];
        let mut parse = S7ParseBool::new(
            0,
            Name::new(&dbg, "S7ParseBool").join(),
            &PointConf {
                id: 0,
                name: Name::new(&dbg, "S7ParseBool").join(),
                type_: PointType::Bool,
                history: Default::default(),
                alarm: Default::default(),
                address: Some(PointConfAddress {
                    offset: Some(0),
                    bit: Some(bit),
                }),
                filters: None,
                comment: None,
            },
        );
        let ts = Utc::now();
        for (step, bytes, target, description) in test_data {
            log::debug!("{dbg} | step {step} | {description} | bytes: {:?}", bytes);
            let t = Instant::now();
            let result = parse.add_raw(bytes, ts);
            log::debug!("{dbg} | step {step} | elapsed {:?}", t.elapsed());
            match (&result, &target) {
                (None, None) => {}
                (Some(Point::Bool(p)), Some((target_value, target_status))) => {
                    assert!(
                        p.value.0 == *target_value,
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
