use std::{hash::{DefaultHasher, Hash, Hasher}, str::FromStr};
use log::{trace, warn};
use regex::RegexBuilder;
use crate::core_::status::status::Status;
///
/// Optional parameters of the [FnConf]
#[derive(Debug, PartialEq, Clone)]
pub struct FnConfOptions {
    pub default: Option<String>,
    pub status: Option<Status>,
}
//
//
impl FnConfOptions {
    ///
    /// Returns 'Options hash' to identify unique set of options
    pub fn hash(&self) -> String {
        format!("default:{}-status:{}", Self::hash_(&self.default), Self::hash_(&self.status))
    }
    ///
    /// Returns hash for T
    fn hash_<T: Hash>(value: &T) -> u64 {
        let mut state = DefaultHasher::new();
        value.hash(&mut state);
        state.finish()
    }    
}
//
//
impl Default for FnConfOptions {
    fn default() -> Self {
        Self { default: None, status: None }
    }
}
//
//
impl FromStr for FnConfOptions {
    type Err = String;
    fn from_str(input: &str) -> Result<FnConfOptions, String> {
        trace!("FnConfOptions.from_str | input: {}", input);
        let re_default = r#"[ \t]?default[ \t](\S+)"#;
        let re_default = RegexBuilder::new(re_default).multi_line(false).build().unwrap();
        let re_status = r#"[ \t]?status[ \t](\S+)"#;
        let re_status = RegexBuilder::new(re_status).multi_line(false).build().unwrap();
        let default = re_default.captures(input).map_or(None, |caps| {
            caps.get(1).map(|value| value.as_str().to_owned())
        });
        let status = match re_status.captures(input) {
            Some(caps) => caps.get(1).map_or(None, |value| {
                match Status::from_str(value.as_str()) {
                    Ok(status) => Some(status),
                    Err(err) => {
                        warn!("FnConfOptions.from_str | Status parsing error: {}", err);
                        None
                    }
                }
            }),
            None => None,
        };
        Ok(Self {
            default,
            status,
        })
    }
}
