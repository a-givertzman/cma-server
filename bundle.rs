mod vibro_monitor {
use std::{sync::Arc, time::Duration};
use dashmap::DashMap;
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{kernel::state::ExitNotify, services::{EventValueAccess, Service, Services, entity::{Name, Object}}, thread_pool::Scheduler};
use crate::{domain::{RECV_TIMEOUT, RecvTimeoutError}, err, err_pass, infra::ApiClient};
use super::{VibroMonitorConf, InputKind};
pub struct VibroMonitor {
    name: Name,
    conf: VibroMonitorConf,
    services: Arc<Services>,
    scheduler: Scheduler,
    tasks: Arc<DashMap<String, Arc<dyn Service>>>,
    exit: Arc<ExitNotify>,
    dbg: Dbg,
}
impl VibroMonitor {
    pub fn new(conf: VibroMonitorConf, services: Arc<Services>, scheduler: Scheduler) -> Self {
        let dbg = Dbg::new(conf.name.parent(), conf.name.me());
        Self {
            name: conf.name.clone(),
            conf,
            services,
            scheduler,
            tasks: Arc::new(DashMap::new()),
            exit: Arc::new(ExitNotify::new(&dbg, None, None)),
            dbg,
        }
    }
    #[named]
    fn configure_database(&self, api_client: &Arc<ApiClient>, exit: &Arc<ExitNotify>) -> Result<(), Error> {
        let fetch_timeout = Duration::from_secs(10);
        let vibration_trends = &self.conf.tables.trends;
        let vibration_faults = &self.conf.tables.faults;
        let sql = format!(r#"
do $$
begin
-- 1. Справочник оборудования
CREATE TABLE IF NOT EXISTS equipment (
    id          integer GENERATED ALWAYS AS IDENTITY,
    name        VARCHAR(255) NOT NULL,       -- Наименование (например, 'Насос НП-101')
    model       VARCHAR(100),                -- Модель/Тип агрегата
    created_at  TIMESTAMPTZ DEFAULT clock_timestamp() NOT NULL,
    CONSTRAINT pk_equipment PRIMARY KEY (id),
    CONSTRAINT uq_equipment_name UNIQUE (name)
);
-- 2. Таблица учета наработки и ресурса (Wear & Lifespan)
CREATE TABLE equipment_wear (
    equipment_id       INTEGER PRIMARY KEY REFERENCES equipment(id) ON DELETE CASCADE,
    operating_hours    REAL NOT NULL DEFAULT 0.0,  -- Фактическая наработка (моточасы)
    nominal_resource   REAL NOT NULL,              -- Номинальный ресурс до кап. ремонта (моточасы)
    updated_at         TIMESTAMPTZ NOT NULL        -- Время последнего обновления наработки
);
-- 3. Обновленная таблица трендов вибрации
CREATE TABLE {vibration_trends} (
    timestamp      TIMESTAMPTZ NOT NULL,
    equipment_id   INTEGER NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    order_id       VARCHAR(10) NOT NULL,       -- '1x', '2x', '3x', '0.5x' и т.д.
    rms_value      REAL NOT NULL,              -- Амплитуда (RMS)
    phase          REAL,                       -- Фаза в градусах [0..360)
    rpm            REAL NOT NULL,              -- Текущие обороты вала
    PRIMARY KEY (timestamp, equipment_id, order_id)
);
-- Индекс для быстрой фильтрации по конкретному агрегату и гармонике
CREATE INDEX idx_equip_order_trends ON {vibration_trends} (equipment_id, order_id, timestamp DESC);
-- Справочник видов дефектов
CREATE TABLE fault_kind (
    id          VARCHAR(64) NOT NULL,
    description TEXT NOT NULL,
    PRIMARY KEY (id)
)
INSERT INTO fault_kind (id, description) VALUES
    ('Imbalance', 'Дисбаланс'),
    ('Misalignment', 'Расцентровка'),
    ('MechanicalLooseness', 'Механический люфт');
-- Степень развития дефекта (Зоны ISO 10816 / 20816)
CREATE TYPE vibration_severity AS ENUM (
    'green',   -- Отличное или новое состояние.
    'yellow',  -- Пригодно для длительной эксплуатации без ограничений.
    'orange',  -- Предупреждение (Warn). Пригодно для ограниченной эксплуатации, требуется планирование ремонта.
    'red'      -- Преждевременный отказ (Alarm). Опасные вибрации, требуется немедленная остановка.
);
COMMENT ON TYPE vibration_severity VALUE 'green' IS 'Отличное или новое состояние.';
COMMENT ON TYPE vibration_severity VALUE 'yellow' IS 'Пригодно для длительной эксплуатации без ограничений.';
COMMENT ON TYPE vibration_severity VALUE 'orange' IS 'Предупреждение (Warn). Пригодно для ограниченной эксплуатации, требуется планирование ремонта.';
COMMENT ON TYPE vibration_severity VALUE 'red' IS 'Преждевременный отказ (Alarm). Опасные вибрации, требуется немедленная остановка.';
-- Оценка состояния оборудования на основании вибрации
CREATE TABLE {vibration_faults} (
    timestamp      TIMESTAMPTZ NOT NULL,
    equipment_id   INTEGER NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    -- Вид неисправности
    fault_kind     VARCHAR(64) NOT NULL REFERENCES fault_kind(id),
    -- Метрика сходства с патерном дефекта [0.0, 1.0]
    score          DOUBLE PRECISION NOT NULL,
    -- Степень опасности текущего дефекта
    -- Green, Yellow, Orange, Red
    severity       vibration_severity NOT NULL,
    -- Текущие обороты расчете, для валидации диагноза
    rpm            DOUBLE PRECISION NOT NULL
    -- Гарантирует, что для каждого оборудования
    -- хранится ровно ОДНА запись по конкретному дефекту
    PRIMARY KEY (equipment_id, fault_kind)
);
-- SQL-запрос для вывода списка оборудования с худшим статусом
SELECT
    e.id AS equipment_id,
    e.name AS equipment_name,
    -- MAX() для ENUM в Postgres выберет самое критическое состояние (последнее в списке ENUM)
    MAX(vf.severity) AS overall_severity,
    -- Собираем список всех обнаруженных дефектов, которые вышли из зоны 'green'
    STRING_AGG(
        CASE WHEN vf.severity != 'green' THEN fk.description END,
        ', '
    ) AS active_faults,
    MAX(vf.timestamp) AS last_update
FROM equipment e
LEFT JOIN vibration_faults vf ON e.id = vf.equipment_id
LEFT JOIN fault_kind fk ON vf.fault_kind = fk.id
GROUP BY e.id, e.name
ORDER BY
    -- Сначала показываем самое "красное" и "оранжевое" оборудование
    overall_severity DESC NULLS LAST,
    e.name;
end; $$
language plpgsql;
        "#);
        match api_client.fetch(sql).timeout(fetch_timeout) {
            Ok(Some(Ok(_))) => Ok(()),
            Ok(Some(Err(err))) => Err(err_pass!(self.dbg, err)),
            Ok(None) => Err(err!(self.dbg, "Timeout {:?}", fetch_timeout)),
            Err(err) => Err(err_pass!(self.dbg, err)),
        }
    }
}
impl Object for VibroMonitor {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
impl std::fmt::Debug for VibroMonitor {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("VibroMonitor")
            .field("dbg", &self.dbg)
            .finish()
    }
}
//
impl Service for VibroMonitor {
    //
    #[named]
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let name = self.name.clone();
        let conf = self.conf.clone();
        let services = self.services.clone();
        let scheduler = self.scheduler.clone();
        let fetch_timeout = Duration::from_secs(3);
        let api_client = Arc::new(ApiClient::new(conf.api.clone(), scheduler.clone()));
        self.tasks.insert(api_client.name().join(), api_client.clone());
        api_client.run().map_err(|err| err_pass!(self.dbg, err))?;
        log::info!("{}.run | ApiClient ready", self.dbg);
        // Конфигурация БД. TODO: Нужно довести SQL что бы он при повторном запуске адекватно срабатывал
        // self.configure_database(&api_client, &self.exit).map_err(|err| err_pass!(self.dbg, err))?;
        let retain = Arc::new(vibro_core::Retain::mock(&self.dbg, []));
        let mut event_values = crate::services::EventValues::new(&name, &conf.subscribe, &services, &scheduler, &self.exit);
        // Регистрация настроенных входных сигналов (RPM) для последующей подписки на них в сервисе conf.subscribe.
        // Выполняется до запуска сервиса!
        for (_adc_id, sensors_conf) in &conf.sensors {
            for sensor in sensors_conf {
                if let InputKind::Point(rpm) = &sensor.rpm {
                    event_values.register(rpm);
                }
            }
        }
        let event_values = Arc::new(event_values);
        self.tasks.insert(event_values.name().join(), event_values.clone());
        let (api_link, api_queue) = crate::domain::bounded(4096);
        // TODO: Может вынести в отдельный сервис
        scheduler.spawn({
            let dbg = self.dbg.clone();
            let exit = self.exit.clone();
            move || {
            while !exit.get() {
                match api_queue.recv_timeout(RECV_TIMEOUT) {
                    Ok(sql) => {
                        match api_client.fetch(&sql).timeout(fetch_timeout) {
                            Ok(Some(Err(err))) => log::warn!("{dbg}.run | Error on sql '{sql}': \n{:?}", err),
                            Ok(None) => log::warn!("{dbg}.run | Timeout ({:?}) on sql '{sql}'", fetch_timeout),
                            Err(err) => log::warn!("{dbg}.run | Error on sql '{sql}': \n{:?}", err),
                            _ => {}
                        }
                    }
                    Err(RecvTimeoutError::Timeout) => {}
                    _ => break,
                }
            }
        }}).map_err(|err| err_pass!(self.dbg, err))?;
        // Настройка диагностики для датчиков, датчики сгруппированы по IP адресам
        // Передаем группу датчиков с одним IP в один модуль
        for (_adc_id, sensors_conf) in &conf.sensors {
            let sensor = Arc::new(super::VibroAdc::new(
                &self.dbg,
                sensors_conf.clone(),
                event_values.clone(),
                retain.clone(),
                api_link.clone(),
                &conf.tables.equipment,
                &conf.tables.trends,
                &conf.tables.faults,
                scheduler.clone(),
                self.exit.clone(),
            ));
            self.tasks.insert(sensor.name().join(), sensor.clone());
            sensor.run().map_err(|err| err_pass!(self.dbg, err))?;
        }
        event_values.run().map_err(|err| err_pass!(self.dbg, err))?;      // have to be started after all subscription being added, then it will subscribe all them on MultiQueue
        log::info!("{}.run | RopeDefect's ready", self.dbg);
        log::info!("{}.run | Starting - Ok", self.dbg);
        Ok(())
    }
    //
    fn wait(&self) -> Result<(), Error> {
        let mut errors = vec![];
        for task in self.tasks.iter() {
            if let Err(err) = task.value().wait() {
                errors.push(err);
            }
        }
        errors
            .is_empty()
            .then(|| {
                log::info!("{}.run | Exit", self.dbg);
                ()
            })
            .ok_or(
                Error::new(&self.dbg, "wait").pass(errors.iter().fold(String::new(), |acc, err| format!("{}\n{}", acc, err)))
            )
    }
    //
    fn is_finished(&self) -> bool {
        let mut is_finished = false;
        for task in self.tasks.iter() {
            is_finished = is_finished & task.value().is_finished();
        }
        is_finished
    }
    //
    fn exit(&self) {
        self.exit.exit();
        for task in self.tasks.iter() {
            task.value().exit();
        }
    }
}
}
pub use vibro_monitor::*;
mod vibro_monitor_conf {
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
///         equipment: 'public.equipment'
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
}}
pub use vibro_monitor_conf::*;
mod tables {
use serde::Deserialize;
/// Tables used for storing diagnostic risults into database
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Tables {
    /// Таблица | Справочник оборудования.
    pub equipment: String,
    /// Таблица | Тренды вибрации.
    pub trends: String,
    /// Таблица | Состояния оборудования на основании вибрации.
    pub faults: String,
}
}
pub(crate) use tables::*;
mod sensor_conf {
use super::UdpClientConf;
/// Параметры датчика виброаналитики цифровой обработки
#[derive(Debug, Clone, PartialEq)]
pub struct SensorConf {
    /// Уникальный идентификатор целевого механизма.
    pub target: String,
    /// Номер канала в АЦП (0..255). 0 - первый канал.
    pub channel: usize,
    /// Сигнал скорости вращения вала механизма
    pub rpm: super::InputKind<f64>,
    /// Параметры связи с датчиком.
    pub connection: UdpClientConf,
    /// Параметры сбора данных с АЦП и цифровой обработки и виброаналитики.
    pub dsp: vibro_core::Conf,
}
}
pub(crate) use sensor_conf::*;
mod udp_client {
//!
//! # Implements communication with Vibro-analytics microcontroller (Sub MC) over udp simple protocol.
//!
//! - Read Data
//!     - Cyclically reads data bytes from the device (STM32 Micro-controller)
//!     - Data bytes contains an array of the samples receaved from the ADC, corresponds to the sample frequence
//!     - Converts data bytes into the amplitudes of the source input signal, where each value coresponds to the exact time
//!     - Sends each amplitude value as `Event` to the specified destination service.
//! - Send Commands
//!     - Writes received command `Event` to the device.
//!
//! ## 1. General
//!
//! - **Functional bytes**
//!
//!     `SYN` = 0x22 - Start
//!     `EOT` =  0x04
//!
//!     `STX` = 0x02 - Data message
//!     `CMD` = 0x05 - Command message
//!     `ERR` = 0x07 - Error message
//!
//! - **Members**
//!
//!     `Client` - Backend application
//!     `Server` - Device (micro-controller)
//!
//! - **Message structure**
//!
//!     |Field name:   | FUN | ADDR | TYPE | COUNT | DATA        |
//!     |---           | --- | ---- | ---- | ----- | ----        |
//!     |Data type:    | u8  | u8   | u8   | u32   | [T; COUNT]  |
//!     |Example value:| 22  | 0    | 16   | 512   | [u16; 512]  |
//!
//!     - `FUN` Functional byte,
//!         - `0x22` - Initialization message
//!         - `0x02` - Data message
//!         - `0x05` - Command message
//!         - `0x07` - Error message
//!     - `ADDR` = 0...255 - Index of the input channel (0 - first input channel)
//!     - `TYPE` - type of values in the array in `DATA` field
//!         - 8 - u8, 1 byte unsigned integer value
//!         - 9 - i8, 1 byte signed integer value
//!         - 16 - u16, 2 byte unsigned integer value
//!         - 17 - i16, 2 byte signed integer value
//!         - 32 - u32, 4 byte unsigned integer value
//!         - 33 - i32, 4 byte signed integer value
//!         - 132 - f32, 4 bytes float value
//!     - `COUNT` - length of the array in the `DATA` field, number of values of type specified in the `TYPE` field
//!     - `DATA` - array of values of type specified in the `TYPE` field
//!
//! - **Error codes**
//!     `0x01` - System error
//!     `0x02` - ADC Error
//!     `0x03` - DMA Error
//!     `0x04` - Network error
//!     `...` - To be extended if necessary
//!
//! ## 2 Initialization
//!
//! `Client` sends to `Device`
//!     `[0x22, 0x04]`
//!
//! - If `Server` hasn't any active connection
//!     - `Client` IP registered
//!     - Data transmission to the registered `Client` IP
//!     - All additional connection ignored until current `Client` is active
//! - If `Server` already has active connection with same IP, connection restored
//! - If `Server` already has active connection, with different IP, initialization ignored
//!
//! ## 3 Flow
//!
//! - `Server` begins transitions of the data messages immediately after initialization, continues until disconnected
//!     `[0x02, 0x01, 0x16, 0x02, 0x00, 0xXX, ..., 0xXX]` - data message
//!     - Byte 0:  0x02 - Data message,
//!     - Byte 1: 0x01 - Index of channel (Second channel)
//!     - Byte 2: 0x16 - Type of values in the array (2 byte unsigned integer value)
//!     - Byte 3, 4: 0x02, 0x00 - Count of values of type u16 in the array (512)
//!     - Data bytes of length 1024 bytes (512 values u16)
//! - Any time the `Server` has internal error, it's code immediately sent to the `Client`
//!     `[0x07, 0x01]`
//!     - Byte 0: `0x07` - Error message
//!     - Byte 1: `0x01` - Error code
//! - `Server` received the Command message, it's handled, applied, then `Server` returns to the data transmission
//!     TODO: Command messages to be defined later
//!
//! ## 4. Configuration example for single Sub MC:
//!
//! **Default port number 15180**
//!
//! ```yaml
//! service UdpClient UdpClientSencor01:
//!     cycle: 10ms
//!     ...
//! ```
//!
mod input_type {
use std::fmt::Display;
use sal_core::error::Error;
///
/// `TYPE` - type of values in the array in `DATA` field
///   - 8 - u8, 1 byte unsigned integer value
///   - 9 - i8, 1 byte signed integer value
///   - 16 - u16, 2 byte unsigned integer value
///   - 17 - i16, 2 byte signed integer value
///   - 32 - u32, 4 byte unsigned integer value
///   - 33 - i32, 4 byte signed integer value
///   - 132 - f32, 4 bytes float value
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum InputType {
    U8 = 8,
    I8 = 9,
    U16 = 16,
    I16 = 17,
    U32 = 32,
    I32 = 33,
    F32 = 132,
}
//
//
impl InputType {
    ///
    /// Returns size of the [InputType] in bytes
    pub fn size(&self) -> usize {
        match self {
            InputType::U8 => 1,
            InputType::I8 => 1,
            InputType::U16 => 2,
            InputType::I16 => 2,
            InputType::U32 => 4,
            InputType::I32 => 4,
            InputType::F32 => 4,
        }
    }
}
//
//
impl TryFrom<u8> for InputType {
    type Error = Error;
    ///
    /// Returns [InputType] created from it's raw value
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            val if val == InputType::U8 as u8 => Ok(InputType::U8),
            val if val == InputType::I8 as u8 => Ok(InputType::I8),
            val if val == InputType::U16 as u8 => Ok(InputType::U16),
            val if val == InputType::I16 as u8 => Ok(InputType::I16),
            val if val == InputType::U32 as u8 => Ok(InputType::U32),
            val if val == InputType::I32 as u8 => Ok(InputType::I32),
            val if val == InputType::F32 as u8 => Ok(InputType::F32),
            _ => Err(Error::new("InputType", "new").err(format!("Unknown InputType value {val}")))
        }
    }
}
//
//
impl Default for InputType {
    fn default() -> Self {
        Self::U16
    }
}
//
//
impl Display for InputType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputType::U8 => write!(f, "U8"),
            InputType::I8 => write!(f, "I8"),
            InputType::U16 => write!(f, "U16"),
            InputType::I16 => write!(f, "I16"),
            InputType::U32 => write!(f, "U32"),
            InputType::I32 => write!(f, "I32"),
            InputType::F32 => write!(f, "F32"),
        }
    }
}}
mod parse_point {
use chrono::{DateTime, Utc};
use sal_core::error::Error;
use sal_sync::services::entity::{{Point, PointType}, Status};
///
/// Returns updated points parsed from the data slice from the S7 device,
pub trait ParsePoint: Send {
    ///
    /// Returns the type of the configured point
    fn typ(&self) -> PointType;
    ///
    /// Adding new raw data to be parsed
    fn add(&mut self, bytes: &[u8], status: Status, timestamp: DateTime<Utc>) -> Result<Vec<Point>, Error> ;
    ///
    /// Returns raw protocol specific address
    fn name(&self) -> String;
    ///
    /// Returns size of the type in the bytes
    fn size(&self) -> usize;
    ///
    /// Returns protocol specific bytes ready to write represents [value]
    fn to_bytes(&self, point: &Point) -> Result<Vec<u8>, String>;
}
}
mod udp_client_conf {
use sal_sync::services::conf::ConfDuration;
use serde::{Deserialize, Deserializer, Serialize};
use std::{str::FromStr, time::Duration};
///
/// ### Creates `UdpClient` config from serde_yaml::Value
///
/// **Example**
///
/// ```yaml
/// reconnect: 1000 ms                      # reconnect timeout when connection is lost
/// protocol: 'udp-raw'                     # udp-raw
/// local-address: 192.168.100.100:15180    # Local machine address
/// remote-address: 192.168.100.241:15180   # IP Address of the vibro-sensor ADC unit
/// mtu: 1500                               # Maximum Transmission Unit, default 1500
/// ```
///
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UdpClientConf {
    pub description: Option<String>,
    pub reconnect: ConfDuration,
    pub protocol: String,
    #[serde(alias = "local-address")]
    pub local_addr: String,
    #[serde(alias = "remote-address")]
    pub remote_addr: String,
    /// Maximum Transmission Unit, default 1500, [Resolve IPv4 Fragmentation, MTU...](https://www.cisco.com/c/en/us/support/docs/ip/generic-routing-encapsulation-gre/25885-pmtud-ipfrag.html)
    pub mtu: usize,
}
}
mod udp_client_connect {
//!
//! # Communication with Vibro-analytics microcontroller (Sub MC) over udp simple protocol
//!
//! Default port number 15180
//!
//! ## Message structure
//!
//!     |Field name:   | FUN | ADDR | TYPE | COUNT | DATA        |
//!     |---           | --- | ---- | ---- | ----- | ----        |
//!     |Data type:    | u8  | u8   | u8   | u32   | [T; COUNT]    |
//!     |Example value:| 22  | 0    | 16   | 512  | [u16; 512] |
//!
//!     - `FUN` Functional byte,
//!         - `0x22` - Initialization message
//!         - `0x02` - Data message
//!         - `0x05` - Command message
//!         - `0x07` - Error message
//!     - `ADDR` = 0...255 - Index of the input channel (0 - first input channel)
//!     - `TYPE` - type of values in the array in `DATA` field
//!         - 8 - u8, 1 byte unsigned integer value
//!         - 9 - i8, 1 byte signed integer value
//!         - 16 - u16, 2 byte unsigned integer value
//!         - 17 - i16, 2 byte signed integer value
//!         - 32 - u32, 4 byte unsigned integer value
//!         - 33 - i32, 4 byte signed integer value
//!         - 132 - f32, 4 bytes float value
//!     - `COUNT` - length of the array in the `DATA` field, number of values of type specified in the `TYPE` field
//!     - `DATA` - array of values of type specified in the `TYPE` field
//!
//! ## Error codes
//!     `0x01` - System error
//!     `0x02` - ADC Error
//!     `0x03` - DMA Error
//!     `0x04` - Network error
//!     `...` - To be extended if necessary
//!
//! ## Basic configuration parameters:
//!
//! ```yaml
//! service UdpClientConnect Id:
//!     parameter: value    # meaning
//!     parameter: value    # meaning
//! ```
//! Message in the UDP has fallowing fiels
//!
//! |Field name:   | SYN | ADDR | TYPE | COUNT | DATA        |
//! |---           | --- | ---- | ---- | ----- | ----        |
//! |Data type:    | u8  | u8   | u8   | u32   | u8[1024]    |
//! |Example value:| 22  | 0    | 16   | 1024  | [u16; 1024] |
//! - `SYN` = 22 - message starts with
//! - `ADDR` = 0...255 - an address of the input channel (0 - first input channel)
//! - `TYPE` - type of values in the array in `DATA` field
//!     - 8 - 1 byte integer value
//!     - 16 - 2 byte float value
//!     - 32 - u16[1024] an array of 2 byte values of length 512
//! - `COUNT` - length of the array in the `DATA` field
//! - `DATA` - array of values of type specified in the `TYPE` field
//!
use std::{net::UdpSocket, time::Duration};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::services::entity::{Name, Object};
use super::UdpClient;
///
/// Establish a connection with Vibro-analytics microcontroller (Sub MC) over udp simple protocol
pub struct UdpClientConnect {
    name: Name,
    local_addr: String,
    remote_addr: String,
    mtu: usize,
    dbg: Dbg,
}
//
//
impl UdpClientConnect {
    ///
    /// Crteates new instance of the [UdpClientConnect]
    pub fn new(parent: impl Into<String>, local_addr: impl Into<String>, remote_addr: impl Into<String>, mtu: usize) -> Self {
        let name = Name::new(parent, "UdpClientConnect");
        let dbg = Dbg::new(name.parent(), name.me());
        Self {
            name,
            local_addr: local_addr.into(),
            remote_addr: remote_addr.into(),
            mtu,
            dbg,
        }
    }
    ///
    /// Returns the socket ready to receive data messages
    /// - Connected to the remote address
    /// - Hanshaked - Start message sent and acknowledged
    fn handshake(&self, socket: &UdpSocket) -> Result<(), Error> {
        let error = Error::new(&self.name, "handshake");
        let mut buf = vec![0; self.mtu];
        match socket.send_to(&[UdpClient::SYN, UdpClient::EOT], &self.remote_addr) {
            Ok(_) => {
                log::debug!("{}.handshake | Start message sent to'{}'", self.dbg, self.remote_addr);
                match socket.recv_from(&mut buf) {
                    Ok((_, src_addr)) => {
                        match buf.as_slice() {
                            // Start ACK received
                            &[UdpClient::SYN, UdpClient::EOT] | &[UdpClient::SYN, UdpClient::EOT, ..] => {
                                log::trace!("{}.handshake | {}: Start message ACK - Ok", self.dbg, src_addr);
                                Ok(())
                            }
                            // Unexpected Data message received, but Start message expected
                            &[UdpClient::DAT, _addr, _type_, _c1,_c2,_c3, _c4, ..] => {
                                Err(error.err(format!("Start message ACK expected, but Data message received: {:?}...", &buf[..=10])))
                            }
                            &[UdpClient::ERR, err] | &[UdpClient::ERR, err, ..] => {
                                Err(error.err(format!("Start message ACK expected, but error received: {:?}", err)))
                            }
                            // Empty message received
                            &[] => Err(error.err("Start message ACK expected, but empty message received")),
                            // Unknown message received
                            _ => Err(error.err(format!("Start message ACK expected, but unknown message received: {:?}", &buf[..=10]))),
                        }
                    }
                    Err(err) => {
                        // notify.add(State::UdpRecvError, format!("{}.handshake | UdpSocket recv error: {:#?}", self_id, err)),
                        match err.kind() {
                            std::io::ErrorKind::WouldBlock => Err(error.pass_with(format!("Socket read timeout"), err.to_string())),
                            std::io::ErrorKind::TimedOut => Err(error.pass_with(format!("Socket read timeout"), err.to_string())),
                            _ => Err(error.pass_with(format!("Socket error"), err.to_string())),
                        }
                    }
                }
            }
            Err(err) => Err(error.pass_with("Can't send Start message", err.to_string())),
        }
    }
    ///
    /// Returns the socket ready to receive data messages
    /// - Connected to the remote address
    /// - Hanshaked - Start message sent and acknowledged
    pub fn connect(&self) -> Result<UdpSocket, Error> {
        let error = Error::new(&self.name, "connect");
        match UdpSocket::bind(&self.local_addr) {
            Ok(socket) => {
                match socket.connect(&self.remote_addr) {
                    Ok(_) => {
                        if let Err(err) = socket.set_read_timeout(Some(Duration::from_millis(512))) {
                            log::error!("{}.connect | Socket Set read timeout error: {:?}", self.dbg, err);
                        }
                        if let Err(err) = socket.set_write_timeout(Some(Duration::from_millis(512))) {
                            log::error!("{}.connect | Socket Set write timeout error: {:?}", self.dbg, err);
                        }
                        match self.handshake(&socket) {
                            Ok(_) => Ok(socket),
                            Err(err) => Err(error.pass(err)),
                        }
                    }
                    Err(err) => {
                        Err(error.pass_with("UdpSocket.connect error", err.to_string()))
                    }
                }
            }
            Err(err) => Err(error.pass_with("UdpSocket::bind error", err.to_string()))
        }
    }
}
//
//
impl Object for UdpClientConnect {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
//
impl std::fmt::Debug for UdpClientConnect {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UdpClientConnect")
            .field("id", &self.dbg)
            .finish()
    }
}
}
mod udp_client {
use std::{cell::RefCell, fs, io::Write, net::UdpSocket, sync::{Arc, atomic::{AtomicBool, Ordering}}};
use chrono::{DateTime, Utc};
use concat_string::concat_string;
use function_name::named;
use indexmap::IndexMap;
use sal_core::{dbg::Dbg, error::{Error, ErrorLimit}};
use sal_sync::{
    kernel::state::ChangeNotify,
    services::{Service, ServiceCycle, Services, entity::{
        Name, Object,
    }}, sync::{Handles, channel::Sender}, thread_pool::Scheduler
};
use crate::{err, err_pass};
use super::{InputType, UdpClientConnect, UdpClientConf};
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum State {
    None,
    Start,
    Exit,
    ReadError,
    ConnectError,
    Connected,
}
pub struct UdpClient {
    name: Name,
    conf: UdpClientConf,
    socket: RefCell<Option<UdpSocket>>,
    buff: RefCell<Vec<u8>>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
impl UdpClient {
    pub const SYN: u8 = 0x22;
    pub const EOT: u8 = 0x04;
    pub const DAT: u8 = 0x02;
    pub const CMD: u8 = 0x05;
    pub const ERR: u8 = 0x07;
    pub const HEAD_LEN: usize = 7;
    const SAMPLE_SIZE: usize = 2;
    pub fn new(parent: impl Into<String>, conf: UdpClientConf) -> Self {
        let name = Name::new(parent, crate::me::<Self>());
        let dbg = Dbg::new(name.parent(), name.me());
        let mtu = conf.mtu;
        Self {
            name,
            conf,
            socket: RefCell::new(None),
            buff: RefCell::new(vec![0; mtu]),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
    #[named]
    pub fn read(&self, values: &mut Vec<Vec<u16>>) -> Result<(), Error> {
        let len = self.receive().map_err(|err| err_pass!(self.dbg, err))?;
        self.parse(values, len).map_err(|err| err_pass!(self.dbg, err))?;
        Ok(())
    }
    #[inline]
    fn convert(&self, channel: usize, channels: usize, bytes: &[u8], values: &mut Vec<u16>) -> Result<(), Error> {
        log::trace!("{}.convert | bytes: {:?}", self.dbg, bytes);
        if bytes.is_empty() {
            return Err(Error::new(&self.name, "convert").err("Input is empty"));
        }
        let (words, remainder) = bytes.as_chunks::<{ Self::SAMPLE_SIZE }>();
        log::trace!("{}.convert | words: {:?}", self.dbg, words.len());
        if remainder.len() > 0 {
            log::warn!("{}. convert | Wrong input len {}, must be divisible by 2", self.dbg, remainder.len());
        }
        let full_length = (bytes.len() / channels) / Self::SAMPLE_SIZE;
        if values.len() < full_length {
            values.resize(full_length, 0);
        }
        let mut index = 0;
        let values = &mut values[..full_length];
        for word in words.iter().skip(channel).step_by(channels) {
            values[index] = u16::from_le_bytes(*word);
            index += 1;
        }
        log::trace!("{}.convert | values: {:?}", self.dbg, values);
        Ok(())
    }
    #[named]
    #[inline]
    fn parse(&self, values: &mut Vec<Vec<u16>>, len: usize) -> Result<(), Error> {
        let buff = &self.buff.borrow()[..len];
        match buff {
            [UdpClient::DAT, channels, typ, c1, c2, c3, c4, ..] => {
                let count = u32::from_le_bytes([*c1, *c2, *c3, *c4]) as usize;
                let typ = InputType::try_from(*typ)
                    .map_err(|err| err_pass!(self.dbg, err, "Wrong value type {}", typ))?;
                let bytes = buff.get(UdpClient::HEAD_LEN..(UdpClient::HEAD_LEN + count))
                    .ok_or(err!(self.dbg, "Wrong message length: {}, expected {}", buff.len(), UdpClient::HEAD_LEN + count))?;
                if *channels as usize > values.len() {
                    values.resize_with(*channels as usize, Vec::new);
                }
                for channel in 0..*channels {
                    let channel_values = &mut values[channel as usize];
                    if let Err(err) = self.convert(channel as usize, *channels as usize, bytes, channel_values) {
                        return Err(err_pass!(self.dbg, err));
                    }
                }
                Ok(())
            }
            [UdpClient::ERR, err] | [UdpClient::ERR, err, ..] => {
                Err(err_pass!(self.dbg, err, "Error received from ADC"))
            }
            [UdpClient::SYN] | [UdpClient::SYN, ..] => {
                let len = std::cmp::min(buff.len(), 12);
                Err(err!(self.dbg, "Data message expected, but SYN received: {:?}...", &buff[..len]))
            }
            [] => Err(err!(self.dbg, "Empty message received")),
            _ => {
                let len = std::cmp::min(buff.len(), 12);
                Err(err!(self.dbg, "Unknown message format: {:?}...", &buff[..len]))
            }
        }
    }
    #[named]
    #[inline]
    fn receive(&self) -> Result<usize, Error> {
        if self.socket.borrow().is_none() {
            let udp_connect = UdpClientConnect::new(&self.name, &self.conf.local_addr, &self.conf.remote_addr, self.conf.mtu);
            match udp_connect.connect() {
                Err(err) => return Err(err_pass!(self.dbg, err, "Socket is not connected")),
                Ok(socket) => {
                    _ = self.socket.borrow_mut().replace(socket);
                }
            }
        }
        let socket = self.socket.borrow();
        let Some(socket) = socket.as_ref() else  {
            return Err(err!(self.dbg, "Socket is not connected"));
        };
        let mut buff = self.buff.borrow_mut();
        match socket.recv_from(&mut buff) {
            Ok((len, _)) => {
                Ok(len)
            }
            Err(err) => match err.kind() {
                std::io::ErrorKind::WouldBlock => Err(err_pass!(self.dbg, err, "Socket read timeout")),
                std::io::ErrorKind::TimedOut => Err(err_pass!(self.dbg, err, "Socket read timeout")),
                _ => Err(err_pass!(self.dbg, err, "Socket read timeout")),
            }
        }
    }
    pub fn exit(&self) {
        if let Some(s) = self.socket.borrow_mut().take() {
            drop(s)
        }
    }
}
mod tests {
    use sal_sync::services::conf::{ConfDuration, ConfDurationUnit};
    use super::*;
    fn mock_conf() -> UdpClientConf {
        UdpClientConf {
            description: Some("test_convert_empty_bytes_returns_error".into()),
            reconnect: ConfDuration::new(1000, ConfDurationUnit::Millis),
            protocol: "udp-raw".into(),
            local_addr: "0.0.0.0".into(),
            remote_addr: "0.0.0.0".into(),
            mtu: 1500,
        }
    }
    fn make_udp_header(fun: u8, channels: u8, data_type: u8, data_bytes_count: u32) -> Vec<u8> {
        let mut header = vec![fun, channels, data_type];
        header.extend_from_slice(&data_bytes_count.to_le_bytes());
        header
    }
}
}
mod udpc_parse_u16 {
use chrono::{DateTime, Utc};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::services::entity::{
    Cot, Point, PointConf, PointHlr, PointType, Status,
};
use super::ParsePoint;
#[derive(Debug)]
pub struct UdpcParseU16 {
    pub txid: usize,
    pub typ: PointType,
    pub name: String,
    pub status: Status,
    dbg: Dbg,
}
impl UdpcParseU16 {
    const SIZE: usize = 2;
    pub fn new(
        txid: usize,
        parent: impl Into<String>,
        conf: &PointConf,
    ) -> UdpcParseU16 {
        let dbg =  Dbg::new(parent, format!("UdpcParseU16({})", conf.name));
        UdpcParseU16 {
            txid,
            typ: conf.type_.clone(),
            name: conf.name.clone(),
            status: Status::Invalid,
            dbg,
        }
    }
    fn convert(&mut self, bytes: &[u8]) -> Result<impl Iterator<Item = u16>, Error> {
        log::trace!("{}.convert | bytes: {:?}", self.dbg, bytes);
        if !bytes.is_empty() {
            let (words, remainder) = bytes.as_chunks::<{ Self::SIZE }>();
            log::trace!("{}.convert | words: {:?}", self.dbg, words.len());
            if remainder.len() > 0 {
                Err(Error::new(&self.name, "convert").err(format!("Wrong input len {}, must be divisible by 2", remainder.len())))
            } else {
                let values = words.iter().enumerate().map(|(index, word)| {
                    log::trace!("{}.convert | index: {}  |  word: {:?}", self.dbg, index, word);
                    u16::from_be_bytes(*word)
                });
                log::trace!("{}.convert | values: {:?}", self.dbg, values);
                Ok(values)
            }
        } else {
            Err(Error::new(&self.name, "convert").err("Input is empty"))
        }
    }
    fn add(&mut self, bytes: &[u8], status: Status, timestamp: DateTime<Utc>) -> Result<Vec<Point>, Error> {
        let dbg = self.dbg.clone();
        self.status = status;
        let name = self.name.clone();
        let txid = self.txid;
        match self.convert(bytes) {
            Ok(values) => {
                Ok(values.map(move |value| Point::Int(PointHlr::new(
                    txid,
                    &name,
                    value as i64,
                    status,
                    Cot::Inf,
                    timestamp,
                ))).collect())
            }
            Err(err) => Err(Error::new(dbg, "add").pass(err))
        }
    }
}
impl ParsePoint for UdpcParseU16 {
    fn typ(&self) -> PointType {
        self.typ.clone()
    }
    fn add(&mut self, bytes: &[u8], status: Status, timestamp: DateTime<Utc>) -> Result<Vec<Point>, Error> {
        self.add(bytes, status, timestamp)
    }
    fn name(&self) -> String {
        self.name.clone()
    }
    fn size(&self) -> usize {
        Self::SIZE
    }
    fn to_bytes(&self, point: &Point) -> Result<Vec<u8>, String> {
        match point.try_as_int() {
            Ok(point) => {
                log::trace!("{}.write | converting '{}' into i16...", self.dbg, point.value);
                match i16::try_from(point.value) {
                    Ok(value) => {
                        Ok(value.to_le_bytes().to_vec())
                    }
                    Err(err) => {
                        let message = format!("{}.write | '{}' to i16 conversion error: {:#?} in the parse point: {:#?}", self.dbg, point.value, err, self.name);
                        log::warn!("{}", message);
                        Err(message)
                    }
                }
            }
            Err(_) => {
                let message = format!("{}.write | Point of type 'Int' expected, but found '{:?}' in the parse point: {:#?}", self.dbg, point.typ(), self.name);
                log::warn!("{}", message);
                Err(message)
            }
        }
    }
}
}
pub(crate) use input_type::*;
pub(crate) use parse_point::*;
pub use udp_client_conf::*;
pub(crate) use udp_client_connect::*;
pub use udp_client::*;
pub(crate) use udpc_parse_u16::*;
}
pub(crate) use udp_client::*;
mod sql_export {
use std::{marker::PhantomData, sync::Arc};
use sal_core::dbg::Dbg;
use vibro_core::Eval;
type ApiClient = crate::infra::ApiClient;
pub struct SqlExport<F, OnErr, Ctx, Child> {
    api_client: Arc<ApiClient>,
    on_err: OnErr,
    builder: F,
    child: Child,
    _ctx: PhantomData<Ctx>,
    dbg: Dbg,
}
impl<F, OnErr, Ctx, Child> SqlExport<F, OnErr, Ctx, Child>
where
    F: Fn(&Ctx) -> Vec<String>,
    OnErr: Fn(Ctx) -> Result<Ctx, Ctx>,
    Child: Eval<Ctx, Ctx> + Send + 'static {
    ///
    /// ### Returns `SqlExport` new instance
    /// - `parent` - Идентификатор родительской сущности (для отладки).
    /// - `on_err` - Замыкание в котором проверяем наличие ошибок в контексте.
    /// - `builder` - Замыкание в котором формируются SQL запросы.
    /// - `child` - Дочерний (предыдущий) расчетный шаг
    pub fn new(parent: &Dbg, api_client: Arc<ApiClient>, on_err: OnErr, builder: F, child: Child) -> Self {
        let dbg = Dbg::new(parent, crate::me::<Self>());
        Self {
            api_client,
            on_err,
            builder,
            child,
            _ctx: PhantomData,
            dbg,
        }
    }
}
//
impl<F, OnErr, Ctx, Child> Eval<Ctx, Ctx> for SqlExport<F, OnErr, Ctx, Child>
where
    F: Fn(&Ctx) -> Vec<String>,
    OnErr: Fn(Ctx) -> Result<Ctx, Ctx>,
    Child: Eval<Ctx, Ctx> + Send + 'static {
    #[inline]
    fn eval(&self, ctx: Ctx) -> Ctx {
        let mut ctx = self.child.eval(ctx);
        let ctx = match (self.on_err)(ctx) {
            Err(ctx) => return ctx,
            Ok(ctx) => ctx,
        };
        let sqls = (self.builder)(&ctx);
        let mut results = Vec::with_capacity(sqls.len());
        for sql in sqls {
            let r = self.api_client.fetch(sql);
            results.push(r);
        }
        for r in results.into_iter().map(|r| r.wait()).flatten() {
            if let Err(err) = r {
                log::warn!("{}.eval | {:?}", self.dbg, err);
                break;
            }
        }
        ctx
    }
    fn exit(&self) {
        self.child.exit();
    }
}
pub fn escape(input: &str) -> String {
    let trimmed = input.trim();
    let mut result = String::with_capacity(trimmed.len() + 8);
    for c in trimmed.chars() {
        match c {
            '\0' => continue,
            '\'' => result.push_str("''"),
            _ => result.push(c),
        }
    }
    result
}
}
pub(self) use sql_export::*;
mod vibro_adc {
use std::{cell::Cell, fmt::Write, sync::Arc, time::Duration};
use chrono::{DateTime, Utc};
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{kernel::state::ExitNotify, services::{EventValueAccess, Service, ServiceWaiting, entity::{Name, Object}}, sync::Handles, thread_pool::Scheduler};
use vibro_core::{DiagFeatures, DiagnosticResult, Eval, FaultKind, Severity, VibroSensor};
use crate::{domain::Sender , err, err_pass};
pub struct VibroAdc<F> {
    name: Name,
    conf: Vec<super::SensorConf>,
    event_values: Arc<F>,
    retain: Arc<vibro_core::Retain>,
    api_link: Sender<String>,
    equipment: String,
    vibration_trends: String,
    vibration_faults: String,
    scheduler: Scheduler,
    handles: Handles<()>,
    exit: Arc<ExitNotify>,
    dbg: Dbg,
}
impl<F> VibroAdc<F>
where
    F: EventValueAccess<str, f64> {
    pub fn new(
        parent: impl Into<String>,
        conf: Vec<super::SensorConf>,
        event_values: Arc<F>,
        retain: Arc<vibro_core::Retain>,
        api_link: Sender<String>,
        equipment: impl AsRef<str>,
        vibration_trends: impl AsRef<str>,
        vibration_faults: impl AsRef<str>,
        scheduler: Scheduler,
        exit: Arc<ExitNotify>,
    ) -> Self {
        let name = Name::new(parent, "VibroSensor");
        let dbg = Dbg::new(name.parent(), name.me());
        Self {
            name,
            conf,
            event_values,
            retain,
            api_link,
            equipment: equipment.as_ref().to_string(),
            vibration_trends: vibration_trends.as_ref().to_string(),
            vibration_faults: vibration_faults.as_ref().to_string(),
            scheduler,
            handles: Handles::new(&dbg),
            exit,
            dbg,
        }
    }
}
impl<F> Object for VibroAdc<F> {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
impl<F> std::fmt::Debug for VibroAdc<F> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("VibroAdc")
            .field("name", &self.name)
            .finish()
    }
}
//
impl<F> Service for VibroAdc<F>
where
    F: EventValueAccess<str, f64> + Sync + Send + 'static {
    #[named]
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let name = self.name.clone();
        let conf = self.conf.clone();
        let connection_conf = conf.first().ok_or(err!(dbg, "Configuration is empty, no sensors configured"))?.connection.clone();
        let api_link = self.api_link.clone();
        let equipment = self.equipment.clone();
        let vibration_trends = self.vibration_trends.clone();
        let vibration_faults = self.vibration_faults.clone();
        let event_values = self.event_values.clone();
        let retain = self.retain.clone();
        let exit = self.exit.clone();
        let wait_started = Some(Duration::from_millis(10));
        let service_waiting = ServiceWaiting::new(&name, wait_started);
        let service_release = service_waiting.release();
        let handle = self.scheduler.spawn(move || {
            service_release.add(Ok(()));
            let dbg = &dbg;
            let udp = super::UdpClient::new(dbg, connection_conf);
            let mut samples = conf.iter().map(|conf| vec![0u16; conf.dsp.adc.chunk_size]).collect();
            let sensors: Vec<(_, _)> = conf.iter().map(|conf| {
                let rpm_key = match &conf.rpm {
                    crate::services::vibro_monitor::InputKind::Const(rpm) => {
                        let key = format!("{name}/rpm");
                        event_values.insert(&key, *rpm);
                        key
                    }
                    crate::services::vibro_monitor::InputKind::Point(k) => k.clone(),
                };
                let sensor = VibroSensor::new(dbg, &conf.dsp, rpm_key, &event_values, &retain,
                    |ctx| {
                        if ctx.is_err() { return; }
                        let equipment_name = &conf.target;
                        let count = ctx.results.iter().filter(|r| r.severity != Severity::Green).count();
                        if count > 0 {
                            let results = ctx.results.iter().filter(|r| r.severity != Severity::Green);
                            let sql = vibration_faults_sql(&equipment, &vibration_faults, equipment_name, results, count);
                            let _ = api_link.send(sql);
                        }
                        if !ctx.features.is_empty() {
                            let sql = vibration_trends_sql(&equipment, &vibration_trends, equipment_name, &ctx.features);
                            let _ = api_link.send(sql);
                        }
                    },
                    &exit,
                ).unwrap();
                (conf.clone(), sensor)
            }).collect();
            while !exit.get() {
                match udp.read(&mut samples) {
                    Err(err) => log::warn!("{dbg}.run | {}", err),
                    Ok(_) => {
                        for ((_conf, sensor), channel_samples) in sensors.iter().zip(&mut samples) {
                            if let Err(err) = sensor.eval(channel_samples) {
                                log::warn!("{dbg}.run | {:?}", err);
                            }
                        }
                    }
                }
            }
            udp.exit();
            for (_, sensor) in &sensors {
                sensor.exit();
            }
            log::info!("{dbg}.run | Exit");
        }).map_err(|err| err_pass!(self.dbg, err, "Start failed"))?;
        self.handles.push(handle);
        let r = match wait_started {
            Some(_) => {
                log::info!("{}.run | Waiting while starting...", self.dbg);
                service_waiting.wait()
            }
            None => Ok(()),
        };
        log::info!("{}.run | Starting - ok", self.dbg);
        r
    }
    fn is_finished(&self) -> bool {
        self.handles.is_finished()
    }
    fn wait(&self) -> Result<(), Error> {
        self.handles.wait()
    }
    fn exit(&self) {
        self.exit.exit();
    }
}
fn vibration_trends_sql(
    table_equipment: &str,
    table_trends: &str,
    equipment_name: &str,
    features: &[DiagFeatures],
) -> String {
    let mut sql = String::with_capacity(750 + features.len() * 120);
    sql.push_str("WITH new_data (eq_name, timestamp, order_id, rms_value, phase, rpm) AS (\n        VALUES ");
    for (i, r) in features.iter().enumerate() {
        if i > 0 { sql.push_str(", "); }
        _ = write!(
            sql,
            "('{}', '{}'::timestamptz, '{}', {}, {}, {})",
            equipment_name,
            r.ts.to_rfc3339(),
            r.order_id,
            r.rms.value(),
            r.phase.to_degrees(),
            r.rpm.value()
        );
    }
    _ = write!(
        sql,
        r#"
),
target_equipment AS (
    SELECT
        d.timestamp,
        e.id AS equipment_id,
        d.order_id,
        d.rms_value,
        d.phase,
        d.rpm
    FROM new_data d
    JOIN {} e ON e.name = d.eq_name
)
INSERT INTO {} (timestamp, equipment_id, order_id, rms_value, phase, rpm)
SELECT timestamp, equipment_id, order_id, rms_value, phase, rpm
FROM target_equipment
ON CONFLICT (timestamp, equipment_id, order_id) DO NOTHING;"#,
    table_equipment,
    table_trends
    );
    sql
}
fn vibration_faults_sql<'a>(
    table_equipment: &str,
    table_faults: &str,
    equipment_name: &str,
    results: impl Iterator<Item = &'a DiagnosticResult>,
    count: usize,
) -> String {
    let mut sql = String::with_capacity(750 + count * 120);
    sql.push_str(r#"WITH new_data (eq_name, timestamp, fault_kind, score, severity, rpm) AS (
        VALUES
    "#);
    for (i, r) in results.enumerate() {
        if i > 0 { sql.push_str(", "); }
        _ = write!(
            sql,
            "('{}', '{}'::timestamptz, '{}', {}, '{}'::vibration_severity, {})",
            equipment_name,
            r.ts.to_rfc3339(),
            r.fault,
            r.score,
            r.severity,
            r.rpm.value()
        );
    }
    _ = write!(sql, r#"
),
target_equipment AS (
    -- Связываем имена с ID за один проход
    SELECT
        d.timestamp,
        e.id AS equipment_id,
        d.fault_kind,
        d.score,
        d.severity,
        d.rpm
    FROM new_data d
    JOIN {} e ON e.name = d.eq_name
)
INSERT INTO {} (timestamp, equipment_id, fault_kind, score, severity, rpm)
SELECT timestamp, equipment_id, fault_kind, score, severity, rpm
FROM target_equipment
ON CONFLICT (equipment_id, fault_kind)
DO UPDATE SET
    timestamp = EXCLUDED.timestamp,
    score     = EXCLUDED.score,
    severity  = EXCLUDED.severity,
    rpm       = EXCLUDED.rpm;
"#, table_equipment, table_faults);
    sql
}
}
pub(self) use vibro_adc::*;
