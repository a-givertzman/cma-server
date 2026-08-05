use sal_sync::{collections::FxIndexMap, services::{conf::{ConfCustomKeywd, ConfTree, ConfTreeGet}, entity::Name}};
use std::{fs, str::FromStr, time::Duration};
use crate::{infra::ApiClientConf};
use super::SensorConf;

/// Config for VibroMonitor format:
/// ```yaml
/// service VibroMonitor VibroMonitor-01:
///     wait-started: 10 ms                 # optional, next service will wait until current completely started plus specified time
///     subscribe: /App/MultiQueue          # Subscriptions will be to the MultiQueue
///     diagnosis:                          # internal diagnosis
///         point Status:                   # Ok(0) / Invalid(10)
///             type: 'Int'
///         point Connection:               # Ok(0) / Invalid(10)
///             type: 'Int'
///     api-client:
///         wait-started: 10 ms         # optional, next service will wait until current completely started plus specified time
///         address: "0.0.0.0:8081"
///         auth-token: "123!@#"
///         database: "cma"
///     tables:
///         faults: 'public.vibration_faults'
///         trends: 'public.vibration_trends'
///     sensor Motor-AC1:
///         target: Motor-AC1               # Уникальный идентификатор целевого механизма
///         rpm: point real '/App/Ied01/Motor.AC1.RPM'  # Текущая скорость вращения вала механизма, об/мин
///         channel: 1                      # Номер канала в АЦП (0..255). 0 - первый канал.
///         connection:                     # Параметры связи с датчиком
///             reconnect: 1000 ms                      # reconnect timeout when connection is lost
///             protocol: 'udp-raw'                     # udp-raw
///             local-addr: 192.168.100.100:15180    # Local machine address
///             remote-addr: 192.168.100.241:15180   # IP Address of the vibro-sensor ADC unit
///             mtu: 1500                               # Maximum Transmission Unit, default 1500
///         adc:                            # Параметры сбора сырых данных с АЦП
///             ds-offset: 2048             # Постоянная составляющая сигнала. Будет вычитаться из сырых сэмплов.
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
    /// Name of the service for subscribing to RPM event's
    pub subscribe: String,
    /// API configuration parametes
    pub api: ApiClientConf,
    /// Names of the database table used for storing common settings for the clients
    pub tables: super::Tables,
    /// Параметры датчиков виброаналитики цифровой обработки
    pub sensors: FxIndexMap<AdcIp, Vec<SensorConf>>,
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
        let subscribe = conf.get("subscribe").expect(&format!("{dbg}.new | 'subscribe' - not found or wrong config"));
        log::trace!("{dbg}.new | subscribe: {:?}", subscribe);
        let api: ConfTree = conf.get("api-client").expect(&format!("{dbg}.new | 'api-client' - not found or wrong config"));
        let api = ApiClientConf::new(&name, api);
        log::trace!("{dbg}.new | api: {:#?}", api);
        let tables: ConfTree = conf.get("tables").expect(&format!("{dbg}.new | 'tables' - not found or wrong config"));
        let tables: super::Tables = serde_yaml::from_value(tables.conf).expect(&format!("{dbg}.new | 'tables' - wrong config"));
        log::trace!("{dbg}.new | tables: {:?}", tables);
        let mut sensors: FxIndexMap<AdcIp, Vec<SensorConf>> = FxIndexMap::default();
        for node in conf.nodes() {
            if let Ok(keywd) = ConfCustomKeywd::from_str(&node.key) {
                if keywd.name().to_lowercase() == "sensor" {
                    let name = keywd.title();
                    let Some(target) = node.get("target") else {
                        log::warn!("{dbg}.new | Sensor '{name}' | 'target' - not found");
                        continue;
                    };
                    let Some(rpm) = ConfTreeGet::<f64>::get(&node, "rpm").map(InputKind::Const)
                        .or_else(|| ConfTreeGet::<i64>::get(&node, "rpm").map(|rpm| InputKind::Const(rpm as f64)))
                        .or_else(|| node.get_fn_config(&dbg, "rpm", &mut vec![]).map(|rpm| InputKind::Point(rpm.name())))
                    else {
                        log::warn!("{dbg}.new | Sensor '{name}' | 'rpm' - not found or wrong format, point or f64 expected");
                        continue;
                    };
                    let Some(channel): Option<u64> = node.get("channel") else {
                        log::warn!("{dbg}.new | Sensor '{name}' | 'channel' - not found");
                        continue;
                    };
                    let Some(connection): Option<ConfTree> = node.get("connection") else {
                        log::warn!("{dbg}.new | Sensor '{name}' | 'connection' - not found");
                        continue;
                    };
                    let connection: super::UdpClientConf = serde_yaml::from_value(connection.conf).expect(&format!("{dbg}.new | Sensor '{name}' | 'connection' - wrong config"));
                    let Some(adc): Option<ConfTree> = node.get("adc") else {
                        log::warn!("{dbg}.new | Sensor '{name}' | 'adc' - not found");
                        continue;
                    };
                    let adc = serde_yaml::from_value(adc.conf).expect(&format!("{dbg}.new | Sensor '{name}' | 'adc' - wrong config"));
                    let Some(analysis): Option<ConfTree> = node.get("analysis") else {
                        log::warn!("{dbg}.new | Sensor '{name}' | 'analysis' - not found");
                        continue;
                    };
                    let analysis = serde_yaml::from_value(analysis.conf).expect(&format!("{dbg}.new | Sensor '{name}' | 'analysis' - wrong config"));
                    let dsp = vibro_core::Conf { adc, analysis };
                    let adc_ip = AdcIp(connection.remote_addr.clone());
                    let sensor = SensorConf {
                        target,
                        rpm,
                        channel: channel as usize,
                        connection,
                        dsp,
                    };
                    sensors.entry(adc_ip).or_default().push(sensor);
                }
            }
        }
        log::trace!("{dbg}.new | sensors: {:#?}", sensors);
        Self {
            name,
            wait_started,
            subscribe,
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
            subscribe: Default::default(),
            api: Default::default(),
            tables: super::Tables {
                faults: Default::default(),
                trends: Default::default(),
            },
            sensors: Default::default(),
        }
    }
}
/// ### Уникальный идентификатор контроллера АЦП (IP адрес)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AdcIp(pub String);

///
/// Variants of the service input
/// - Const: Value
/// - Point: point real 'App/MultiQueue/Load.MainBoomAngle'
#[derive(Debug, Clone, PartialEq)]
pub(super) enum InputKind<T> {
    Const(T),
    Point(String),
}
impl<T: vibro_core::Zero> Default for InputKind<T> {
    fn default() -> Self {
        Self::Const(T::zero())
    }
}