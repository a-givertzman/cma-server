use sal_sync::
    services::{conf::{ConfTree, ConfTreeGet},
    entity::{Name, PointConf},
    task::functions::{FnConfKeywd, FnConfKindName}}
;
use std::{fs, str::FromStr};

///
/// ## Config for `ModbusTcpBlock` format:
/// ```yaml
/// unit-00:                        # Modbus Unit 
///     unit: 00                    # Modbus Unit ID, used to identify a remote server located behaind the TCP/IP network (for serial bridging), ignored in a typical Modbus TCP/IP server
///     01:                         # Modbus function code, groups multiple registers
///         point Drive.Speed: 
///             type: 'Real'
///             offset: 0           # Modbus register addres
///     02:                         # Modbus function code, groups multiple registers
///         point Drive.Speed: 
///             type: 'Real'
///             offset: 0           # Modbus register addres
///```
#[derive(Debug, PartialEq, Clone)]
pub struct ModbusTcpBlockConf {
    pub name: Name,
    /// Modbus Unit ID, used to identify a remote server located behaind the TCP/IP network (for serial bridging), ignored in a typical Modbus TCP/IP server
    pub unit: u8,
    /// Defined registers and corresponding `Point`'s
    pub points: Vec<PointConf>,
}
//
// 
impl ModbusTcpBlockConf {
    ///
    /// Returns [ModbusTcpBlockConf] built from `ConfTree`:
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let me = conf.sufix_or(conf.name().unwrap());
        let dbg = format!("ModbusTcpBlockConf '{}'", me);
        log::trace!("{dbg}.new | conf: {:?}", conf);
        let name = Name::new(parent, me);
        log::trace!("{dbg}.new | name: {:?}", name);
        let unit: u64 = conf.get("unit").expect(&format!("{dbg}.new | 'unit' - not found or wrong config"));
        log::trace!("{dbg}.new | unit: {:?}", unit);
        let points = conf.nodes()
            .filter(|node| FnConfKeywd::from_str(&node.key).map_or(false, |keywd| keywd.kind() == FnConfKindName::Point))
            .map(|point| {
                PointConf::new(&name, &point)
            }).collect();
        Self {
            name,
            unit: unit as u8,
            points,
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub(crate) fn from_yaml(parent: impl Into<String>, value: &serde_yaml::Value) -> Self {
        match value.as_mapping().unwrap().into_iter().next() {
            Some((key, value)) => {
                Self::new(parent, ConfTree::new(key.as_str().unwrap(), value.clone()))
            }
            None => {
                panic!("ModbusTcpConf.from_yaml | Format error or empty conf: {:#?}", value)
            }
        }        
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    pub fn read(parent: impl Into<String>, path: &str) -> Self {
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        Self::from_yaml(parent, &config)
                    }
                    Err(err) => {
                        panic!("ModbusTcpConf.read | Error in config: {:?}\n\terror: {:?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("ModbusTcpConf.read | File {} reading error: {:?}", path, err)
            }
        }
    }
}
