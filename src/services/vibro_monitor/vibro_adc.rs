use std::{sync::Arc, time::Duration, fmt::Write};
use chrono::Utc;
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{kernel::state::ExitNotify, services::{EventValueAccess, Service, ServiceWaiting, entity::{Name, Object}}, sync::Handles, thread_pool::Scheduler};
use vibro_core::{Eval, Severity, VibroSensor};
use crate::{domain::{RECV_TIMEOUT, RecvTimeoutError, Sender, }, err, err_pass};

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
    /// Конфигурация модуля виброаналитики
    conf: Vec<super::SensorConf>,
    /// Провайдер среза входных евентов
    event_values: Arc<F>,
    /// Хранение пар Key-Value на диске
    retain: Arc<vibro_core::Retain>,
    /// Очередь для SQL запросов
    api_link: Sender<String>,
    /// Thread scheduler
    scheduler: Scheduler,
    /// Handles of the internaly executed threads 
    handles: Handles<()>,
    /// Exit signal
    exit: Arc<ExitNotify>,
    /// Для отладки
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
    /// - `api_link` - Провайдер отправки SQL запросов
    pub fn new(
        parent: impl Into<String>,
        conf: Vec<super::SensorConf>,
        event_values: Arc<F>,
        retain: Arc<vibro_core::Retain>,
        api_link: Sender<String>,
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
        let event_values = self.event_values.clone();
        let retain = self.retain.clone();
        let exit = self.exit.clone();
        // Ожидание пока сервис запустится
        let wait_started = Some(Duration::from_millis(1));
        let service_waiting = ServiceWaiting::new(&name, wait_started);
        let service_release = service_waiting.release();
        let handle = self.scheduler.spawn(move || {
            service_release.add(Ok(()));
            let dbg = &dbg;
            let udp = super::UdpClient::new(dbg, connection_conf);
            let mut samples = conf.iter().map(|conf| vec![0u16; conf.dsp.adc.chunk_size]).collect();
            let sensors: Vec<(_, _)> = conf.iter().map(|conf| {
                let rpm_key = match conf.rpm {
                    crate::services::vibro_monitor::InputKind::Const(rpm) => {
                        let key = format!("{name}/rpm");
                        event_values.insert(key.clone(), rpm);
                        key
                    }
                    crate::services::vibro_monitor::InputKind::Point(k) => k,
                };
                let sensor = VibroSensor::new(dbg, &conf.dsp, rpm_key, &event_values, &retain,
                    |ctx| {
                        if ctx.is_err() { return; }
                        let equipment_id = &conf.target;
                        let mut sql = String::with_capacity(ctx.results.len() * 120 + 150);
                        let mut results = ctx.results.iter().filter(|r| r.severity != Severity::Green).peekable();
                        if results.peek().is_some() {
                            "WITH target_item AS (
                                -- Находим id товара по его имени из справочника Items
                                SELECT id FROM Items WHERE name = 'Ноутбук' LIMIT 1
                            )
                            INSERT INTO vibration_faults (timestamp, equipment_id, fault_kind, score, severity, rpm)
                            SELECT id, 50000 FROM target_item
                            WHERE id IS NOT NULL -- Защита на случай, если имя не найдено в Items
                            ON CONFLICT (item_id) 
                            DO UPDATE SET value = EXCLUDED.value;";
                            
                            sql.push_str("INSERT INTO vibration_faults (timestamp, equipment_id, fault_kind, score, severity, rpm) VALUES ");
                            for (i, r) in results.enumerate() {
                                if i > 0 { sql.push_str(", "); }
                                _ = write!( // use std::fmt::Write - Required
                                    sql,
                                    "('{}', {}, '{}', {}, '{}', {})",
                                    r.ts.to_rfc3339(),
                                    equipment_id,
                                    r.fault,
                                    r.score,
                                    r.severity,
                                    r.rpm.value()
                                );
                            }
                            sql.push_str(" ON CONFLICT (equipment_id, fault_kind) DO UPDATE SET ");
                            sql.push_str("timestamp = EXCLUDED.timestamp, score = EXCLUDED.score, severity = EXCLUDED.severity, rpm = EXCLUDED.rpm;");
                            _ = api_link.send(sql);
                        }
                        if !ctx.features.is_empty() {
                            let mut sql = String::with_capacity(ctx.features.len() * 120 + 150);
                            sql.push_str("INSERT INTO order_vibration_trends (timestamp, equipment_id, order_id, rms_value, phase, rpm) VALUES ");
                            for (i, r) in ctx.features.iter().enumerate() {
                                if i > 0 { sql.push_str(", "); }
                                _ = write!(
                                    sql,
                                    "('{}', {}, '{}', {}, {}, {})",
                                    r.ts.to_rfc3339(),
                                    equipment_id,
                                    r.order_id,
                                    r.rms.value(),
                                    r.phase.to_degrees(),
                                    r.rpm.value()
                                );
                            }
                            sql.push_str(" ON CONFLICT (timestamp, equipment_id, order_id) DO NOTHING;");
                            _ = api_link.send(sql);
                        }
                    },
                    exit.clone(),
                ).unwrap();
                (conf.clone(), sensor)
            }).collect();
            while !exit.get() {
                // Получение АЦП-выборки из сети
                udp.read(&mut samples);
                // Запускаем расчеты
                for ((_conf, sensor), channel_samples) in sensors.iter().zip(&mut samples) {
                    sensor.eval(channel_samples);
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
