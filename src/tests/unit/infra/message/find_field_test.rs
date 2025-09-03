#[cfg(test)]

use std::{sync::Once, time::{Duration, Instant}};
use sal_core::{dbg::Dbg, error::Error};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
use crate::infra::message::{Bytes, FindField, MessageParse};
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
/// Testing [FixedField].parse
#[test]
fn parse_u8() {
    DebugSession::init(LogLevel::Debug, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("FindField-parse");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        (01, vec![0x00], Err(())),
        (02, vec![0x00], Err(())),
        (03, vec![0x40], Ok((64, vec![]))),  // dec 64
        (04, vec![0x00], Err(())),
        (05, vec![0x00], Err(())),
        (06, vec![0x40,0x01,0x02], Ok((64, vec![0x01,0x02]))),   // dec 64
        (07, vec![0xDE], Err(())),  // dec 222
    ];
    let dbg1 = dbg.clone();
    let mut fixed_field = FindField::new(
        &dbg, 1,
        move |bytes| {
            log::debug!("{dbg1} | Bytes to u8: {:?}", bytes);
            match bytes.try_into() {
                Ok(bytes) => {
                    let val = u8::from_be_bytes(bytes);
                    log::debug!("{dbg1} | Value u8: {:?}", val);
                    match val == 64 {
                        true => Ok(Some(val)),
                        false => Ok(None),
                    }
                }
                Err(_) => todo!(),
            }
        },
        Terminator::new(),
    );
    for (step, bytes, target) in test_data {
        log::debug!("{dbg} | Bytes: {:?}", bytes);
        let t = Instant::now();
        let result = fixed_field.parse(bytes);
        match (&result, &target) {
            (Ok((_, result, remainder)), Ok((target, target_remainder))) => {
                log::debug!("{dbg} | Elapsed: {:?}", t.elapsed());
                assert!(result == target, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
                assert!(remainder == target_remainder, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, remainder, target_remainder);
            }
            (Err(_), Err(_)) => {},
            _ => panic!("{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result, target),
        };
    }
    test_duration.exit();
}
///
/// Testing [FixedField].parse
#[test]
fn parse_u16() {
    DebugSession::init(LogLevel::Debug, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("FindField-parse");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        (01, vec![0x00], Err(())),
        (02, vec![0x04], Err(())),  // dec 12
        (03, vec![0xF0], Ok((1264, vec![]))), // dec 64
        (04, vec![0x00], Err(())),
        (05, vec![0x04,0xF0,0x01,0x02], Ok((1264, vec![0x01,0x02]))),
        (06, vec![0x04], Err(())),   // dec 12
        (07, vec![0xDE], Err(())),  // dec 222
        (08, vec![0x00], Err(())),
        (09, vec![0x04], Err(())),  // dec 12
        (10, vec![0xF0], Ok((1264, vec![]))), // dec 64
    ];
    let dbg1 = dbg.clone();
    let mut fixed_field = FindField::new(
        &dbg, 2,
        move |bytes| {
            log::debug!("{dbg1} | Bytes to u8: {:?}", bytes);
            match bytes.try_into() {
                Ok(bytes) => {
                    let val = u16::from_be_bytes(bytes);
                    log::debug!("{dbg1} | Value u8: {:?}", val);
                    match val == 1264 {
                        true => Ok(Some(val)),
                        false => Ok(None),
                    }
                }
                Err(_) => todo!(),
            }
        },
        Terminator::new(),
    );
    for (step, bytes, target) in test_data {
        log::debug!("{dbg} | Bytes: {:?}", bytes);
        let t = Instant::now();
        let result = fixed_field.parse(bytes);
        match (&result, &target) {
            (Ok((_, result, remainder)), Ok((target, target_remainder))) => {
                log::debug!("{dbg} | Elapsed: {:?}", t.elapsed());
                assert!(result == target, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
                assert!(remainder == target_remainder, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, remainder, target_remainder);
            }
            (Err(_), Err(_)) => {},
            _ => panic!("{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result, target),
        };
    }
    test_duration.exit();
}
///
/// Testing [FixedField].parse
#[test]
fn parse_u32() {
    DebugSession::init(LogLevel::Debug, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("FindField-parse");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        (01, vec![0x00], Err(())),
        (02, vec![0x01], Err(())),  // dec 12
        (03, vec![0x02], Err(())),  // dec 64
        (04, vec![0x03], Err(())),  // dec 12
        (05, vec![0x04], Ok((16909060, vec![]))), // dec 64
        (06, vec![0x01], Err(())),
        (07, vec![0x00], Err(())),
        (08, vec![0x01], Err(())),  // dec 12
        (09, vec![0x02], Err(())),  // dec 64
        (10, vec![0x03,0x04,0x01,0x02], Ok((16909060, vec![0x01,0x02]))),
        (11, vec![0x04], Err(())),   // dec 12
        (12, vec![0xDE], Err(())),  // dec 222
        (13, vec![0x00], Err(())),
        (14, vec![0x01], Err(())),  // dec 64
        (15, vec![0x02], Err(())),  // dec 12
        (16, vec![0x03], Err(())),  // dec 12
        (17, vec![0x04], Ok((16909060, vec![]))), // dec 64
    ];
    let dbg1 = dbg.clone();
    let mut fixed_field = FindField::new(
        &dbg, 4,
        move |bytes| {
            log::debug!("{dbg1} | Bytes to u8: {:?}", bytes);
            match bytes.try_into() {
                Ok(bytes) => {
                    let val = u32::from_be_bytes(bytes);
                    log::debug!("{dbg1} | Value u8: {:?}", val);
                    match val == 16909060 {
                        true => Ok(Some(val)),
                        false => Ok(None),
                    }
                }
                Err(_) => panic!("{dbg1} | Error parsing u32 from bytes {:?}", bytes),
            }
        },
        Terminator::new(),
    );
    for (step, bytes, target) in test_data {
        log::debug!("{dbg} | Bytes: {:?}", bytes);
        let t = Instant::now();
        let result = fixed_field.parse(bytes);
        match (&result, &target) {
            (Ok((_, result, remainder)), Ok((target, target_remainder))) => {
                log::debug!("{dbg} | Elapsed: {:?}", t.elapsed());
                assert!(result == target, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
                assert!(remainder == target_remainder, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, remainder, target_remainder);
            }
            (Err(_), Err(_)) => {},
            _ => panic!("{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result, target),
        };
    }
    test_duration.exit();
}
///
/// Used locally for testing only
pub struct Terminator {}
impl Terminator {
    pub fn new() -> Self {
        Self {}
    }
}
impl<'a> MessageParse<'a, (), (), Bytes> for Terminator {
    ///
    /// Resets passed `bytes`
    fn parse(&mut self, bytes: Bytes) -> Result<((), (), Bytes), Error> {
        Ok(((), (), bytes))
    }
}
