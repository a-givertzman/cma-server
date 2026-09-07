use std::{fmt::Write, sync::Arc, time::Duration};
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{kernel::state::ExitNotify, services::{EventValueAccess, Service, ServiceWaiting, entity::{Name, Object}}, sync::Handles, thread_pool::Scheduler};
use vibro_core::{DiagFeatures, DiagnosticResult, Eval, Severity, VibroSensor};
use crate::{domain::Sender, err, err_pass};

/// ### VibroSensor | Расчетный вибродиагностики для одного датчика
/// 
/// Алгоритмы анализа и диагностики разделены на три частотных диапазона:
///  
/// #### 1. Диагностика низкочастотных макромеханических дефектов (диапазон 0.2x .. 3.0x RPM)
/// 
///    * Вычисляет текущую частоту вращения (RPM) и фазовый угол поворота ротора по сигналу вибродатчика.
///    * Выполняет синхронное временно́е накопление и преобразование исходного сигнала из временного домена в угловой домен.
///    * Производит спектральный анализ (БПФ) полученного сигнала в угловом домене.
///    * Осуществляет цифровую фильтрацию и сглаживание кратковременных выбросов амплитуд целевых гармоник (0.5x, 1.0x, 1.5x, 2.5x, 3.0x).
///    * Реализует логику автоматической классификации дефектов на основе паттернов и порогов целевых гармоник.
///    * Регистрирует тренды целевых гармоник и фиксирует результаты диагностики дефектов в БД.
/// 
/// #### 2. Диагностика развитых (проявленных) дефектов (среднечастотный диапазон 10x RPM .. 5 кГц: BPFI, BPFO, FTF, BSF)
/// 
///    * **Функционал в разработке (не реализован)**
///    * Выполняет полосовую фильтрацию (Bandpass) исходного вибросигнала в целевом диапазоне частот.
///    * Пересчитывает отфильтрованный сигнал из временной области в угловую на основе единой сетки углов поворота вала (Angular Grid).
///    * Вычисляет упорядоченный спектр (Order Spectrum) с заданным размером окна БПФ (FFT Size).
///    * Производит автоматический поиск дефектов подшипников (BPFI, BPFO, FTF, BSF) с помощью детектора (Defect Detector) по настроенным порогам.
///    * Формирует SQL-запросы на основе результатов детекции и экспортирует данные в БД через API-клиент.
/// 
/// 
/// #### 3. Диагностика дефектов на ранней стадии зарождения (высокочастотный диапазон 5 кГц .. 15 кГц: BPFI, BPFO, FTF, BSF)
///    * **Функционал в разработке (не реализован)**
///    * Выполняет полосовую фильтрацию (Bandpass) высокочастотного вибросигнала в диапазоне 5–10 кГц для изоляции резонансов.
///    * Выделяет огибающую отфильтрованного сигнала (Signal Envelope) методом демодуляции для обнаружения повторяющихся ударных импульсов.
///    * Пересчитывает полученную огибающую из временной области в угловую на основе единой сетки углов поворота вала (Angular Grid).
///    * Вычисляет упорядоченный спектр огибающей (Order Spectrum) с заданным размером окна БПФ (FFT Size).
///    * Реализует автоматическую идентификацию зарождающихся дефектов подшипников (BPFI, BPFO, FTF, BSF) детектором по высокочастотным порогам.
///    * Формирует SQL-запросы с результатами анализа и экспортирует их в базу данных через API-клиент.
pub struct VibroAdc<F> {
    name: Name,
    /// Конфигурация модуля виброаналитики.
    conf: Vec<super::SensorConf>,
    /// Провайдер среза входных евентов.
    event_values: Arc<F>,
    /// Хранение пар Key-Value на диске.
    retain: Arc<vibro_core::Retain>,
    /// Очередь для SQL запросов.
    api_link: Sender<String>,
    /// Таблица | Справочник оборудования.
    equipment: String,
    /// Таблица | Тренды вибрации.
    vibration_trends: String,
    /// Таблица | Состояния оборудования на основании вибрации.
    vibration_faults: String,
    /// Thread scheduler.
    scheduler: Scheduler,
    /// Handles of the internaly executed threads. 
    handles: Handles<()>,
    /// Exit signal.
    exit: Arc<ExitNotify>,
    /// Для отладки.
    dbg: Dbg,
}
//
//
impl<F> VibroAdc<F>
where
    F: EventValueAccess<str, f64> {
    ///
    /// ### Returns [VibroSensor] new instance
    /// - `parent` - Parent entity identifier (for debugging).
    /// - `conf` - Конфигурации цифровой обработки вибросигналов.
    /// - `event_values` - Агрегатор входных эвентов.
    /// - `retain` - Хранение пар Key-Value на диске.
    /// - `api_link` - Провайдер отправки SQL запросов.
    /// - `equipment` - Таблица | Справочник оборудования.
    /// - `vibration_trends` - Таблица | Тренды вибрации.
    /// - `vibration_faults` - Таблица | Состояния оборудования на основании вибрации.
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
// 
impl<F> Object for VibroAdc<F> {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
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
    // 
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
        // Ожидание пока сервис запустится
        let wait_started = Some(Duration::from_millis(10));
        let service_waiting = ServiceWaiting::new(&name, wait_started);
        let service_release = service_waiting.release();
        let handle = self.scheduler.spawn(move || {
            service_release.add(Ok(()));
            let dbg = &dbg;
            let udp = super::UdpClient::new(dbg, conf.len(), connection_conf);
            let mut samples = conf.iter().map(|conf| vec![0u16; conf.dsp.adc.chunk_size]).collect();
            let sensors: Vec<(_, _)> = conf.iter().filter_map(|conf| {
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
                            if let Some(sql) = sql {
                                // TODO: Добавить разовое логирование в момент перехода в состояние ошибки
                                let _ = api_link.try_send(sql);
                            }
                        }
                        if !ctx.features.is_empty() {
                            let sql = vibration_trends_sql(&equipment, &vibration_trends, equipment_name, &ctx.features);
                            if let Some(sql) = sql {
                                // TODO: Добавить разовое логирование в момент перехода в состояние ошибки
                                let _ = api_link.try_send(sql);
                            }
                        }
                    },
                    &exit,
                );
                match sensor {
                    Ok(sensor) => Some((conf.clone(), sensor)),
                    Err(err) => {
                        log::warn!("{dbg}.run | Can't create vibro sensor for '{}': {:?}", conf.target, err);
                        None
                    }
                }
            }).collect();
            while !exit.get() {
                // Получение АЦП-выборки из сети
                match udp.read(&mut samples) {
                    Err(err) => log::warn!("{dbg}.run | {}", err),
                    Ok(_) => {
                        // Запускаем расчеты
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
    //
    fn is_finished(&self) -> bool {
        self.handles.is_finished()
    }
    //
    fn wait(&self) -> Result<(), Error> {
        self.handles.wait()
    }
    //
    fn exit(&self) {
        self.exit.exit();
    }    
}

/// ### Формирует SQL для таблицы трендов вибрации.
/// - `table_equipment` - Таблица | Справочник оборудования.
/// - `table_trends` - Таблица | Тренды вибрации.
/// - `equipment_name` - Уникальное наименование целевого механизма.
/// - `features` - Признаки целевых порядков.
fn vibration_trends_sql(
    table_equipment: &str,
    table_trends: &str,
    equipment_name: &str,
    features: &[DiagFeatures],
) -> Option<String> {
    if features.is_empty() { return None; }
    let equipment_name = super::escape(equipment_name);
    let mut sql = String::with_capacity(750 + features.len() * 120);
    sql.push_str("WITH new_data (eq_name, timestamp, order_id, rms_value, phase, rpm) AS (\n        VALUES ");
    for (i, r) in features.iter().enumerate() {
        if i > 0 { sql.push_str(", "); }
        _ = write!( // use std::fmt::Write - Required
            sql,
            "('{}', '{}'::timestamptz, '{}', {}, {}, {})",
            equipment_name,
            r.ts.to_rfc3339(),
            r.order_id,
            wrap_nan(&r.rms.value()),
            wrap_nan(&r.phase.to_degrees()),
            wrap_nan(&r.rpm.value())
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
    Some(sql)
}

/// ### Формирует SQL для таблицы состояния оборудования на основании вибрации.
/// - `table_equipment` - Таблица | Справочник оборудования.
/// - `table_faults` - Таблица | Оценка состояния оборудования на основании вибрации
/// - `equipment_name` - Уникальное наименование целевого механизма.
/// - `results` - Итератор по результатам диагностики для данного механизма.
/// - `count` - Количество записей в итераторе.
fn vibration_faults_sql<'a>(
    table_equipment: &str,
    table_faults: &str,
    equipment_name: &str,
    results: impl Iterator<Item = &'a DiagnosticResult>,
    count: usize,
) -> Option<String> {
    if count == 0 { return None; }
    let equipment_name = super::escape(equipment_name);
    let mut sql = String::with_capacity(750 + count * 120);
    sql.push_str(r#"WITH new_data (eq_name, timestamp, fault_kind, score, severity, rpm) AS ( 
        VALUES 
    "#);
    for (i, r) in results.enumerate() {
        if i > 0 { sql.push_str(", "); }
        _ = write!( // use std::fmt::Write - Required
            sql,
            "('{}', '{}'::timestamptz, '{}', {}, '{}'::vibration_severity, {})",
            equipment_name,
            r.ts.to_rfc3339(),
            r.fault,
            wrap_nan(&r.score),
            r.severity,
            wrap_nan(&r.rpm.value())
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
    Some(sql)
}
/// Подготовка NaN для БД
fn wrap_nan(v: &f64) -> &dyn std::fmt::Display {
    if v.is_nan() {
        return &"'NaN'";
    }
    v
}