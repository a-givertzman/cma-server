use sal_sync::{collections::FxIndexMap, services::{ConfSubscribe, LinkName, conf::{ConfCustomKeywd, ConfDuration, ConfTree, ConfTreeGet}, entity::{Name, PointConf, PointType}, task::functions::{FnConfKeywd, FnConfKindName}}};
use std::{fs, str::FromStr, time::Duration};

use crate::{infra::ApiClientConf, services::{ResultKind, SqlResult}};

///
/// Config for `VirtualDevice` service:
/// ```yaml
/// service VirtualDevice MocIed12:
///     path: './test_ied12.ods'            # Optional, if signal have to be charged from the table
///     api:                                # Optional, if databese access for example required
///         address: 0.0.0.0:8080
///         auth-token: 123!@#
///         database: crane_data_server
///     inputs:                             # Input signal to be charged from the specified table file, or calculated in `Task`
///         point Winch.ValveEV1: 
///             type: Bool
///             history: rw
///         point Winch.ValveEV2: 
///             type: Bool
///             history: rw
///         point Winch.EncoderBR1: 
///             type: Int
///             comment: 'Скорость об/мин'
///     results:
///         point Result.Name1:             # the name of calculated result to be stored into the table column 'Result.Name1/result'
///             type: Int
///         sql Result.Name2:               # the name of result stored in the database, to be stored into the table column 'Result.Name2/result'
///             sql: 'select Name2 from table_name'
///             delay:  10ms                # Optional delay, to be awaited before select apears
///```
#[derive(Debug, Clone, PartialEq)]
pub struct VirtualDeviceConf {
    pub name: Name,
    /// Next service will wait until current completely started plus specified time, optional
    pub wait_started: Option<Duration>,
    /// Service name, to subscribe for rope positin and crane angles event's
    pub subscribe: ConfSubscribe,
    /// The service name to send all events
    pub send_to: LinkName,
    /// Optional, if signal have to be charged from the table
    pub path: Option<String>,
    /// Optional, if `path` specified, then work sheet name required
    pub sheet: Option<String>,
    /// API configuration parametes
    pub api: ApiClientConf,
    /// Names of the database table used for storing common settings for the clients
    pub inputs: FxIndexMap<String, PointConf>,
    pub results: FxIndexMap<String, ResultKind>,
}
//
// 
impl VirtualDeviceConf {
    ///
    /// Returns [VirtualDeviceConf] built from `ConfTree`:
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let me = conf.sufix_or(conf.name().unwrap());
        let dbg = format!("VirtualDeviceConf '{}'", me);
        log::trace!("{dbg}.new | conf: {:?}", conf);
        let name = Name::new(parent, me);
        log::trace!("{dbg}.new | name: {:?}", name);
        let wait_started: Option<Duration> = conf.get_duration("wait-started").ok();
        log::trace!("{}.new | wait-started: {:?}", dbg, wait_started);
        let subscribe: ConfTree = conf.get("subscribe").expect(&format!("{dbg}.new | 'subscribe' - not found or wrong config"));
        let subscribe = ConfSubscribe::new(subscribe.conf);
        log::trace!("{dbg}.new | subscribe: {:?}", subscribe);
        let send_to: String = conf.get("send-to").expect(&format!("{dbg}.new | 'send-to' - not found or wrong config"));
        let send_to = LinkName::from_str(&send_to).expect(&format!("{dbg}.new | 'send-to' - wrong config"));
        log::trace!("{dbg}.new | send-to: {:?}", send_to);
        let path: Option<String> = conf.get("path");
        log::trace!("{}.new | path: {:?}", dbg, path);
        let sheet: Option<String> = conf.get("sheet");
        log::trace!("{}.new | wait-started: {:?}", dbg, sheet);
        let api: ConfTree = conf.get("api").expect(&format!("{dbg}.new | 'api' - not found or wrong config"));
        let api = ApiClientConf::new(&name, api);
        log::trace!("{dbg}.new | api: {:#?}", api);
        let inputs: ConfTree = conf.get("inputs").expect(&format!("{dbg}.new | 'inputs' - not found or wrong config"));
        let inputs: FxIndexMap<String, PointConf> = inputs.nodes().filter_map(|node| {
            match FnConfKeywd::from_str(&node.key) {
                Ok(keyword) => match keyword.kind() {
                    FnConfKindName::Point => {
                        let point_name = format!("{name}/{}", keyword.data());
                        log::trace!("{}.new | Point '{}'", dbg, point_name);
                        log::trace!("{}.new | Point '{}'   |   conf: {:?}", dbg, point_name, node);
                        let node_conf = PointConf::new(&name, &node);
                        Some((point_name, node_conf))
                    }
                    _ => {
                        log::warn!("{}.new | Input Point expected, but found {:?}", dbg, keyword);
                        None
                    }
                }
                Err(err) => {
                    log::warn!("{}.new | Can't parse Input conf: {:?}, \n\terror: {:?}", dbg, node.key, err);
                    None
                }
            }
        }).collect();
        log::debug!("{dbg}.new | inputs: {:#?}", inputs.iter().map(|(n, p)| format!("{n}")).collect::<Vec<String>>());
        let results: ConfTree = conf.get("results").expect(&format!("{dbg}.new | 'results' - not found or wrong config"));
        let results: FxIndexMap<String, ResultKind> = results.nodes().filter_map(|node| {
            match FnConfKeywd::from_str(&node.key) {
                Ok(keyword) => match keyword.kind() {
                    FnConfKindName::Point => {
                        let point_name = format!("{name}/{}", keyword.data());
                        log::trace!("{}.new | Point '{}'", dbg, point_name);
                        log::trace!("{}.new | Point '{}'   |   conf: {:?}", dbg, point_name, node);
                        let node_conf = PointConf::new(&name, &node);
                        Some((point_name, ResultKind::Event(node_conf)))
                    }
                    _ => {
                        log::warn!("{}.new | Result Point expected, but found {:?}", dbg, keyword);
                        None
                    }
                }
                Err(_) => match ConfCustomKeywd::from_str(&node.key) {
                    Ok(keyword) => match keyword.name().as_str() {
                        "sql" => {
                            let point_name = format!("{name}/{}", keyword.title());
                            let sql: FxIndexMap<String, serde_yaml::Value> = serde_yaml::from_value(node.conf).unwrap();
                            let typ = sql.get("type").expect(&format!("Key 'type' missed in the '{point_name}'")).to_owned();
                            let typ: PointType = serde_yaml::from_value(typ).expect(&format!("Key 'type' - wrong config in the '{point_name}'"));
                            let sql = SqlResult {
                                name: point_name.clone(),
                                typ,
                                sql: sql.get("sql").expect(&format!("Key 'sql' missed in the '{point_name}'")).as_str().unwrap().to_owned(),
                                delay: ConfDuration::from_str(sql.get("delay").expect(&format!("Key 'delay' missed in the '{point_name}'")).as_str().unwrap()).unwrap(),
                            };
                            Some((point_name, ResultKind::Sql(sql)))
                        }
                        _ => {
                            log::warn!("{}.new | Result SQL expected, but found {:?}", dbg, keyword);
                            None
                        }
                    }
                    Err(err) => {
                        log::warn!("{}.new | Can't parse Result Point/SQL conf: {:?}", dbg, node.key);
                        None
                    }
                }
            }
        }).collect();
        log::debug!("{dbg}.new | results: {:#?}", results.iter().map(|(n, p)| format!("{n}: {:?}", p)).collect::<Vec<String>>());
        Self {
            name,
            wait_started,
            subscribe,
            send_to,
            path,
            sheet,
            api,
            inputs,
            results,
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
                panic!("VirtualDeviceConf.from_yaml | Format error or empty conf: {:#?}", value)
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
                        VirtualDeviceConf::from_yaml(parent, &config)
                    }
                    Err(err) => {
                        panic!("VirtualDeviceConf.read | Error in config: {:?}\n\terror: {:?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("VirtualDeviceConf.read | File {} reading error: {:?}", path, err)
            }
        }
    }
}
//
//
impl Default for VirtualDeviceConf {
    fn default() -> Self {
        Self {
            name: Name::new("", "VirtualDeviceConf"),
            wait_started: Default::default(),
            subscribe: ConfSubscribe::default(),
            send_to: Default::default(),
            path: Default::default(),
            sheet: Default::default(),
            api: Default::default(),
            inputs: Default::default(),
            results: Default::default(),
        }
    }
}
