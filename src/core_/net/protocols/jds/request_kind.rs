//!
//! JdsService implements behavior on the JDS communication protocol for the following kinds of requests:
//! Basic configuration parameters
//! service JdsService Id:
//!     parameter: value    # meaning
//!     parameter: value    # meaning
//! ```
use log::warn;
use regex::RegexBuilder;

///
/// Enum кузкуыутеы all supported Kinds of the request.
/// Also implements parsing from the string.
/// ```
/// "Auth.Secret" <-> RequestKind::AuthSecret
/// "Auth.Ssh" <-> RequestKind::AuthSsh
/// "Auth.Points" <-> RequestKind::Points
/// "Auth.Subscribe" <-> RequestKind::Subcribe
/// RequestKind::Unknown // - request string wasn't recognised
/// ```
///
#[derive(Debug, PartialEq)]
pub enum RequestKind {
    AuthSecret,
    AuthSsh,
    Points,
    Subscribe,
    Unknown,
}
//
// 
impl RequestKind {
    pub const AUTH_SECRET: &'static str = "Auth.Secret";
    pub const AUTH_SSH: &'static str = "Auth.Ssh";
    pub const POINTS: &'static str = "Points";
    pub const SUBSCRIBE: &'static str = "Subscribe";
}
//
// 
impl From<&str> for RequestKind {
    fn from(value: &str) -> Self {
        let re = r#"(?:/(?:[^/]+))*/(Auth\.Secret|Auth\.Ssh|Points|Subscribe)"#;
        let re = RegexBuilder::new(re).multi_line(false).build().unwrap();
        let group_kind = 1;
        match re.captures(value) {
            Some(caps) => {
                let kind = match &caps.get(group_kind) {
                    Some(first) => first.as_str(),
                    None => "",
                };
                match kind {
                    RequestKind::AUTH_SECRET => {
                        RequestKind::AuthSecret
                    }
                    RequestKind::AUTH_SSH => {
                        RequestKind::AuthSsh
                    }
                    RequestKind::POINTS => {
                        RequestKind::Points
                    }
                    RequestKind::SUBSCRIBE => {
                        RequestKind::Subscribe
                    }
                    _ => {
                        warn!("RequestKind.from<&str> | Unknown request: '{}'", value);
                        RequestKind::Unknown
                    }
                }
            }
            None => {
                warn!("RequestKind.from<&str> | Unknown request: '{}'", value);
                RequestKind::Unknown
            }
        }
    }
}
//
// 
impl From<String> for RequestKind {
    fn from(value: String) -> Self {
        RequestKind::from(value.as_str())
    }
}
//
// 
impl From<&String> for RequestKind {
    fn from(value: &String) -> Self {
        RequestKind::from(value.as_str())
    }
}
///
///
#[cfg(test)]
mod tests {
    use std::{sync::Once, time::Duration};
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};

    use crate::core_::net::protocols::jds::request_kind::RequestKind;
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
    #[test]
    fn test_task_cycle() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test RequestKind";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new("RequestKind test", Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = [
            (format!("{}/JdsService/Auth.Secret", self_id), RequestKind::AuthSecret),
            (format!("{}/JdsService/Auth.Ssh", self_id), RequestKind::AuthSsh),
            (format!("{}/JdsService/Points", self_id), RequestKind::Points),
            (format!("{}/JdsService/Subscribe", self_id), RequestKind::Subscribe),
        ];
        for (request, target) in test_data {
            let result = RequestKind::from(&request);
            assert!(result == target, "\nrequest: {:?} \nresult: {:?}\ntarget: {:?}", request, result, target);
        }
        test_duration.exit();
    }
}
