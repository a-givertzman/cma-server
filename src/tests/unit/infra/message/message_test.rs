#[cfg(test)]

use std::{sync::Once, time::{Duration, Instant}};
use sal_core::{dbg::Dbg, error::Error};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
use crate::infra::message::{Bytes, FieldSize, FixedField, Message, MessageField, MessageParse, SizedField};
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
/// Testing [Message].parse
#[test]
fn parse() {
    DebugSession::init(LogLevel::Debug, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("Message-parse");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data: &[(i32, Vec<u8>, Result<(u16, u16, u16, u8, u8, &'static str), ()>)] = &[
        // (01, vec![0x00], Err(())),
        (02, vec![0x00,0x07], Err(())),  // Transaction Identifier u16
        (03, vec![0x00,0x00], Err(())),  // Protocol Identifier u16
        (04, vec![0x00,0x0b], Err(())),  // Length Field u16
        (05, vec![0x21], Err(())),  // Unit ID 33, u8
        (06, vec![0x04], Err(())),  // Function Code 4, u8
        (07, b"Hallo Wirld".to_vec(), Ok((07, 00, 11, 33, 04, "Hallo Wirld"))),
        (12, vec![0x00,0x07], Err(())),  // Transaction Identifier u16
        (13, vec![0x00,0x00], Err(())),  // Protocol Identifier u16
        (14, vec![0x00,0x27], Err(())),  // Length Field u16
        (15, vec![0x21], Err(())),  // Unit ID 33, u8
        (16, vec![0x04], Err(())),  // Function Code 4, u8
        (17, b"This is parsed field of variable length".to_vec(), Ok((07, 00, 39, 33, 04, "This is parsed field of variable length"))),
    ];
    let (dbg1, dbg2, dbg3, dbg4, dbg5, dbg6) = (dbg.clone(), dbg.clone(), dbg.clone(), dbg.clone(), dbg.clone(), dbg.clone());
    let mut message = Message::new(
        &dbg,
        vec![MessageField::Size(FieldSize(2))],
        SizedField::new(
            &dbg,
            |((_, size), _), _| *size as usize,
            move |bytes| {
                log::debug!("{dbg1} | Bytes to String: {:?}", bytes);
                let val = String::from_utf8_lossy(bytes.try_into().expect(&format!("{dbg1} | Error parsing String from bytes {:?}", bytes))).into_owned();
                log::debug!("{dbg1} | Value String: {:?}", val);
                Ok(val)
            },
            FixedField::new(
                &dbg, 1,    // Function Code 4, u8
                move |bytes| {
                    log::debug!("{dbg2} | Bytes to u8: {:?}", bytes);
                    let val = u8::from_be_bytes(bytes.try_into().expect(&format!("{dbg2} | Error parsing u8 from bytes {:?}", bytes)));
                    log::debug!("{dbg2} | Value u8: {:?}", val);
                    Ok(val)
                },
                FixedField::new(
                    &dbg, 1,    // Unit ID 33, u8
                    move |bytes| {
                        log::debug!("{dbg3} | Bytes to u8: {:?}", bytes);
                        let val = u8::from_be_bytes(bytes.try_into().expect(&format!("{dbg3} | Error parsing u8 from bytes {:?}", bytes)));
                        log::debug!("{dbg3} | Value u8: {:?}", val);
                        Ok(val)
                    },
                    FixedField::new(
                        &dbg, 2,    // Length Field u16
                        move |bytes| {
                            log::debug!("{dbg4} | Bytes to u16: {:?}", bytes);
                            let val = u16::from_be_bytes(bytes.try_into().expect(&format!("{dbg4} | Error parsing u16 from bytes {:?}", bytes)));
                            log::debug!("{dbg4} | Value u16: {:?}", val);
                            Ok(val)
                        },
                        FixedField::new(
                            &dbg, 2,    // Protocol Identifier u16
                            move |bytes| {
                                log::debug!("{dbg5} | Bytes to u16: {:?}", bytes);
                                let val = u16::from_be_bytes(bytes.try_into().expect(&format!("{dbg5} | Error parsing u16 from bytes {:?}", bytes)));
                                log::debug!("{dbg5} | Value u16: {:?}", val);
                                Ok(val)
                            },
                            FixedField::new(
                                &dbg, 2,    // Transaction Identifier u16
                                move |bytes| {
                                    log::debug!("{dbg6} | Bytes to u16: {:?}", bytes);
                                    let val = u16::from_be_bytes(bytes.try_into().expect(&format!("{dbg6} | Error parsing u16 from bytes {:?}", bytes)));
                                    log::debug!("{dbg6} | Value u16: {:?}", val);
                                    Ok(val)
                                },
                                Terminator::new(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    for (step, bytes, target) in test_data {
        log::debug!("{dbg} | Bytes: {:?}", bytes);
        let t = Instant::now();
        match (message.parse(bytes.to_owned()), target) {
            // (((((((), ()), u16), u16), u16), u8), u8), String
            (Ok(((((((_, id), prot), size), unit), code), result)), Ok((target_id, target_prot, target_size, target_unit, target_code, target))) => {
                log::debug!("{dbg} | Elapsed: {:?}", t.elapsed());
                assert!(id == *target_id, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, id, target_id);
                assert!(prot == *target_prot, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, prot, target_prot);
                assert!(size == *target_size, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, size, target_size);
                assert!(unit == *target_unit, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, unit, target_unit);
                assert!(code == *target_code, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, code, target_code);
                assert!(result == *target, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
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
