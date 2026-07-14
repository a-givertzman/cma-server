#[cfg(test)]

use std::{sync::Once, time::{Duration, Instant}};
use sal_core::{dbg::Dbg, error::Error};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::infra::message::{Bytes, FixedField, MessageParse};
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
fn parse() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("FixedField-parse");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        // (01, vec![0x00], Err(())),
        // (02, vec![0x00], Err(())),
        (03, vec![0x40], Err(())),  // dec 64
        (04, vec![0x00], Err(())),
        (05, vec![0x00], Err(())),
        (06, vec![0x00], Err(())),
        (07, vec![0xDE], Err(())),  // dec 222
        (08, vec![0x00,0x0B], Ok((64, 222, 11, vec![]))),
        (09, vec![0x57], Err(())),  // dec 87
        (10, vec![0x00,0x00,0x01,0xBC], Err(())),   // 444
        (11, vec![0x00,0x21,0x01,0x02], Ok((87, 444, 33, vec![0x01,0x02]))),
    ];
    let mut fixed_field = FixedField::new(
        &dbg, 2,
        |dbg, bytes| {
            log::debug!("{dbg} | Bytes to u16: {:?}", bytes);
            match bytes.try_into() {
                Ok(bytes) => {
                    let val = u16::from_be_bytes(bytes);
                    log::debug!("{dbg} | Value u16: {:?}", val);
                    Ok(val)
                }
                Err(_) => panic!("{dbg} | Error parsing u16 from bytes {:?}", bytes),
            }
        },
        FixedField::new(
            &dbg, 4,
            |dbg, bytes| {
                log::debug!("{dbg} | Bytes to u32: {:?}", bytes);
                match bytes.try_into() {
                    Ok(bytes) => {
                        let val = u32::from_be_bytes(bytes);
                        log::debug!("{dbg} | Value u32: {:?}", val);
                        Ok(val)
                    }
                    Err(_) => panic!("{dbg} | Error parsing u32 from bytes {:?}", bytes),
                }
            },
            FixedField::new(
                &dbg, 1,
                |_, bytes| {
                    match bytes.try_into() {
                        Ok(bytes) => Ok(u8::from_be_bytes(bytes)),
                        Err(_) => todo!(),
                    }
                },
                Terminator::new(),
            ),
        ),
    );
    for (step, bytes, target) in test_data {
        log::debug!("{dbg} | Bytes: {:?}", bytes);
        let t = Instant::now();
        match (fixed_field.parse(bytes), target) {
            (Ok((((_, result_u8), result_u32), result_u16, remainder)), Ok((target_u8, target_u32, target_u16, target_remainder))) => {
                log::debug!("{dbg} | Elapsed: {:?}", t.elapsed());
                assert!(result_u8 == target_u8, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result_u8, target_u8);
                assert!(result_u32 == target_u32, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result_u32, target_u32);
                assert!(result_u16 == target_u16, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result_u16, target_u16);
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
impl MessageParse<(), (), Bytes> for Terminator {
    ///
    /// Resets passed `bytes`
    fn parse(&mut self, bytes: Bytes) -> Result<((), (), Bytes), Error> {
        Ok(((), (), bytes))
    }
}
