#[cfg(test)]

use std::{sync::Once, time::{Duration, Instant}};
use sal_core::{dbg::Dbg, error::Error};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};

use crate::infra::message::{Bytes, FixedField, FromBytes, MessageParse};
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
fn functionality() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("functionality");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        (01, vec![0x01,0x01], Err(())),
        (02, vec![0x00,0x02,0x02,0x02], Ok((11, 222))),
        (03, vec![0x03,0x03], Err(())),
        (04, vec![0x00,0x04,0x04,0x04], Ok((33, 444))),
    ];
    let mut fixed_field = FixedField::new(
        &dbg,
        2,
        |bytes| {
            match bytes.try_into() {
                Ok(bytes) => Ok(u16::from_be_bytes(bytes)),
                Err(_) => todo!(),
            }
        },
        FixedField::new(
            &dbg,
            4,
            |bytes| {
                match bytes.try_into() {
                    Ok(bytes) => Ok(u32::from_be_bytes(bytes)),
                    Err(_) => todo!(),
                }
            },
            Terminator::new(),
        ),
    );
    for (step, bytes, target) in test_data {
        // Result<((((), ()), u32), u16, Vec<u8>), Error>
        match (fixed_field.parse(bytes), target) {
            (Ok(((_, result_u32), result_u16, _)), Ok((target_u16, target_u32))) => {
                assert!(result_u32 == target_u32, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result_u32, target_u32);
                assert!(result_u16 == target_u16, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result_u16, target_u16);
            }
            (Ok(_), Err(_)) => todo!(),
            (Err(_), Ok(_)) => todo!(),
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
