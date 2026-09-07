#[cfg(test)]

use sal_sync::services::entity::ToPoint;
use std::sync::Once;
use regex::RegexBuilder;
use debugging::session::{DebugSession, LogLevel};
use crate::domain::format::FormatPoint;
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
///
#[test]
fn simple_name() {
    DebugSession::new().filter(LogLevel::Info).init().unwrap();
    init_once();
    init_each();
    log::info!("test_bool");

    // let (initial, switches) = init_each();
    let test_data = vec![
        ("abc {a} xyz {b} rty {c} str {d}.", (false, 12, 1.618, "1223"), "abc false xyz 12 rty 1.618 str 1223."),
        ("abc {a} xyz '{b}' rty \"{c}\" str '{d}'.", (false, 12, 1.618, "1223"), "abc false xyz '12' rty \"1.618\" str '1223'."),
        ("abc {a} xyz '{b}' rty \"{c}\" str \"{d}\".", (false, 12, 1.618, "1223"), "abc false xyz '12' rty \"1.618\" str \"1223\"."),
    ];
    for (input, values, target) in test_data {
        let mut format = FormatPoint::new(input).unwrap();
        format.insert("a", values.0.to_point(0, ""));
        format.insert("b", values.1.to_point(0, ""));
        format.insert("c", values.2.to_point(0, ""));
        format.insert("d", values.3.to_point(0, ""));
        log::debug!("result: {}", format);
        assert!(format.out() == target, "format != target \nformat: {} \ntarget: {}", format.out(), target);
    }
}
///
/// Testing sufixt with supported `input.id`
#[test]
#[ignore = "sufixt `input.id` isn't supported yet"]
fn name_sufix_id() {
    DebugSession::new().filter(LogLevel::Info).init().unwrap();
    init_once();
    init_each();
    log::info!("test_name_sufix_id");
    let test_data = vec![
        ("abc {a.value} xyz {b.name} rty {c.timestamp} str {c.id}.", (false, 12, 1.618, "1223"), r"abc false xyz  rty {c.timestamp} UTC str {c.id}."),
        ("abc {a.value} xyz {b.name} rty {c.timestamp} str {c.id}.", (false, 02, 0.618, "1223"), r"abc false xyz  rty {c.timestamp} UTC str {c.id}."),
    ];
    for (input, values, target) in test_data {
        let mut format = FormatPoint::new(input).unwrap();
        format.insert("a.value", values.0.to_point(0, ""));
        format.insert("b.name", values.1.to_point(0, ""));
        format.insert("c.timestamp", values.2.to_point(0, ""));
        log::debug!("result: {}", format);
        let out = format.out();
        let target = target.replace(
            "{c.timestamp}",
            values.2.to_point(0, "").ts().to_rfc3339_opts(chrono::SecondsFormat::Secs, true).replace("T", " ").replace("Z", "").as_str(),
        );
        let re = format!(
            r"(abc false xyz  rty {})(\.\d{{9}})( UTC str \{{c\.id\}}\.)",
            values.2.to_point(0, "").ts().to_rfc3339_opts(chrono::SecondsFormat::Secs, true).replace("T", " ").replace("Z", ""),
        );
        log::trace!("re: {}", re);
        let re = RegexBuilder::new(&re).multi_line(false).build().unwrap();
        let out = re.replace(&out, "$1$3");
        log::trace!("out: {}", out);
        assert!(out == target, "format != target \nformat: {} \ntarget: {}", out, target);
    }
}
///
/// 
#[test]
fn name_sufix() {
    DebugSession::new().filter(LogLevel::Info).init().unwrap();
    init_once();
    init_each();
    log::info!("test_name_sufix");
    let test_data = vec![
        ("abc {a.value} xyz {b.name} rty {c.timestamp} str.", (false, 12, 1.618, "1223"), r"abc false xyz  rty {c.timestamp} str."),
        ("abc {a.value} xyz {b.name} rty {c.timestamp} str.", (false, 02, 0.618, "1223"), r"abc false xyz  rty {c.timestamp} str."),
    ];
    for (input, values, target) in test_data {
        let mut format = FormatPoint::new(input).unwrap();
        let c = values.2.to_point(0, "");
        format.insert("a.value", values.0.to_point(0, ""));
        format.insert("b.name", values.1.to_point(0, ""));
        format.insert("c.timestamp", c.clone());
        log::debug!("result: {}", format);
        let out = format.out();
        let target = target.replace(
            "{c.timestamp}",
            c.ts().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true).as_str(),
        );
        let re = format!(
            r"(abc false xyz  rty {})(\.\d{{9}})( UTC str \{{c\.id\}}\.)",
            c.ts().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
        );
        log::trace!("re: {}", re);
        let re = RegexBuilder::new(&re).multi_line(false).build().unwrap();
        let out = re.replace(&out, "$1$3");
        log::trace!("out: {}", out);
        assert!(out == target, "format != target \nformat: {} \ntarget: {}", out, target);
    }
}    #[test]
fn prepare() {
    DebugSession::new().filter(LogLevel::Info).init().unwrap();
    init_once();
    init_each();
    log::info!("test_prepare");
    // let (initial, switches) = init_each();

    let mut format = FormatPoint::new("abc {const} xyz '{b.name}' rty {c.value} str {c.timestamp}.").unwrap();
    format.insert("const", 12345.to_point(0, ""));
    format.insert("b.name", "".to_point(0, "the.name"));
    log::trace!("format: {}", format);
    let target = "abc 12345 xyz 'the.name' rty {c.value} str {c.timestamp}.";
    assert!(format.out() == target, "prepared format != target \nformat: {} \ntarget: {}", format, target);

    let test_data = vec![
        (1.618, r"abc 12345 xyz 'the.name' rty {c.value} str {c.timestamp}."),
        (0.618, r"abc 12345 xyz 'the.name' rty {c.value} str {c.timestamp}."),
    ];
    for (value, target) in test_data {
        let value = value.to_point(0, "");
        format.insert("a.value", value.clone());
        format.insert("c.timestamp", value.clone());
        log::debug!("result: {}", format);
        let out = format.out();
        let target = target.replace(
            "{c.timestamp}",
            value.ts().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true).as_str(),
        );
        let re = r"(.+)(\.\d+)( UTC)";
            // r"(abc false xyz  rty {})(\.\d{{9}})( UTC str \{{c\.id\}}\.)",
        // );
        // values.toPoint("").ts().to_rfc3339_opts(chrono::SecondsFormat::Secs, true).replace("T", " ").replace("Z", ""),
        log::trace!("re: {}", re);
        let re = RegexBuilder::new(&re).multi_line(false).build().unwrap();
        let out = re.replace(&out, "$1$3");
        log::trace!("out: {}", out);
        assert!(out == target, "format != target \nformat: {} \ntarget: {}", out, target);
    }
}
