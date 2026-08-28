#[cfg(test)]

use std::{sync::Once, time::{Duration, Instant}};
use sal_core::dbg::Dbg;
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::{DebugSession, LogLevel};
use crate::infra::message::{Field, FieldConf, FieldTerminator, FixedField, Message, SizedField};
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
/// ## Data messages (PDU)
/// 
///  Modbus Application Protocol, Protocol Data Unit (PDU) 7 Bytes length
/// 
/// ```ignore
///  Transaction ID | Protocol ID | Length Field |  Unit ID | Function Code | Data
///  ---            | ---         | ---          | ---      | ---           | ---
/// 2 Bytes         | 2 Bytes     | 2 Bytes      | 1 Bytes  | 1 Byte        | Varies
/// ```
/// 
#[test]
fn parse() {
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("Message.parse-test");
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
    let mut message = Message::new(
        &dbg,
        vec![FieldConf::Const(vec![0x00])],
        SizedField::new(
            &dbg,
            |((_, size), _), _| *size as usize,
            |dbg, bytes| {
                log::debug!("{dbg} | Bytes to String: {:?}", bytes);
                let val = String::from_utf8_lossy(bytes.try_into().expect(&format!("{dbg} | Error parsing String from bytes {:?}", bytes))).into_owned();
                log::debug!("{dbg} | Value String: {:?}", val);
                Ok(val)
            },
            FixedField::new(
                &dbg, 1,    // Function Code 4, u8
                |dbg, bytes| {
                    log::debug!("{dbg} | Bytes to u8: {:?}", bytes);
                    let val = u8::from_be_bytes(bytes.try_into().expect(&format!("{dbg} | Error parsing u8 from bytes {:?}", bytes)));
                    log::debug!("{dbg} | Value u8: {:?}", val);
                    Ok(val)
                },
                FixedField::new(
                    &dbg, 1,    // Unit ID 33, u8
                    |dbg, bytes| {
                        log::debug!("{dbg} | Bytes to u8: {:?}", bytes);
                        let val = u8::from_be_bytes(bytes.try_into().expect(&format!("{dbg} | Error parsing u8 from bytes {:?}", bytes)));
                        log::debug!("{dbg} | Value u8: {:?}", val);
                        Ok(val)
                    },
                    FixedField::new(
                        &dbg, 2,    // Length Field u16
                        |dbg, bytes| {
                            log::debug!("{dbg} | Bytes to u16: {:?}", bytes);
                            let val = u16::from_be_bytes(bytes.try_into().expect(&format!("{dbg} | Error parsing u16 from bytes {:?}", bytes)));
                            log::debug!("{dbg} | Value u16: {:?}", val);
                            Ok(val)
                        },
                        FixedField::new(
                            &dbg, 2,    // Protocol Identifier u16
                            |dbg, bytes| {
                                log::debug!("{dbg} | Bytes to u16: {:?}", bytes);
                                let val = u16::from_be_bytes(bytes.try_into().expect(&format!("{dbg} | Error parsing u16 from bytes {:?}", bytes)));
                                log::debug!("{dbg} | Value u16: {:?}", val);
                                Ok(val)
                            },
                            FixedField::new(
                                &dbg, 2,    // Transaction Identifier u16
                                |dbg, bytes| {
                                    log::debug!("{dbg} | Bytes to u16: {:?}", bytes);
                                    let val = u16::from_be_bytes(bytes.try_into().expect(&format!("{dbg} | Error parsing u16 from bytes {:?}", bytes)));
                                    log::debug!("{dbg} | Value u16: {:?}", val);
                                    Ok(val)
                                },
                                FieldTerminator::new(),
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
/// Testing [Message].build all supported kinds of [Field]'s
#[test]
fn build_all_fields() {
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("Message.build-test");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    log::debug!("{dbg} |  11 -> {:x?} be", ( 11i8).to_be_bytes());
    log::debug!("{dbg} | -12 -> {:x?} le", (-12i8).to_le_bytes());
    log::debug!("{dbg} |  13 -> {:x?} be", ( 13i16).to_be_bytes());
    log::debug!("{dbg} | -14 -> {:x?} le", (-14i16).to_le_bytes());
    log::debug!("{dbg} |  15 -> {:x?} be", ( 15i32).to_be_bytes());
    log::debug!("{dbg} | -16 -> {:x?} le", (-16i32).to_le_bytes());
    log::debug!("{dbg} |  17 -> {:x?} be", ( 17i64).to_be_bytes());
    log::debug!("{dbg} | -18 -> {:x?} le", (-18i64).to_le_bytes());
    log::debug!("{dbg} |  19 -> {:x?} be", ( 19i128).to_be_bytes());
    log::debug!("{dbg} | -20 -> {:x?} le", (-20i128).to_le_bytes());
    log::debug!("{dbg} |  21.21 -> {:x?} be", ( 21.21f32).to_be_bytes());
    log::debug!("{dbg} | -22.22 -> {:x?} le", (-22.22f32).to_le_bytes());
    log::debug!("{dbg} |  23.23 -> {:x?} be", ( 23.23f64).to_be_bytes());
    log::debug!("{dbg} | -24.24 -> {:x?} le", (-24.24f64).to_le_bytes());
    log::debug!("{dbg} | 'TestString' -> {:x?} le", "TestString".as_bytes());
    let test_data: &[(i32, Vec<Field>, Vec<u8>)] = &[
        (01,
            vec![
                Field::Const,
                Field::Byte(02),
                Field::U16(03), // Be
                Field::U16(04), // Le
                Field::U32(05),
                Field::U32(06),
                Field::Const,
                Field::U64(07),
                Field::U64(08),
                Field::U128(09),
                Field::U128(10),
                Field::I8(11),
                Field::I8(-12),
                Field::I16(13),
                Field::I16(-14),
                Field::I32(15),
                Field::I32(-16),
                Field::Const,
                Field::I64(17),
                Field::I64(-18),
                Field::I128(19),
                Field::I128(-20),
                Field::F32(21.21),
                Field::F32(-22.22),
                Field::F64(23.23),
                Field::F64(-24.24),
                Field::Const,
                Field::String("TestString".to_owned()),
            ],
            vec![
                // const    02     03 be      04 le           05 be                06 le
                0x00,0x01, 0x02, 0x00,0x03, 0x04,0x00, 0x00,0x00,0x00,0x05, 0x06,0x00,0x00,0x00,
                // const                    07 be                                    08 le
                0x00,0x02, 0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x07, 0x08,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
                // 09 be                                                                         10 le
                0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x09, 0x0a,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
                // 11 be   -12 le
                0x0b,      0xf4, 
                // 13 be   -14 le
                0x00,0x0d, 0xf2,0xff, 
                // 15 be             -16 le
                0x00,0x00,0x00,0x0f, 0xf0,0xff,0xff,0xff,
                // const
                0x00, 0x03,
                // 17 be                                 -18 le
                0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x11, 0xee,0xff,0xff,0xff,0xff,0xff,0xff,0xff,
                // 19 be                                                                         -20 le
                0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x13, 0xec,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,
                // 21.21 be          -22.22 le
                0x41,0xa9,0xae,0x14, 0x8f,0xc2,0xb1,0xc1,
                // 23.23 be                              -24.24 le
                0x40,0x37,0x3a,0xe1,0x47,0xae,0x14,0x7b, 0x3d,0x0a,0xd7,0xa3,0x70,0x3d,0x38,0xc0,
                // const
                0x00, 0x04,
                // "TestString",
                0x54,0x65,0x73,0x74,0x53,0x74,0x72,0x69,0x6e,0x67,
            ],
        ),
    ];
    let mut message = Message::new(
        &dbg,
        vec![
            FieldConf::Const(vec![0x00, 0x01]),
            FieldConf::Byte,
            FieldConf::U16Be,
            FieldConf::U16Le,
            FieldConf::U32Be,
            FieldConf::U32Le,
            FieldConf::Const(vec![0x00, 0x02]),
            FieldConf::U64Be,
            FieldConf::U64Le,
            FieldConf::U128Be,
            FieldConf::U128Le,
            FieldConf::I8Be,
            FieldConf::I8Le,
            FieldConf::I16Be,
            FieldConf::I16Le,
            FieldConf::I32Be,
            FieldConf::I32Le,
            FieldConf::Const(vec![0x00, 0x03]),
            FieldConf::I64Be,
            FieldConf::I64Le,
            FieldConf::I128Be,
            FieldConf::I128Le,
            FieldConf::F32Be,
            FieldConf::F32Le,
            FieldConf::F64Be,
            FieldConf::F64Le,
            FieldConf::Const(vec![0x00, 0x04]),
            FieldConf::String,
        ],
        FieldTerminator::new(),
    );
    for (step, input, target) in test_data {
        log::debug!("{dbg} | input: {:?}", input);
        let t = Instant::now();
        let result = message.build(input);
        log::debug!("{dbg} | Elapsed: {:?}", t.elapsed());
        assert!(result == *target, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    }
    test_duration.exit();
}
/// 
/// Testing [Message].build
/// ## Data messages (PDU)
/// 
///  Modbus Application Protocol, Protocol Data Unit (PDU) 7 Bytes length
/// 
/// ```ignore
///  Transaction ID | Protocol ID | Length Field |  Unit ID | Function Code | Data
///  ---            | ---         | ---          | ---      | ---           | ---
/// 2 Bytes         | 2 Bytes     | 2 Bytes      | 1 Bytes  | 1 Byte        | Varies
/// ```
#[test]
fn build() {
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("Message.build-test");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data: &[(i32, (u16, i32, u16, u8, u8, &'static str), Vec<u8>); 1] = &[
        (17, (07, 00, 39, 33, 04, "This is parsed field of variable length"), [&[0x00,0x07,0x00,0x00,0x00,0x27,0x21,0x04], b"This is parsed field of variable length".as_slice()].concat()),
    ];
    let mut message = Message::new(
        &dbg,
        vec![
            FieldConf::U16Be,        // Transaction Identifier u16       , index 0
            FieldConf::Const(vec![0x00, 0x00]),   // Protocol Identifier u16
            FieldConf::U16Be,        // Length Field u16                 , index 1
            FieldConf::Byte,         // Unit ID, u8                      , index 2
            FieldConf::Byte,         // Function Code, u8                , index 3
            FieldConf::String,       // Bytes, Vec<u8>                   , index 4
        ],
        FieldTerminator::new(),
    );
    for (step, (id, _, size, unit, code, bytes), target) in test_data {
        log::debug!("{dbg} | Bytes: {:?}", bytes);
        let t = Instant::now();
        let result = message.build(&[Field::U16(*id), Field::Const, Field::U16(*size), Field::Byte(*unit), Field::Byte(*code), Field::String(bytes.to_string())]);
        log::debug!("{dbg} | Elapsed: {:?}", t.elapsed());
        assert!(result == *target, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    }
    test_duration.exit();
}
