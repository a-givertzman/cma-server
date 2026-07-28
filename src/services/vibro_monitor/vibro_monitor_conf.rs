use sal_sync::services::{conf::{ConfCustomKeywd, ConfTree, ConfTreeGet}, entity::Name};
use std::{fs, str::FromStr, time::Duration};
use crate::{infra::ApiClientConf, services::{SensorConf, UdpClientConf}};

/// Config for VibroMonitor format:
/// ```yaml
/// service VibroMonitor VibroMonitor-01:
///     wait-started: 10 ms         # optional, next service will wait until current completely started plus specified time
///     diagnosis:                          # internal diagnosis
///         point Status:                   # Ok(0) / Invalid(10)
///             type: 'Int'
///         point Connection:               # Ok(0) / Invalid(10)
///             type: 'Int'
///     api-client:
///         wait-started: 10 ms         # optional, next service will wait until current completely started plus specified time
///         address: "0.0.0.0:8081",
///         auth-token: "123!@#",
///         database: "cma",
///     tables:
///         faults: 'public.vibration_faults'
///         trends: 'public.vibration_trends'
///     sensor Motor-AC1:
///         target: Motor-AC1               # Уникальный идентификатор целевого механизма
///         connection:                     # Параметры связи с датчиком
///             reconnect: 1000 ms                      # reconnect timeout when connection is lost
///             protocol: 'udp-raw'                     # udp-raw
///             local-address: 192.168.100.100:15180    # Local machine address
///             remote-address: 192.168.100.241:15180   # IP Address of the vibro-sensor ADC unit
///             mtu: 1500                               # Maximum Transmission Unit, default 1500
///         adc:                            # Параметры сбора сырых данных с АЦП
///             sample-rate-hz: 320000
///             chunk-size: 512             # Размер выборки, поступающей из АЦП (сэмплов u16).
///         analysis:                       # Параметры цифровой обработки и виброаналитики
///             order-tracking:             
///                 max-order: 300              # Максимальный порядок (кратность частоты вращения), до которого производится спектральный анализ.
///                 order-resolution: 0.05      # Требуемая спектральное разрешение в угловом домене.
///                 # samples-per-rev: 256      # Плотность угловой дискретизации (сэмплов на оборот).
///             bands:
///                 low-order: 0.5..10.0        # Границы низкочастотной зоны в порядках (Orders)
///                 mid-hz: ..5000              # Верхняя граница среднего диапазона в Герцах (нижняя определяется порядками).
///                 high-hz: 5000..10000        # Границы высокочастотной зоны в Герцах.
///```
#[derive(Debug, Clone, PartialEq)]
pub struct VibroMonitorConf {
    pub name: Name,
    /// Next service will wait until current completely started plus specified time, optional
    pub wait_started: Option<Duration>,
    // pub cycle: Option<Duration>,
    // /// Service name, to subscribe for rope positin and crane angles event's
    // pub subscribe: String,
    /// API configuration parametes
    pub api: ApiClientConf,
    /// Names of the database table used for storing common settings for the clients
    pub tables: super::Tables,
    /// Параметры датчика виброаналитики цифровой обработки
    pub sensors: Vec<SensorConf>,
}
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
        // let subscribe = conf.get("subscribe").expect(&format!("{dbg}.new | 'subscribe' - not found or wrong config"));
        // log::trace!("{dbg}.new | subscribe: {:?}", subscribe);
        let api: ConfTree = conf.get("api-client").expect(&format!("{dbg}.new | 'api-client' - not found or wrong config"));
        let api = ApiClientConf::new(&name, api);
        log::trace!("{dbg}.new | api: {:#?}", api);
        let tables: ConfTree = conf.get("tables").expect(&format!("{dbg}.new | 'tables' - not found or wrong config"));
        let tables: super::Tables = serde_yaml::from_value(tables.conf).expect(&format!("{dbg}.new | 'tables' - wrong config"));
        log::trace!("{dbg}.new | tables: {:?}", tables);
        let sensors = conf.nodes().filter_map(|node| {
            match ConfCustomKeywd::from_str(&node.key) {
                Ok(keywd) => {
                    if keywd.name().to_lowercase() == "sensor" {
                        let name = keywd.title();
                        let Some(target) = node.get("target") else {
                            log::warn!("{dbg}.new | Sensor '{name}' | 'target' - not found");
                            return None;
                        };
                        let Some(connection) = ConfTreeGet::<ConfTree>::get(&node, "connection") else {
                            log::warn!("{dbg}.new | Sensor '{name}' | 'connection' - not found");
                            return None;
                        };
                        let connection = serde_yaml::from_value(connection.conf).expect(&format!("{dbg}.new | Sensor '{name}' | 'connection' - wrong config"));
                        let Some(adc) = ConfTreeGet::<ConfTree>::get(&node, "adc") else {
                            log::warn!("{dbg}.new | Sensor '{name}' | 'adc' - not found");
                            return None;
                        };
                        let adc = serde_yaml::from_value(adc.conf).expect(&format!("{dbg}.new | Sensor '{name}' | 'adc' - wrong config"));
                        let Some(analysis) = ConfTreeGet::<ConfTree>::get(&node, "analysis") else {
                            log::warn!("{dbg}.new | Sensor '{name}' | 'analysis' - not found");
                            return None;
                        };
                        let analysis = serde_yaml::from_value(analysis.conf).expect(&format!("{dbg}.new | Sensor '{name}' | 'analysis' - wrong config"));
                        let dsp = vibro_core::Conf { adc, analysis };
                        let sensor = SensorConf {
                            target,
                            connection,
                            dsp,
                        };
                        return Some(sensor)
                    }
                    None
                }
                Err(_) => None,
            }
        }).collect();
        log::trace!("{dbg}.new | sensors: {:#?}", sensors);
        Self {
            name,
            wait_started,
            // subscribe,
            api,
            tables,
            sensors,
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
            // subscribe: Default::default(),
            api: Default::default(),
            tables: super::Tables {
                faults: Default::default(),
                trends: Default::default(),
            },
            sensors: Default::default(),
        }
    }
}
