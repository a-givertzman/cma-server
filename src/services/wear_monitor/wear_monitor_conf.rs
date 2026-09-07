use sal_sync::{collections::FxIndexMap, services::{conf::{ConfCustomKeywd, ConfTree, ConfTreeGet}, entity::Name}};
use std::{fs, str::FromStr, time::Duration};
use crate::{infra::ApiClientConf};

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
///         equipment: 'public.equipment'
///         faults: 'public.vibration_faults'
///         trends: 'public.vibration_trends'
///     sensor Motor-AC1:
///         # ---------------------------------------------------------------------
///         # Блок геометрии/кинематики узла
///         # ---------------------------------------------------------------------
///         steady-state:
///             motor-d: 0.050 m        # Диаметр вала двигателя [м].
///                                     # Значение по умолчанию: 0.050 (50 мм) — типично для валов
///                                     # мощностью ~5–15 кВт. Диапазон реальных значений: 0.020–0.150 м.
///                                     # Используется для пересчёта окружной скорости/момента,
///                                     # если нагрузка оценивается косвенно (через ток/момент двигателя).
///        
///         # ---------------------------------------------------------------------
///         # Паспортные и расчётные параметры подшипника (ISO 281)
///         # ---------------------------------------------------------------------
///         bearing:
///             cr: 45000.0             # Динамическая радиальная грузоподъёмность Cr [Н].
///                                     # Паспортное значение производителя, значения по умолчанию нет —
///                                     # обязательно брать из каталога. Для сравнения:
///                                     # SKF 21318 E: Cr = 393000 Н.
///                                     # Диапазон для узлов малой/средней мощности: 10 000–100 000 Н.
///          
///             c0r: 90000.0            # Статическая радиальная грузоподъёмность C0r [Н].
///                                     # В расчёте накопленного повреждения не используется,
///                                     # нужна только для проверки статической перегрузки
///                                     # (P_i / C0r не должно превышать ~0.5–1.0 в пике).
///                                     # По умолчанию не задаётся; ориентировочно C0r ≈ 1.7–2.0 · Cr.
///          
///             pu: 5200.0              # Предел усталостной нагрузки Pu [Н] (fatigue load limit).
///                                     # Нужен только для модифицированного (SKF) расчёта ресурса
///                                     # с учётом смазки/чистоты (aSKF); в базовой ISO 281 не используется.
///                                     # По умолчанию — не задаётся (0 = не используется).
///          
///             bearing-type: roller    # Тип подшипника: 
///                                     # - `roller` - роликоподшипник
///                                     # - `ball` - шарикоподшипник
///                                     # Поле необходимо для выбора показателя усталости кривой в формуле номинального ресурса подшипника `p`
///                                     # Значения по умолчанию:
///                                     #   3.0        — шариковые подшипники
///                                     #   10/3≈3.333 — роликовые подшипники (в т.ч. сферические, конические)
///                                     # Других значений на практике не встречается, поле — фиксированный выбор.
///          
///             x: 1.0                  # Коэффициент радиальной нагрузки X (в формуле P = X·Fr + Y·Fa).
///                                     # По умолчанию: 1.0 — учитывается только радиальная нагрузка,
///                                     # осевая составляющая отсутствует или пренебрежимо мала.
///                                     # Каталожные значения зависят от типа подшипника и отношения Fa/Fr
///                                     # (например, для SKF 21318E: X = 1 при Fa/Fr ≤ e, X = 0.67 при Fa/Fr > e).
///          
///             y: 0.0                  # Коэффициент осевой нагрузки Y (в формуле P = X·Fr + Y·Fa).
///                                     # По умолчанию: 0.0 — осевая нагрузка не учитывается
///                                     # (радиальный подшипник без осевой составляющей).
///                                     # Если осевая нагрузка есть, но не измеряется, можно задать
///                                     # оценочно через k_a (см. ниже) и каталожные Y1/Y2.
///          
///             k_a: 0.2               # Коэффициент оценки осевой нагрузки: Fa_i = k_a · Fr_i.
///             #                         # Инженерная оценка, НЕ паспортный параметр SKF.
///             #                         # По умолчанию: 0.0…0.2, типично 0.1 при отсутствии данных.
///             #
///             # e: 0.24                # Предельное отношение Fa/Fr для выбора формулы X/Y.
///             #                         # Паспортное значение, для SKF 21318E: e = 0.24.
///             #
///             # y1: 2.8                # Каталожный коэффициент Y для Fa/Fr ≤ e.
///             # y2: 4.2                # Каталожный коэффициент Y для Fa/Fr > e.
///        
///         # ---------------------------------------------------------------------
///         # Температурная модель (MVP-поправка, не строгая ISO-формула)
///         # ---------------------------------------------------------------------
///         temp-model:
///             t-ref: 40.0              # Опорная (номинальная) температура T_ref [°C].
///                                     # По умолчанию: 40 °C — типичная температура холостого хода
///                                     # исправного узла. Относительно неё считается ускорение износа.
///                                     # Диапазон: 30–60 °C в зависимости от типа смазки/условий эксплуатации.
///          
///             q10: 2.0                # Коэффициент ускорения износа Q10 (правило Вант-Гоффа) —
///                                     # во сколько раз возрастает "скорость повреждения" при
///                                     # повышении температуры на каждые +10 °C.
///                                     # По умолчанию: 1.5–2.0. Значение 2.0 — консервативная оценка
///                                     # (соответствует эмпирике для смазочных материалов и подшипниковых узлов).
///                                     # Это упрощённая MVP-поправка, а не паспортная величина ISO 281.
///          
///             t-min: -20.0             # Минимально допустимая температура эксплуатации [°C].
///                                     # По умолчанию: -20 °C (нижняя граница для стандартной смазки).
///                                     # При температуре ниже — сигнал об отказе датчика/аварийном режиме,
///                                     # расчёт повреждения не корректен (вязкость смазки не гарантирована).
///          
///             t-max: 120.0             # Максимально допустимая температура эксплуатации [°C].
///                                     # По умолчанию: 120 °C (типичный предел для стандартных
///                                     # уплотнений и консистентных смазок; для высокотемпературной
///                                     # смазки/термостойких подшипников может доходить до 150 °C).
///                                     # Превышение — повод для аварийной остановки, а не просто
///                                     # учёта в K_T.
///```
#[derive(Debug, Clone, PartialEq)]
pub struct WearMonitorConf {
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
impl WearMonitorConf {
    ///
    /// Returns [WearMonitorConf] built from `ConfTree`:
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let me = conf.sufix_or(conf.name().unwrap());
        let dbg = format!("WearMonitorConf '{}'", me);
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
    pub(crate) fn from_yaml(parent: impl Into<String>, value: &serde_yaml::Value) -> WearMonitorConf {
        match value.as_mapping().unwrap().into_iter().next() {
            Some((key, value)) => {
                Self::new(parent, ConfTree::new(key.as_str().unwrap(), value.clone()))
            }
            None => {
                panic!("WearMonitorConf.from_yaml | Format error or empty conf: {:#?}", value)
            }
        }        
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    pub fn read(parent: impl Into<String>, path: &str) -> WearMonitorConf {
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        WearMonitorConf::from_yaml(parent, &config)
                    }
                    Err(err) => {
                        panic!("WearMonitorConf.read | Error in config: {:?}\n\terror: {:?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("WearMonitorConf.read | File {} reading error: {:?}", path, err)
            }
        }
    }
}
//
//
impl Default for WearMonitorConf {
    fn default() -> Self {
        Self {
            name: Name::new("", "WearMonitorConf"),
            wait_started: Default::default(),
            subscribe: Default::default(),
            api: Default::default(),
            tables: super::Tables {
                equipment: Default::default(),
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

/// Варианты механизмов и их специфичные конфигурации
pub(super) enum EquipmentKind {
    /// Подшипник
    Bearing(wear_core::BearingWearConf),
    /// Редуктор
    Gearbox(wear_core::GearboxWearConf),
}