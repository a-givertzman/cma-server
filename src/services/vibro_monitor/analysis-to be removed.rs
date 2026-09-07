use std::{sync::Arc, time::Duration, fmt::Write};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{kernel::state::ExitNotify, services::{EventValueAccess, RegistryConf, Service, ServiceWaiting, Services, SubscriptionCriteria, conf::ServicesConf, entity::{Cot, Name, Object, Point}}, sync::Handles, thread_pool::{Scheduler, ThreadPool}};
use vibro_core::{AngularGrid, Autocorrelation, Context, Eval, Frame, ImbContext, ImbalanceDetector, LowPassSignal, OrderDomainSamples, OrderFeatureFilter, OrderSpectrum, Pass, ReadInputs, Retain, Severity, WindowFn};
use crate::domain::{FxSccHashMap, RECV_TIMEOUT, Receiver, RecvTimeoutError, Sender, unbounded};

/// ### Analysis | Расчетный модуль вибродиагностики
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
pub struct Analysis<T> {
    name: Name,
    /// Конфигурации обработки вибросигнала с одним IP адресом.
    conf: Vec<super::SensorConf>,
    /// Провайдер среза входных евентов
    event_values: Arc<T>,
    /// Хранение пар Key-Value на диске
    retain: Arc<vibro_core::Retain>,
    /// Thread scheduler
    scheduler: Scheduler,
    /// Handles of the internaly executed threads 
    handles: Handles<()>,
    /// Exit signal
    exit: Arc<ExitNotify>,
    dbg: Dbg,
}
//
//
impl<T> Analysis<T> {
    ///
    /// ### Returns [Analysis] new instance
    /// - `parent` - Parent entity identifier (for debugging).
    /// - `equipment_id` - Идентификатор наблюдаемого механизма.
    /// - `conf` - Конфигурации обработки вибросигнала с одним IP адресом.
    /// - `event_values` - Агрегатор входных эвентов.
    /// - `retain` - Хранение пар Key-Value на диске.
    pub fn new(
        parent: impl Into<String>,
        conf: Vec<super::SensorConf>,
        event_values: Arc<T>,
        retain: Arc<vibro_core::Retain>,
        scheduler: Scheduler,
        exit: Arc<ExitNotify>,
    ) -> Self {
        let name = Name::new(parent, "Analysis");
        let dbg = Dbg::new(name.parent(), name.me());
        Self {
            name,
            conf,
            event_values,
            retain,
            scheduler,
            handles: Handles::new(&dbg),
            exit,
            dbg,
        }
    }
}
//
impl<T> Object for Analysis<T> {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
// 
impl<T> std::fmt::Debug for Analysis<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Analysis")
            .field("dbg", &self.dbg)
            .finish()
    }
}
//
impl<T: EventValueAccess<str, f64> + Send + Sync> Service for Analysis<T> {
    // 
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let name = self.name.clone();
        let conf = self.conf.clone();
        // Идентификатор наблюдаемого механизма
        let equipment_id = conf.target.clone();
        let wait_started = Some(Duration::from_millis(1));
        let service_waiting = ServiceWaiting::new(&name, wait_started);
        let service_release = service_waiting.release();
        let event_values = self.event_values.clone();
        let scheduler = self.scheduler.clone();
        let exit = self.exit.clone();
        let handle = self.scheduler.spawn(move || {
            let dbg = &dbg;
            service_release.add(Ok(()));
            let mut udp = super::UdpClient::new(dbg, conf.connection);
            // Вектор массивов для чтения из UDP
            let mut samples: Vec<Vec<u16>> = conf.iter().map(|_| [0u16; Frame::SIZE]).collect();

            while !exit.get() {
                match udp.read(&mut samples) {
                    Err(err) => log::warn!("{}.run | {:?}", self.dbg, err),
                    Ok(_) => {
                        
                    }
                }
            }

            let mut ctx = Context::new();
            let angular_grid = AngularGrid::new(dbg,
                Autocorrelation::new(&dbg,
                    conf.dsp.adc.sample_rate_hz,
                    ReadInputs::new(&dbg, event_values.clone())
                ),
            );
            let window_size = conf.dsp.analysis.n_fft();
            let window_fn = WindowFn::<f32>::kaiser(&dbg, window_size, window_size, 0, 5.65).unwrap();
            let api_client: Arc<crate::infra::ApiClient>;
            let low_range = super::SqlExport::new(&dbg,
                api_client,
                |mut ctx| {
                    if ctx.is_err() {
                        return Err(ctx.pass_err(&self.dbg, "eval"));
                    }
                    Ok(ctx)
                },
                move |ctx| {
                    if ctx.is_err() { return vec![]; }
                    let mut sqls = Vec::with_capacity(2);
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
                        sqls.push(sql);
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
                        sqls.push(sql);
                    }
                    sqls
                },
                ImbalanceDetector::new(dbg,
                    OrderFeatureFilter::new(dbg,
                        conf.dsp.analysis.n_fft(),
                        conf.dsp.analysis.angular_step_rad(),
                        OrderSpectrum::new(dbg,
                            conf.dsp.analysis.n_fft(),
                            Some(window_fn),
                            OrderDomainSamples::new(dbg,
                                conf.dsp.analysis.samples_per_rev(),
                                LowPassSignal::new(dbg,
                                    conf.dsp.adc.sample_rate_hz,
                                    conf.dsp.analysis.bands.low_cutoff_order(),
                                    Pass::new(),
                                ),
                            ),
                        ),
                    ),
                ),
            );
            let mut low_range_ctx = ImbContext::new(dbg, conf.dsp.analysis.samples_per_rev(), conf.analysis.n_fft(), retain);
            let (low_send, low_recv) = crate::domain::bounded(1);
            let (mid_send, mid_recv) = crate::domain::bounded(1);
            let (high_send, high_recv) = crate::domain::bounded(1);
            _ = scheduler.spawn({
                move || {
                loop {
                    match low_recv.recv_timeout(RECV_TIMEOUT) {
                        Ok(frame) => {
                            low_range_ctx.update(frame);
                            low_range_ctx = low_range.eval(low_range_ctx);
                        }
                        Err(RecvTimeoutError::Timeout) => {}
                        _ => break,
                    }
                }
            }});
            _ = scheduler.spawn({
                move || {
                loop {
                    match mid_recv.recv_timeout(RECV_TIMEOUT) {
                        Ok(frame) => {
                            // mid_range_ctx.update(frame);
                            // mid_range_ctx = mid_range.eval(mid_range_ctx);
                        }
                        Err(RecvTimeoutError::Timeout) => {}
                        _ => break,
                    }
                }
            }});
            _ = scheduler.spawn({
                move || {
                loop {
                    match high_recv.recv_timeout(RECV_TIMEOUT) {
                        Ok(frame) => {
                            // high_range_ctx.update(frame);
                            // high_range_ctx = high_range.eval(high_range_ctx);
                        }
                        Err(RecvTimeoutError::Timeout) => {}
                        _ => break,
                    }
                }
            }});
        
            while !exit.get() {
                // Получение АЦП-выборки из сети
                udp.read(&mut samples);
                ctx.push_chunk(&samples);
                let phases;
                (ctx, phases) = angular_grid.eval(ctx);
                match &ctx.err {
                    Some(err) => log::warn!("{}", err),
                    None => {
                        if ctx.ac_samples.is_full() {
                            let ts = Utc::now();
                            let frame = Frame::new(ts, &samples, phases);
                            _ = low_send.send(frame.clone());
                            _ = mid_send.send(frame.clone());
                            _ = high_send.send(frame.clone());
                        }
                    }
                }
            }
            log::info!("{dbg}.run | Exit");
        });
        match handle {
            Ok(handle) => self.handles.push(handle),
            Err(err) => return Err(Error::new(&self.dbg, "run").pass_with("Start failed", err.to_string())),
        }
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
    fn wait(&self) -> Result<(), Error> {
        self.handles.wait()
    }
    //
    fn is_finished(&self) -> bool {
        self.handles.is_finished()
    }
    //
    fn exit(&self) {
        self.exit.exit();
    }    
}
