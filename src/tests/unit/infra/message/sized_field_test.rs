#[cfg(test)]

use std::{sync::Once, time::{Duration, Instant}};
use sal_core::{dbg::Dbg, error::Error};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
use crate::infra::message::{Bytes, FixedField, MessageParse, SizedField};
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
/// Testing [SizedField].parse
#[test]
fn parse() {
    DebugSession::init(LogLevel::Debug, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("SizedField-parse");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        (01, vec![0x00], Err(())),
        (02, vec![0x0b], Err(())),
        (03, vec![0x48], Err(())),
        (04, vec![0x61], Err(())),
        (05, vec![0x6c], Err(())),
        (06, vec![0x6c], Err(())),
        (07, vec![0x6f,0x20], Err(())),
        (08, vec![0x57,0x69,0x72,0x6c,0x64], Ok((11, "Hallo Wirld", vec![]))),
        (11, vec![0x00], Err(())),
        (12, vec![0x27], Err(())),
        (13, vec![0x54,0x68,0x69], Err(())),  // dec 87
        (14, vec![0x73,0x20,0x69,0x73,0x20], Err(())),   // 444
        (15, vec![0x70,0x61,0x72,0x73,0x65,0x64,0x20,0x66], Err(())),   // 444
        (16, vec![0x69,0x65,0x6c,0x64,0x20,0x6f,0x66], Err(())),   // 444
        (17, vec![0x20,0x76,0x61,0x72,0x69,0x61,0x62,0x6c,0x65], Err(())),   // 444
        (18, vec![0x20,0x6c,0x65,0x6e,0x67,0x74,0x68,0x01,0x02,0x03], Ok((39, "This is parsed field of variable length", vec![0x01,0x02,0x03]))),
    ];
    let dbg1 = dbg.clone();
    let dbg2 = dbg.clone();
    let mut fixed_field = SizedField::new(
        &dbg,
        |((), ()), size| *size as usize,
        move |bytes| {
            log::debug!("{dbg1} | Bytes to String: {:?}", bytes);
            match bytes.try_into() {
                Ok(bytes) => {
                    let val = String::from_utf8_lossy(bytes).into_owned();
                    log::debug!("{dbg1} | Value String: {:?}", val);
                    Ok(val)
                }
                Err(_) => panic!("{dbg1} | Error parsing String from bytes {:?}", bytes),
            }
        },
        FixedField::new(
            &dbg, 2,
            move |bytes| {
                log::debug!("{dbg2} | Bytes to u16: {:?}", bytes);
                match bytes.try_into() {
                    Ok(bytes) => {
                        let val = u16::from_be_bytes(bytes);
                        log::debug!("{dbg2} | Value u16: {:?}", val);
                        Ok(val)
                    }
                    Err(_) => panic!("{dbg2} | Error parsing u16 from bytes {:?}", bytes),
                }
            },
            Terminator::new(),
        ),
    );
    for (step, bytes, target) in test_data {
        log::debug!("{dbg} | Bytes: {:?}", bytes);
        let t = Instant::now();
        match (fixed_field.parse(bytes), target) {
            (Ok(((_, size), result, remainder)), Ok((target_size, target, target_remainder))) => {
                log::debug!("{dbg} | Elapsed: {:?}", t.elapsed());
                assert!(size == target_size, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, size, target_size);
                assert!(result == target, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
                assert!(remainder == target_remainder, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, remainder, target_remainder);
            }
            (Ok(result), Err(target)) => panic!("{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result, target),
            (Err(result), Ok(target)) => panic!("{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result, target),
            (Err(_), Err(_)) => {},
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
