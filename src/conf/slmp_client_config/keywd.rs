use std::str::FromStr;
use log::trace;
use regex::RegexBuilder;
use serde::Deserialize;


///
/// 
#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone)]
pub enum Kind {
    Db,
}
//
// 
impl FromStr for Kind {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "db" => Ok(Kind::Db),
            _ => Err(format!("Kind.fron_str | Unknown keyword: '{}'", input))
        }
    }
}
impl ToString for Kind {
    fn to_string(&self) -> String {
        match self {
            Kind::Db => "db",
        }.to_string()
    }
}
///
/// 
#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct KeywdValue {
    pub prefix: String,
    pub kind: Kind,
    pub name: String,
}

///
/// keyword konsists of 3 fields:
/// ```
/// | prefix |  kind  |  name               |
/// |--------|--------|---------------------|
/// | opt    | requir |  requir             |
/// |--------|--------|---------------------|
/// |        | db     | db889               |
/// ````
#[derive(Debug, Deserialize, PartialEq, Eq, Hash)]
pub enum Keywd {
    Db(KeywdValue),
}
//
// 
impl Keywd {
    pub fn prefix(&self) -> String {
        match self {
            Keywd::Db(v) => v.prefix.clone(),
        }
    }
    pub fn kind(&self) -> Kind {
        match self {
            Keywd::Db(v) => v.kind.clone(),
        }
    }
    pub fn name(&self) -> String {
        match self {
            Keywd::Db(v) => v.name.clone(),
        }
    }
}

impl FromStr for Keywd {
    type Err = String;
    fn from_str(input: &str) -> Result<Keywd, String> {
        trace!("Keywd.from_str | input: {}", input);
        // let re = r#"(?:(?:(\w+)|))(?:(?:\s|)(device|db){1}(?:$|(?:[ \t]['"]*(\S+)['"]*)))"#;
        let re = r#"(?:(?:(\w+)|))(?:(?:\s|)(db){1}(?:$|(?:[ \t]['"]*(\S+)['"]*)))"#;
        let re = RegexBuilder::new(re).multi_line(false).build().unwrap();
        let group_prefix = 1;
        let group_kind = 2;
        let group_name = 3;
        match re.captures(input) {
            Some(caps) => {
                let prefix = match &caps.get(group_prefix) {
                    Some(first) => String::from(first.as_str()),
                    None => String::new(),
                };
                let kind = match &caps.get(group_kind) {
                    Some(kind) => {
                        match Kind::from_str(&kind.as_str().to_lowercase()) {
                            Ok(kinde) => kinde,
                            Err(_err) => {
                                panic!("Keywd.from_str | Error parsing kind of keyword '{}'", &input);
                                // Kind::Unknown
                            }
                        }
                    }
                    None => {
                        panic!("Keywd.from_str | Error parsing kind of keyword '{}'", &input);
                        // Kind::Unknown
                    }
                };
                let name = match &caps.get(group_name) {
                    Some(arg) => {
                        Ok(arg.as_str().to_string())
                    }
                    None => {
                        if input.is_empty() {                            
                            Err(format!("Error reading data of keyword '{}'", &input))
                        } else {
                            Ok(String::new())
                        }
                    }
                };
                match &name {
                    Ok(name) => {
                        match &caps.get(group_kind) {
                            Some(keyword) => {
                                match keyword.as_str() {
                                    "db" => Ok( Keywd::Db( KeywdValue { prefix, kind, name: name.to_string() } )),
                                    _      => Err(format!("Unknown keyword '{:?}'", &keyword)),
                                }
                            }
                            None => {
                                Err(format!("Unknown keyword '{}'", &input))
                            }
                        }
                    }
                    Err(err) => Err(err.to_string()),
                }
            }
            None => {
                Err(format!("Prefix Kinde Name - not found in keyword '{}'", &input))
            }
        }
    }
}
