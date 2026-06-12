use std::{fs::OpenOptions, io::{BufWriter, Write}, path::PathBuf, sync::Arc};
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{kernel::state::ExitNotify, services::{Service, ServiceCycle, Services, entity::{Cot, Name, Object, Point, PointConf, PointHlr}, types::Bool}, sync::{Handles, Owner}, thread_pool::Scheduler};
use serde::{Serialize, Serializer, ser::SerializeMap};
use crate::{domain::{FxSccHashMap, RECV_TIMEOUT, Receiver, RecvTimeoutError, Sender, bounded, me}, err, err_pass, services::task::{RetainMode, TaskRetainConf, retain::{RetainEvent, RetainState, RetainValue}}};

///
/// ### Retained values for the task
/// 
/// Отложенная запись retain значений для оптимизации работы с диском.
/// Получает Key-Point в канале, сохраняет в единый для `TaskRetain` файл добавлением в конец.
/// Периодически актуализирует весь журнал.
pub struct TaskRetain {
    txid: usize,
    name: Name,
    cache: Arc<FxSccHashMap<String, Point>>,
    conf: TaskRetainConf,
    path: PathBuf,
    send: Sender<RetainEvent>,
    recv: Owner<Receiver<RetainEvent>>,
    scheduler: Scheduler,
    handles: Handles<()>,
    exit: Arc<ExitNotify>,
    dbg: Dbg,
}
//
impl TaskRetain {
    ///
    /// Returns `TaskRetain` new instance
    /// - `parent` - Родительская сущность.
    /// - `txid` - Идентификатор сервиса отправителя (в данном случае родительского `Task`).
    #[named]
    pub fn new(parent: &Name, txid: usize, conf: TaskRetainConf, services: &Arc<Services>, scheduler: Scheduler) -> Result<Self, Error> {
        let name = Name::new(parent, me::<Self>());
        let dbg = Dbg::new(parent, me::<Self>());
        let Some(retain_path) = services.retain().path else {
            return Err(err!(dbg, "Retain: path - missed in Application config"));
        };
        let cw_dir = std::env::current_dir().map_err(|err| err_pass!(dbg, err))?;
        let dir = cw_dir.join(retain_path).join(parent.join().trim_start_matches("/"));
        std::fs::create_dir_all(&dir).map_err(|err| err_pass!(dbg, err, "Error creating dir: '{}'", dir.display()))?;
        let path = dir.join("retain").with_extension("json");
        let (send, recv) = bounded(4096);
        Ok(Self {
            txid,
            name,
            cache: Arc::new(FxSccHashMap::default()),
            conf,
            path,
            send,
            recv: Owner::new(recv),
            scheduler,
            handles: Handles::new(parent),
            exit: Arc::new(ExitNotify::new(parent, None, None)),
            dbg,
        })
    }
    ///
    /// ### Returns link to send `Point` to be retained
    fn link(&self) -> Sender<RetainEvent> {
        self.send.clone()
    }
    ///
    /// ### Returns Point for the specified `key`
    pub fn get(&self, key: &str) -> Option<Point> {
        self.cache.read_sync(key, |_, p| p.clone())
    }
    ///
    /// ### Чтение с диска и парсинг `RetainState`
    fn load(&self) -> Option<Point> {
        let f = std::fs::File::open(&self.path).ok()?;
        let state: RetainState = serde_json::from_reader(f).map_err(|err| {
            log::error!("{}.load | Can't parse JSON from {}: {:?}", self.id, self.path.display(), err);
        }).ok()?;
        let key = todo!();
        Some(match state.value {
            RetainValue::Bool(v) => Point::Bool(PointHlr::new(self.txid, &key, Bool(v), state.status, Cot::Inf, state.ts)),
            RetainValue::Int(v) => Point::Int(PointHlr::new(self.txid, &key, v, state.status, Cot::Inf, state.ts)),
            RetainValue::Real(v) => Point::Real(PointHlr::new(self.txid, &key, v, state.status, Cot::Inf, state.ts)),
            RetainValue::Double(v) => Point::Double(PointHlr::new(self.txid, &key, v, state.status, Cot::Inf, state.ts)),
            RetainValue::String(v) => Point::String(PointHlr::new(self.txid, &key, v, state.status, Cot::Inf, state.ts)),
            RetainValue::Bytes(v) => Point::Bytes(PointHlr::new(self.txid, &key, v, state.status, Cot::Inf, state.ts)),
        })
    }
    // ///
    // /// Serialise key-value into single Map entry
    // fn serialize_entry(key: &str, data: impl Serialize) -> Result<>
    ///
    /// ### Пишет один пакет с кадрированием длины
    #[named]
    fn append<T: Serialize>(
        dbg: &Dbg,
        writer: &mut BufWriter<std::fs::File>, 
        mode: RetainMode, 
        key: &String,
        data: &T,
    ) -> Result<(), Error> {
        match mode {
            RetainMode::Debug => {
                let mut serializer = serde_json::Serializer::new(&mut *writer);
                let mut map = serializer.serialize_map(Some(1)).map_err(|err| err_pass!(dbg, err))?;
                map.serialize_entry(&key, &data).map_err(|err| err_pass!(dbg, err))?;
                map.end().map_err(|err| err_pass!(dbg, err))?;
                writer.write_all(b"\n").map_err(|err| err_pass!(dbg, err))?;                            
            }
            RetainMode::Release => {
                postcard::to_io(&(key, data), writer).map_err(|err| err_pass!(dbg, err))?;
            }
        }
        Ok(())
    }
    ///
    /// ### Физическая запись состояния на диск
    /// 
    /// `RetainState` пишется через атомарную подмену файлов
    #[named]
    fn store(&self) -> Result<(), Error> {
        let mut snapshot = Vec::with_capacity(self.cache.len());
        self.cache.iter_sync(|key, point| {
            snapshot.push((key.clone(), point.clone()));
            true
        });
        let tmp_path = self.path.with_extension("json.tmp");
        // let mut f = std::fs::OpenOptions::new().truncate(true).create(true).write(true).open(&tmp_path)
        let file = std::fs::File::create(&tmp_path)
            .map_err(|err| err_pass!(self.dbg, err, "Can't open '{}'", tmp_path.display()))?;
        let mut writer = BufWriter::new(file);
        for (key, point) in snapshot {
            if let Err(err) = Self::append(&self.dbg, &mut writer, self.conf.mode, &key, &RetainState::from(point)) {
                log::warn!("{}.store | Can't store '{}' into '{}', error: {:?}", self.dbg, key, tmp_path.display(), err);
            }
        }
        let file = writer.into_inner()
            .map_err(|err| err_pass!(self.dbg, err, "Can't flush {}", tmp_path.display()))?;
        file.sync_data()
            .map_err(|err| err_pass!(self.dbg, err, "Can't Sync {}", tmp_path.display()))?;
        std::fs::rename(&tmp_path, &self.path)
            .map_err(|err| err_pass!(self.dbg, err, "Can't Rename '{}' -> '{}'", tmp_path.display(), self.path.display()))?;
        log::trace!("{}.store | Can't retain state to '{}'", self.dbg, self.path.display());
        Ok(())
    }
}
//
impl Object for TaskRetain {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
impl std::fmt::Debug for TaskRetain {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TaskRetain")
            .field("name", &self.name)
            .finish()
    }
}
//
impl Service for TaskRetain {
    //
    // fn get_link(&self, _: &str) -> Sender<Point> {
    //     self.send.clone()
    // }
    //
    #[named]
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let exit = self.exit.clone();
        let conf = self.conf.clone();
        let rx_recv = self.recv.take().ok_or_else(|| err!(dbg, "Can't take recv"))?;
        let path = match conf.mode {
            RetainMode::Debug => self.path.with_extension("json"),
            RetainMode::Release => self.path.with_extension("dat"),
        };
        let file = OpenOptions::new()
            .append(true)  // Писать в конец файла
            .create(true)  // Создать файл, если файла нет
            .write(true)   // Разрешить запись
            .open(&path)
            .map_err(|err| err_pass!(dbg, err))?;
        let cache = self.cache.clone();
        let handle = self.scheduler.spawn({
            let dbg = dbg.clone();
            move || {
            let mut cycle = ServiceCycle::new(&dbg, conf.journal.flush.interval);
            let mut writer = BufWriter::new(file);
            'main: while !exit.get() {
                cycle.start();
                match rx_recv.recv_timeout(RECV_TIMEOUT) {
                    Ok(event) => {
                        log::trace!("{dbg}.run | point '{}': {:?}", event.key, event.val);
                        let sate = RetainState::from(&event.val);
                        if let Err(err) = Self::append(&dbg, &mut writer, conf.mode, &event.key, &sate) {
                            log::warn!("{dbg}.run | Can't store retain '{}' to '{}', error: {:?}", event.key, path.display(), err)
                        }
                        cache.insert_sync(event.key, event.val);
                    }
                    Err(err) => match err {
                        RecvTimeoutError::Timeout => {},
                        _ => {
                            log::error!("{dbg}.run | Receiv error: {:?}", err);
                            break 'main;
                        }
                    }
                };
            }
            log::info!("{dbg}.run | Exit");
            Ok(())
        }});
        match handle {
            Ok(handle) => {
                log::info!("{dbg}.run | Starting - ok");
                self.handles.push(handle);
                Ok(())
            }
            Err(err) => Err(err_pass!(dbg, err, "Start failed")),
        }
    }
    //
    //
    fn points(&self) -> Vec<PointConf> {
        self.conf.points()
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
        self.exit.exit();
    }
}
