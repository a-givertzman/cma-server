//!
//! Service storing all received events in the memory,
//! - Subscribe on points by configured criteria
//! - Storing all received events on the disk if 'retain' option is true
//! - Storing received points into the HasMap by name as key
//! - Cyclically delyed stores accumulated changes to the disk if 'retain' option is true
//! Basic configuration parameters:
//! ```yaml
//! service CacheService Cache:
//!     retain: true    # true / false - enables storing cache on the disk
//!     suscribe:
//!         /App/MultiQueue: []
//! ```
use std::{env, fmt::Debug, fs, io::{BufReader, BufWriter, Write}, path::{Path, PathBuf}, sync::{Arc, atomic::{AtomicBool, Ordering}}};
use chrono::Utc;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    collections::FxIndexMap, services::{
        entity::{Cot, Name, Object, Point, PointConf, PointType, PointHlr, PointTxId, Status}, future::Future, types::Bool, Service, Services, SubscriptionCriteria
    }, sync::{channel::RecvTimeoutError, Handles}, thread_pool::Scheduler,
};
use serde::Serialize;
use crate::{domain::{FxDashMap, Sender, RECV_TIMEOUT}, services::{CacheServiceConf, cache::delay_store::DelyStore}};
///
/// CacheService service
/// - Subscribe on points by configured criteria
/// - Storing all received events on the disk if 'retain' option is true
pub struct CacheService {
    dbg: Dbg,
    name: Name,
    conf: CacheServiceConf,
    services: Arc<Services>,
    cache: Arc<FxDashMap<String, Point>>,
    scheduler: Scheduler,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
}
//
//
impl CacheService {
    ///
    /// Creates new instance of the CacheService
    pub fn new(conf: CacheServiceConf, services: Arc<Services>, scheduler: Scheduler) -> Self {
        let dbg = Dbg::new(conf.name.parent(), conf.name.me());
        Self {
            name: conf.name.clone(),
            conf: conf.clone(),
            services,
            cache: Arc::new(FxDashMap::default()),
            scheduler,
            handles: Handles::new(&dbg),
            dbg,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// Returns vector of the SubscriptionCriteria by config and list of configured Point's
    fn subscriptions(&self, conf: &CacheServiceConf, points: &[PointConf]) -> (String, Vec<SubscriptionCriteria>) {
        if conf.subscribe.is_empty() {
            panic!("{}.subscribe | Error. Subscription can`t be empty: {:#?}", self.dbg, conf.subscribe);
        } else {
            log::debug!("{}.subscribe | conf.subscribe: {:#?}", self.dbg, conf.subscribe);
            let subscriptions = conf.subscribe.with(points);
            log::trace!("{}.subscribe | subscriptions: {:#?}", self.dbg, subscriptions);
            if subscriptions.len() > 1 {
                panic!("{}.run | Error. Task does not supports multiple subscriptions for now: {:#?}.\n\tTry to use single subscription.", self.dbg, subscriptions);
            } else {
                match subscriptions.clone().into_iter().next() {
                    Some((service_name, Some(points))) => {
                        (service_name, points)
                    }
                    Some((_, None)) => panic!("{}.run | Error. Subscription configuration error in: {:#?}", self.dbg, subscriptions),
                    None => panic!("{}.run | Error. Subscription configuration error in: {:#?}", self.dbg, subscriptions),
                }
            }
        }
    }
    ///
    /// Creates directiry (all necessary folders in the 'path' if not exists)
    ///  - path is relative, will be joined with current working dir
    fn create_dir(dbg: &Dbg, path: &str) -> Result<PathBuf, String> {
        let current_dir = env::current_dir().unwrap();
        let path = current_dir.join(path);
        match path.exists() {
            true => Ok(path),
            false => {
                match fs::create_dir_all(&path) {
                    Ok(_) => Ok(path),
                    Err(err) => {
                        let message = format!("{}.create_dir | Error create path: '{:?}'\n\terror: {:?}", dbg, path, err);
                        log::error!("{}", message);
                        Err(message)
                    }
                }
            }
        }
    }
    ///
    /// Loads stored chache
    fn load(dbg: &Dbg, name: &Name, cache: &FxDashMap<String, Point>) -> Result<(), Error> {
        let error = Error::new(dbg, "load");
        let dir = Name::new("assets/cache/", name.join()).join().trim_start_matches('/').to_owned();
        let path = Path::new(&dir).join("cache.json");
        if !path.is_file() {
            log::info!("{}.load | No cache file found at '{:?}'. Starting with empty cache.", dbg, path);
            return Ok(());
        }
        let f = fs::OpenOptions::new().read(true).open(&path)
            .map_err(|err| error.pass_with(format!("Can't open file '{:?}'", path), err.to_string()))?;
        let rdr = BufReader::new(f);
        let points: Vec<Point> = match serde_json::from_reader(rdr) {
            Ok(points) => points,
            Err(err) => {
                log::error!("{}", error.pass_with(format!("Cache file corrupted '{:?}'. Starting with empty cache", path), err.to_string()));
                return Ok(())
            }
        };
        let len = points.len();
        for point in points {
            cache.insert(point.dest(), point);
        }
        let dir = fs::read_dir(&dir)
            .map_err(|err| error.pass_with(format!("Can't read dir '{:?}'", dir), err.to_string()))?;
        for entry in dir {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                            if name.starts_with("cache.tmp") {
                                if let Err(_) = fs::remove_file(&path) {
                                    log::info!("{}.load | Posible crashed file in the cache dir: '{:?}'", dbg, path);
                                }
                            }
                        }
                    }
                }
                _ => log::info!("{}.load | Posible crashed file in the cache dir: '{:?}'", dbg, entry),
            }
        }
        log::info!("{}.load | Cache loaded ({} points) from: '{:?}'", dbg, len, path);
        Ok(())
    }
    ///
    /// Writes array of the points to the json file:
    /// ```json
    /// [
    ///     {"type": "Bool","value": 1,"name": "/App/path/Point.name1","status": 2,"cot": "Inf","timestamp": "2024-04-08T08:52:32.656576549+00:00"},
    ///     {...,
    ///     ...
    /// ]
    /// ```
    fn write<S: Serialize>(dbg: &Dbg, name: &Name, points: Vec<S>) -> Result<(), Error> {
        let error = Error::new(dbg, "write");
        match Self::create_dir(dbg, Name::new("assets/cache/", name.join()).join().trim_start_matches('/')) {
            Ok(dir) => {
                let path = dir.join("cache.json");
                let path_tmp = dir.join(format!("cache.tmp-{:?}", std::thread::current().id()));
                let f = fs::OpenOptions::new().truncate(true) .create(true).write(true).open(&path_tmp)
                    .map_err(|err| error.pass_with(format!("Can't open file '{:?}'", path_tmp), err.to_string()))?;
                let mut writer = BufWriter::new(f);
                serde_json::to_writer(&mut writer, &points)
                    .map_err(|err| error.pass_with(format!("Can't write file '{:?}'", path_tmp), err.to_string()))?;
                writer.flush()
                    .map_err(|err| error.pass_with(format!("Can't flush file '{:?}'", path_tmp), err.to_string()))?;
                writer.get_ref().sync_all()
                    .map_err(|err| error.pass_with(format!("Can't sync_all file '{:?}'", path_tmp), err.to_string()))?;
                drop(writer);   // Закрываем файл
                fs::rename(&path_tmp, &path)
                    .map_err(|err| error.pass_with(format!("Can't rename temp cache '{:?}' into actual '{:?}'", path_tmp, path), err.to_string()))?;
                let dir_f = fs::File::open(&dir)
                    .map_err(|err| error.pass_with(format!("Can't sync cache dir '{:?}'", dir), err.to_string()))?;
                dir_f.sync_all()
                    .map_err(|err| error.pass_with(format!("Can't sync cache dir '{:?}'", dir), err.to_string()))?;
                log::debug!("{}.write | Cache stored in: {:?}", dbg, path);
                Ok(())
            }
            Err(err) => {
                log::error!("{:#?}", err);
                Err(error.pass(err))
            }
        }
    }
    ///
    /// Stores self.cache on the disk
    fn store(dbg: &Dbg, name: &Name, points: FxIndexMap<String, Point>, status: Status) -> Result<(), Error> {
        let points: Vec<Point> = points.into_iter().map(|(_dest, point)| {
            match point.clone() {
                Point::Bool(mut point) => {
                    point.status = status;
                    Point::Bool(point)
                }
                Point::Int(mut point) => {
                    point.status = status;
                    Point::Int(point)
                }
                Point::Real(mut point) => {
                    point.status = status;
                    Point::Real(point)
                }
                Point::Double(mut point) => {
                    point.status = status;
                    Point::Double(point)
                }
                Point::String(mut point) => {
                    point.status = status;
                    Point::String(point)
                }
                Point::Bytes(mut point) => {
                    point.status = status;
                    Point::Bytes(point)
                }
            }
        }).collect();
        Self::write(dbg, name, points)
    }
    ///
    /// Fills self cache with initial values for all configured points
    pub fn initial(
        dbg: &Dbg,
        txid: usize, 
        cache: &FxDashMap<String, Point>,
        points: &[PointConf],
        initial_status: Status,
    ) {
        let timestamp = Utc::now();
        log::trace!("{}.initial | Initial cashe generated at {:?}", dbg, timestamp);
        for point_config in points {
            let point = match point_config.type_ {
                PointType::Bool => Point::Bool(PointHlr::new(
                    txid,
                    &point_config.name,
                    Bool(false),
                    initial_status,
                    Cot::Inf,
                    timestamp,
                )),
                PointType::Int => Point::Int(PointHlr::new(
                    txid,
                    &point_config.name,
                    0,
                    initial_status,
                    Cot::Inf,
                    timestamp,
                )),
                PointType::Real => Point::Real(PointHlr::new(
                    txid,
                    &point_config.name,
                    0.0,
                    initial_status,
                    Cot::Inf,
                    timestamp,
                )),
                PointType::Double => Point::Double(PointHlr::new(
                    txid,
                    &point_config.name,
                    0.0,
                    initial_status,
                    Cot::Inf,
                    timestamp,
                )),
                PointType::String => Point::String(PointHlr::new(
                    txid,
                    &point_config.name,
                    String::new(),
                    initial_status,
                    Cot::Inf,
                    timestamp,
                )),
                PointType::Bytes => Point::Bytes(PointHlr::new(
                    txid,
                    &point_config.name,
                    vec![],
                    initial_status,
                    Cot::Inf,
                    timestamp,
                )),
                PointType::Json => Point::String(PointHlr::new(
                    txid,
                    &point_config.name,
                    String::new(),
                    initial_status,
                    Cot::Inf,
                    timestamp,
                )),
            };
            cache.insert(SubscriptionCriteria::dest(&Cot::Inf, &point_config.name), point);
        }
    }
    ///
    /// Sorting cached values by key into IndexMap
    fn sorted(cache: &FxDashMap<String, Point>) -> FxIndexMap<String, Point> {
        let vec: Vec<(String, Point)> = cache.iter().map(|r| (r.key().to_string(), r.value().clone())).collect();
        let mut sorted = FxIndexMap::from_iter(vec);
        sorted.sort_by(|k1, _, k2, _| k1.cmp(&k2));
        sorted
    }
}
//
//
impl Object for CacheService {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
//
impl Debug for CacheService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CacheService")
            .field("dbg", &self.dbg)
            .field("name", &self.name)
            .finish()
    }
}
//
//
impl Service for CacheService {
    //
    //
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let self_name = self.name.clone();
        let tx_id = PointTxId::from_str(&self_name.join());
        let exit = self.exit.clone();
        let conf = self.conf.clone();
        let services = self.services.clone();
        let cache = self.cache.clone();
        let point_configs = services.points(&self_name.join())
            .then(
                |points| points,
            |err| {
                log::error!("{}.run | Requesting Points error: {:?}", dbg, err);
                vec![]
            }
        );
        let (service_name, points) = self.subscriptions(&conf, &point_configs);
        log::debug!("{}.run | points: {:#?}", dbg, points.len());
        log::trace!("{}.run | points: {:#?}", dbg, points);
        let (_, rx_recv) = services.subscribe(
            &service_name,
            &self.name.join(),
            &points,
        );
        let mut dely_store = DelyStore::new(conf.retain_delay);
        log::info!("{}.run | Preparing thread...", dbg);
        let handle = self.scheduler.spawn(move || {
            let initial_status = Status::Invalid;
            let retain_status = Status::Invalid;
            Self::initial(&dbg, tx_id, &cache, &point_configs, initial_status);
            if let Err(err) = Self::load(&dbg, &self_name, &cache) {
                log::warn!("{}.run | Error: {:?}", dbg, err);
            }
            'main: loop {
                match rx_recv.recv_timeout(RECV_TIMEOUT) {
                    Ok(point) => {
                        cache.insert(point.dest(), point);
                        if dely_store.exceeded() && Self::store(&dbg, &self_name, Self::sorted(&cache), retain_status).is_ok() {
                            dely_store.set_stored();
                        }
                    }
                    Err(err) => {
                        match err {
                            RecvTimeoutError::Timeout => if !dely_store.stored() {
                                if dely_store.exceeded() && Self::store(&dbg, &self_name, Self::sorted(&cache), retain_status).is_ok() {
                                    dely_store.set_stored();
                                }
                            }
                            _ => {
                                log::error!("{}.run | Error receiving from queue: {:?}", dbg, err);
                                break 'main;
                            }
                        }
                    }
                }
                if exit.load(Ordering::SeqCst) {
                    if !dely_store.stored() {
                        _ = Self::store(&dbg, &self_name, Self::sorted(&cache), retain_status);
                    }
                    break;
                }
            }
            if let Err(err) = services.unsubscribe(&service_name, &self_name.join(), &points) {
                log::error!("{}.run | Unsubscribe error: {:#?}", dbg, err);
            }
            log::info!("{}.run | Exit", dbg);
        });
        match handle {
            Ok(handle) => {
                log::info!("{}.run | Starting - ok", self.dbg);
                self.handles.push(handle);
                Ok(())
            }
            Err(err) => {
                let err = Error::new(&self.dbg, "run").pass_with("Start failed", err.to_string());
                log::warn!("{}", err);
                Err(err)
            }
        }
    }
    //
    //
    fn gi(&self, receiver_name: &str, points: &[SubscriptionCriteria], send: Sender<Point>) -> Future<Result<(), Error>> {
        let dbg = self.dbg.clone();
        log::info!("{}.gi | Gi from '{}' requested {} points", dbg, receiver_name, if points.is_empty() {"all".to_string()} else {points.len().to_string()});
        log::trace!("{}.gi | Gi from '{}' points: {:#?}", dbg, receiver_name, points);
        let (result, sink) = Future::new();
        let cache = self.cache.clone();
        let points = points.to_owned();
        let receiver_name = receiver_name.to_string();
        let handle = self.scheduler.spawn(move || {
            if points.is_empty() {
                for point in cache.iter().map(|r| r.value().clone()) {
                    if let Err(err) = send.send(point) {
                        log::error!("{dbg}.gi | Cant send GI for '{receiver_name}', channel is closed");
                        sink.add(Err(Error::new(&dbg, "gi").pass(err.to_string())));
                        return;
                    }
                }
            } else {
                for point in points {
                    match cache.get(&point.destination()) {
                        Some(point) => {
                            if let Err(err) = send.send(point.clone()) {
                                log::error!("{dbg}.gi | Cant send GI for '{receiver_name}', channel is closed");
                                sink.add(Err(Error::new(&dbg, "gi").pass(err.to_string())));
                                return;
                            }
                        }
                        None => {
                            log::warn!("{dbg}.gi | Requested point '{}' - not found", point.destination());
                        }
                    }
                }
            }
            sink.add(Ok(()));
        });
        match handle {
            Ok(handle) => self.handles.push(handle),
            Err(err) => log::error!("{}.gi | Can't schedule task: {:?}", self.dbg, err),
        }
        result
    }
    //
    //
    fn wait(&self) -> Result<(), Error> {
        self.handles.wait()
    }
    //
    //
    fn is_finished(&self) -> bool {
        self.handles.is_finished()
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }
}
//
// TODO: Критические проблемы и уязвимости
// 
// ## Критические ошибки логики и потери данных
// 
// - Игнорирование таймера сохранения при отсутствии новых событий: 
//     В методе `run` проверка `dely_store.exceeded()` находится строго внутри ветки `Ok(point)`.
//     Если новые события (points) перестают поступать, цикл будет бесконечно уходить в `RecvTimeoutError::Timeout` и игнорировать таймер.
//     Накопленные изменения **не будут сохранены** на диск до тех пор, пока не придет хотя бы одно новое событие или не поступит сигнал завершения.
// 
// - Потеря данных при сбое (Неатомарная запись):
//     В функции `write` файл открывается с флагом `.truncate(true)`.
//     Это мгновенно очищает существующий файл `cache.json`.
//     Если в процессе записи произойдет сбой (кончится место на диске, отключится питание, процесс упадет с паникой),
//     старый кэш будет уничтожен, а новый не дописан. Файл останется пустым или поврежденным.
// 
// - Исчерпание памяти (OOM) и чудовищные аллокации: 
//     Ручная сборка JSON-массива через `points.into_iter().fold(String::new(), ...)` — это крайне опасный паттерн.
//     Для каждой точки создается временная строка, которая затем склеивается в одну гигантскую строку `cache` в оперативной памяти.
//     Если кэш разрастется до сотен тысяч точек, сервис упрется в лимиты памяти и упадет.
// 
// ## Проблемы производительности
// 
// - Блокирующий I/O в потоке планировщика:
//     Методы `write` и `load` используют синхронные вызовы `std::fs`.
//     Судя по наличию `scheduler.spawn` и каналов, вы работаете в асинхронном контексте или пуле потоков.
//     Чтение и особенно запись больших объемов данных на диск заблокирует текущий поток ОС.
//     Это может привести к "голоданию" пула потоков (thread pool starvation) и отказу всего приложения.
// 
// - Глобальные блокировки DashMap:
//     В методе `gi` вызов `cache.iter().map(...).collect()` блокирует все шарды `DashMap` на время итерации и глубокого клонирования каждой точки.
//     При активной записи из других потоков это создаст бутылочное горлышко и сильную деградацию производительности.
// 
// - Неэффективная ручная сериализация:
//     Использование макроса `concat_string!` вместе с `json!(point).to_string()` внутри цикла создает огромный оверхед.
//     Rust и `serde` умеют делать это "из коробки" гораздо быстрее, не создавая промежуточных аллокаций.
// 
// ## Уязвимости и стабильность
// 
// - Уязвимость Path Traversal: 
//     Построение путей вида `Name::new("assets/cache/", name.join())` без строгой валидации содержимого `name`.
//     Если `name.join()` придет извне и будет содержать символы вроде `../../../`,
//     сервис может прочитать или перезаписать критичные файлы за пределами рабочей директории.
// 
// - Паники вместо мягкой деградации:
//     В методе `subscriptions` обильно используется `panic!`.
//     В микросервисной архитектуре неверная конфигурация одной подписки не должна "убивать" весь процесс без возможности
//     обработать ошибку на уровне выше (если только это не строгая задумка при запуске, но в Rust предпочтительнее возвращать `Result`).
// 
// - Состояние гонки (Race Condition) при выходе:
//     Переменная `exit` проверяется в конце цикла. Если очередь `rx_recv` забита сообщениями,
//     цикл может долго не доходить до проверки `exit.load`, из-за чего сервис будет игнорировать команду на плавное завершение (graceful shutdown).
// 
