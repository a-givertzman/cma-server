use sal_sync::services::{conf::{ConfTree, ConfTreeGet}, entity::Name};
use std::{fs, time::Duration};
use crate::{infra::ApiClientConf};

///
/// Config for VibroMonitor format:
/// ```yaml
/// service VibroMonitor VibroMonitor-01:
///     wait-started: 10 ms         # optional, next service will wait until current completely started plus specified time
///     subscribe: MultiQueue       # Service name, to subscribe for event's required for the calculations like rope positin and crane angles
///     api-client:
///         wait-started: 10 ms         # optional, next service will wait until current completely started plus specified time
///         address: "0.0.0.0:8081",
///         auth-token: "123!@#",
///         database: "cma",
///     tables:
///         faults: 'public.vibration_faults'
///         trends: 'public.vibration_trends'
///     sensor Motor-AC1:
///         udp:
///             reconnect: 1000 ms                      # reconnect timeout when connection is lost
///             protocol: 'udp-raw'                     # udp-raw
///             local-address: 192.168.100.100:15180    # Local machine address
///             remote-address: 192.168.100.241:15180   # IP Address of the vibro-sensor ADC unit
///             mtu: 1500                               # Maximum Transmission Unit, default 1500
///         hardware:
///             sample-rate-hz: 320000
///             chunk-size: 512             # Размер выборки, поступающей из АЦП (сэмплов u16).
///         angular:
///             max-order: 300              # Максимальный порядок (кратность частоты вращения), до которого производится спектральный анализ.
///             order-resolution: 0.05      # Требуемая спектральное разрешение в угловом домене.
///             # samples-per-rev: 256      # Плотность угловой дискретизации (сэмплов на оборот).
///         bands:
///             low-order: 0.5..10.0        # Границы низкочастотной зоны в порядках (Orders)
///             mid-hz: ..5000              # Верхняя граница среднего диапазона в Герцах (нижняя определяется порядками).
///             high-hz: 5000..10000        # Границы высокочастотной зоны в Герцах.
///```
#[derive(Debug, Clone, PartialEq)]
pub struct VibroMonitorConf {
    pub name: Name,
    /// Next service will wait until current completely started plus specified time, optional
    pub wait_started: Option<Duration>,
    // pub cycle: Option<Duration>,
    /// Service name, to subscribe for rope positin and crane angles event's
    pub subscribe: String,
    /// API configuration parametes
    pub api: ApiClientConf,
    /// Names of the database table used for storing common settings for the clients
    pub tables: Tables,
}
//
// 
impl VibroMonitorConf {
    ///
    /// Returns [VibroMonitorConf] built from `ConfTree`:
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let me = conf.sufix_or(conf.name().unwrap());
        let dbg = format!("VibroMonitorConf '{}'", me);
        log::trace!("{dbg}.new | conf: {:?}", conf);
        let name = Name::new(parent, me);
        log::trace!("{dbg}.new | name: {:?}", name);
        let wait_started: Option<Duration> = conf.get_duration("wait-started").ok();
        log::trace!("{}.new | wait-started: {:?}", dbg, wait_started);
        let subscribe = conf.get("subscribe").expect(&format!("{dbg}.new | 'subscribe' - not found or wrong config"));
        log::trace!("{dbg}.new | subscribe: {:?}", subscribe);
        let api: ConfTree = conf.get("api-client").expect(&format!("{dbg}.new | 'api-client' - not found or wrong config"));
        let api = ApiClientConf::new(&name, api);
        log::trace!("{dbg}.new | api: {:#?}", api);
        let table_settings = conf.get("table-settings").expect(&format!("{dbg}.new | 'table-settings' - not found or wrong config"));
        log::trace!("{dbg}.new | table-settings: {:?}", table_settings);
        let rope_defect: ConfTree = conf.get("rope-defect").expect(&format!("{dbg}.new | 'rope-defect' - not found or wrong config"));
        let rope_defect = RopeDefectConf::new(&name, rope_defect, api.clone());
        log::trace!("{dbg}.new | rope-defect: {:#?}", rope_defect);
        let rope_deprecation: ConfTree = conf.get("rope-deprecation").expect(&format!("{dbg}.new | 'rope-deprecation' - not found or wrong config"));
        let rope_deprecation = RopeDeprecationConf::new(&name, rope_deprecation, api.clone());
        log::trace!("{dbg}.new | rope-deprecation: {:#?}", rope_deprecation);
        Self {
            name,
            wait_started,
            subscribe,
            api,
            table_settings,
            rope_defect,
            rope_deprecation,
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub(crate) fn from_yaml(parent: impl Into<String>, value: &serde_yaml::Value) -> VibroMonitorConf {
        match value.as_mapping().unwrap().into_iter().next() {
            Some((key, value)) => {
                Self::new(parent, ConfTree::new(key.as_str().unwrap(), value.clone()))
            }
            None => {
                panic!("VibroMonitorConf.from_yaml | Format error or empty conf: {:#?}", value)
            }
        }        
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    pub fn read(parent: impl Into<String>, path: &str) -> VibroMonitorConf {
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        VibroMonitorConf::from_yaml(parent, &config)
                    }
                    Err(err) => {
                        panic!("VibroMonitorConf.read | Error in config: {:?}\n\terror: {:?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("VibroMonitorConf.read | File {} reading error: {:?}", path, err)
            }
        }
    }
}
//
//
impl Default for VibroMonitorConf {
    fn default() -> Self {
        Self {
            name: Name::new("", "VibroMonitorConf"),
            wait_started: Default::default(),
            subscribe: Default::default(),
            api: Default::default(),
            table_settings: Default::default(),
            rope_defect: Default::default(),
            rope_deprecation: Default::default(),
        }
    }
}
