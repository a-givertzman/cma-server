use sal_sync::{collections::FxIndexMap, 
    services::{conf::{ConfCustomKeywd, ConfTree},
    entity::{Name, PointConf},
    task::functions::{FnConfKeywd, FnConfKindName}}}
;
use std::{fs, str::FromStr};
use crate::services::FunctionCode;

///
/// ## Config for `ModbusTcpBlock` format:
/// ```yaml
/// unit 00:                        # Modbus Unit 
///     function 01:                # Modbus function code, groups multiple registers
///         address: 00             # Modbus Unit ID, used to identify a remote server located behaind the TCP/IP network (for serial bridging), ignored in a typical Modbus TCP/IP server
///         size
///         point Drive.Speed: 
///             type: 'Real'
///             offset: 0           # Modbus register addres
///     function 02:                # Modbus function code, groups multiple registers
///         point Drive.Speed: 
///             type: 'Real'
///             offset: 0           # Modbus register addres
///```
#[derive(Debug, PartialEq, Clone)]
pub struct ModbusUnitConf {
    pub name: Name,
    /// Modbus Unit ID, used to identify a remote server located behaind the TCP/IP network (for serial bridging), ignored in a typical Modbus TCP/IP server
    pub unit: u8,
    /// Defined registers and corresponding `Point`'s
    pub functions: FxIndexMap<FunctionCode, Vec<PointConf>>,
}
//
// 
impl ModbusUnitConf {
    ///
    /// Returns [ModbusUnitConf] built from `ConfTree`:
    pub fn new(parent: impl Into<String>, unit: u8, conf: ConfTree) -> Self {
        let me = conf.sufix_or(conf.name().unwrap());
        let dbg = format!("ModbusUnitConf '{}'", me);
        log::trace!("{dbg}.new | conf: {:?}", conf);
        let name = Name::new(parent, me);
        log::trace!("{dbg}.new | name: {:?}", name);
        let functions = conf.nodes().filter_map(|node| {
            match ConfCustomKeywd::from_str(&node.key) {
                Ok(keywd) => match keywd.name().to_lowercase() == "function" {
                    true => {
                        let code = FunctionCode::from_str(&keywd.title()).expect(&format!("{dbg}.new | Can't parse Modbus 'Function Code' from {:?}", keywd));
                        let points = node.nodes()
                            .filter(|node| FnConfKeywd::from_str(&node.key).map_or(false, |keywd| keywd.kind() == FnConfKindName::Point))
                            .map(|point| {
                                PointConf::new(&name, &point)
                            }).collect();
                        Some((code, points))
                    }
                    false => None,
                }
                Err(_) => None
            }
        }).collect();
        Self {
            name,
            unit,
            functions,
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub(crate) fn from_yaml(parent: impl Into<String>, unit: u8, value: &serde_yaml::Value) -> Self {
        match value.as_mapping().unwrap().into_iter().next() {
            Some((key, value)) => {
                Self::new(parent, unit, ConfTree::new(key.as_str().unwrap(), value.clone()))
            }
            None => {
                panic!("ModbusTcpConf.from_yaml | Format error or empty conf: {:#?}", value)
            }
        }        
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    pub fn read(parent: impl Into<String>, unit: u8, path: &str) -> Self {
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        Self::from_yaml(parent, unit, &config)
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
