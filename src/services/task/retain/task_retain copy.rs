use std::{fs::{File, OpenOptions}, io::{BufRead, BufReader, BufWriter, Read, Write}, path::{Path, PathBuf}, sync::Arc, time::{Duration, Instant}};
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{kernel::state::ExitNotify, services::{Service, Services, entity::{Cot, Name, Object, Point, PointConf, PointHlr}, types::Bool}, sync::{Handles, Owner}, thread_pool::Scheduler};
use serde::{Serialize, Serializer, ser::SerializeMap};
use crate::{domain::{FxSccHashMap, RECV_TIMEOUT, Receiver, RecvTimeoutError, Sender, bounded, me}, err, err_pass, services::task::{Append, Compactation, FlushConf, LoadRetain, MarkObcolete, RetainMode, TaskRetainConf, retain::{RetainEvent, RetainState, RetainValue}}};
use super::{RetainCtx};
///
/// ### Локальный флаг завершения чтения из файла
enum IoState {
    Continue((String, RetainState)),
    Done,
}
///
/// ### Retained values for the `Task`
/// 
/// - **Формат данных на диске в режиме `Debug`**
/// ```json
/// { "key_01": { "value": {"Bool": false}, "status": 0, "ts": "2026-06-11T09:06:45.123456789Z" } }
/// { "key_02": { "value": {"Int": 123}, "status": 0, "ts": "2026-06-11T09:06:45.123456789Z" } }
/// { "key_03": { "value": {"Real": 12.3}, "status": 0, "ts": "2026-06-11T09:06:45.123456789Z" } }
/// { "key_04": { "value": {"Double": 12.3}, "status": 0, "ts": "2026-06-11T09:06:45.123456789Z" } }
/// { "key_05": { "value": {"String": "String value"}, "status": 0, "ts": "2026-06-11T09:06:45.123456789Z" } }
/// ```
/// - **Формат данных на диске в режиме `Release`**
/// 
///  Key len | Key |  State len | RetainState
///   :---: | :---: | :---: | :---:
///  `u32`<br>(4 bytes) | `String`<br>( Key len bytes) | `u32`<br>(4 bytes) | `RetainState`<br>(State len bytes)
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
    scheduler: Option<Scheduler>,
    handles: Handles<()>,
    exit: Arc<ExitNotify>,
    dbg: Dbg,
}
//
impl TaskRetain {
    ///
    /// Returns `TaskRetain` new instance
    /// - `parent` - Родительский сервис `Task`.
    /// - `txid` - Идентификатор сервиса отправителя (в данном случае родительского `Task`).
    #[named]
    pub fn new(parent: &Name, txid: usize, conf: TaskRetainConf, services: &Arc<Services>, scheduler: Scheduler) -> Result<Self, Error> {
        let name = Name::new(parent, me::<Self>());
        let dbg = Dbg::new(parent, me::<Self>());
        let Some(retain_path) = services.retain().path else {
            return Err(err!(dbg, "Retain: path - missed in Application config"));
        };
        let cw_dir = std::env::current_dir().map_err(|err| err_pass!(dbg, err))?;
        let dir = cw_dir.join(retain_path).join(parent.join().trim_start_matches('/'));
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
            scheduler: Some(scheduler),
            handles: Handles::new(parent),
            exit: Arc::new(ExitNotify::new(parent, None, None)),
            dbg,
        })
    }
    ///
    /// Returns `TaskRetain` test instance
    pub fn mock(parent: impl Into<String>, cache: impl IntoIterator<Item = (String, Point)>) -> Self {
        let name = Name::new(parent, crate::domain::me::<Self>());
        let dbg = Dbg::new(name.parent(), me::<Self>());
        let (send, recv) = bounded(4096);
        Self {
            txid: 0,
            name,
            cache: Arc::new(cache.into_iter().collect()),
            conf: TaskRetainConf::default(),
            path: PathBuf::new(),
            send,
            recv: Owner::new(recv),
            scheduler: None,
            handles: Handles::new(&dbg),
            exit: Arc::new(ExitNotify::new(&dbg, None, None)),
            dbg,
        }
    }
    ///
    /// ### Returns link to send `Point` to be retained
    pub fn link(&self) -> Sender<RetainEvent> {
        self.send.clone()
    }
    ///
    /// ### Returns Point for the specified `key`
    pub fn get(&self, key: &str) -> Option<Point> {
        self.cache.read_sync(key, |_, p| p.clone())
    }
    ///
    /// ### Парсит одну запись из байтов `postcard` в `(String, RetainState)`
    #[named]
    fn decode_entry(dbg: &Dbg, reader: &mut BufReader<File>, len_buf: &mut [u8; 4], key_buf: &mut Vec<u8>, state_buf: &mut Vec<u8>) -> Result<IoState, Error> {
        if reader.read_exact(len_buf).is_err() { return Ok(IoState::Done); }
        let len = u32::from_le_bytes(*len_buf);
        if len > 1024 {
            return Err(err!(dbg, "Размер ключа retain-записи: {} - превышает допустимые 1KB, вероятно файл кэша испорчен", len))
        }
        key_buf.resize(len as usize, 0u8);
        reader.read_exact(key_buf).map_err(|err| err_pass!(dbg, err))?;
        let key = std::str::from_utf8(key_buf).map_err(|err| err_pass!(dbg, err))?;
        reader.read_exact(len_buf).map_err(|err| err_pass!(dbg, err))?;
        let len = u32::from_le_bytes(*len_buf);
        if len > 10 * 1024 * 1024 {
            return Err(err!(dbg, "Размер значения retain-записи: {} - превышает допустимые 100MB, вероятно файл кэша испорчен", len))
        }
        state_buf.resize(len as usize, 0u8);
        reader.read_exact(state_buf).map_err(|err| err_pass!(dbg, err))?;
        let state = postcard::from_bytes::<RetainState>(state_buf).map_err(|err| err_pass!(dbg, err))?;
        Ok(IoState::Continue((key.to_owned(), state)))
    }
    ///
    /// ### Создает `Point` из `RetainState`
    fn point(state: &RetainState, txid: usize, name: impl Into<String>) -> Point {
        match &state.value {
            RetainValue::Bool(v) => Point::Bool(PointHlr::new(txid, name, Bool(*v), state.status, Cot::Inf, state.ts)),
            RetainValue::Int(v) => Point::Int(PointHlr::new(txid, name, *v, state.status, Cot::Inf, state.ts)),
            RetainValue::Real(v) => Point::Real(PointHlr::new(txid, name, *v, state.status, Cot::Inf, state.ts)),
            RetainValue::Double(v) => Point::Double(PointHlr::new(txid, name, *v, state.status, Cot::Inf, state.ts)),
            RetainValue::String(v) => Point::String(PointHlr::new(txid, name, v.clone(), state.status, Cot::Inf, state.ts)),
            RetainValue::Bytes(v) => Point::Bytes(PointHlr::new(txid, name, v.clone(), state.status, Cot::Inf, state.ts)),
        }
    }
    ///
    /// ### Чтение с диска и парсинг `RetainState`
    /// 
    /// - Максимальный размер ключа - 1 KB
    /// - Максимальный размер `RetainState` - 10 MB
    #[named]
    fn load(&self) -> Result<(), Error> {
        let json_path = self.path.with_extension("json");
        let dat_path = self.path.with_extension("dat");
        if dat_path.exists() {
            let path = dat_path;
            let file = File::open(&path).map_err(|err| err_pass!(self.dbg, err, "Can't open file '{}'", path.display()))?;
            let mut reader = BufReader::new(file);
            let mut len_buf = [0u8; 4];
            let mut key_buf = Vec::with_capacity(1024);
            let mut state_buf = Vec::with_capacity(4096);
            loop {
                match Self::decode_entry(&self.dbg, &mut reader, &mut len_buf, &mut key_buf, &mut state_buf) {
                    Ok(IoState::Done) => break,
                    Ok(IoState::Continue((key, state))) => {
                        let val = Self::point(&state, self.txid, &key);
                        if let Err(err) = self.cache.insert_sync(key, val) {
                            log::warn!("{}.load | Can't extend cache: {:?}", self.dbg, err);
                        }
                    }
                    Err(err) => return Err(err_pass!(self.dbg, err, "Can't parse retain entries in {}", path.display())),
                }
            }
            return Ok(());
        }
        if json_path.exists() {
            let path = json_path;
            let file = File::open(&path).map_err(|err| err_pass!(self.dbg, err, "Can't open file '{}'", path.display()))?;
            let reader = BufReader::new(file);
            let mut lines = reader.lines();
            while let Some(line) = lines.next() {
                match line {
                    Ok(line) => {
                        match serde_json::from_str(&line) {
                            Ok(parsed) => {
                                let (key, state): (String, RetainState) = parsed;
                                let val = Self::point(&state, self.txid, &key);
                                if let Err(err) = self.cache.insert_sync(key, val) {
                                    log::warn!("{}.load | Can't extend cache: {:?}", self.dbg, err);
                                }
                            }
                            Err(err) => log::warn!("{}.load | Can't parse entry in {}, error: {:?}", self.dbg, path.display(), err),
                        }
                    }
                    Err(err) => log::warn!("{}.load | Can't read entry from {}, error: {:?}", self.dbg, path.display(), err),
                }
            }
        }
        Ok(())
    }
    ///
    /// ### Пишет один пакет с кадрированием длины
    /// 
    /// - Возвращает количество записанноых байт
    #[named]
    fn append<T: Serialize>(
        dbg: &Dbg,
        writer: &mut BufWriter<File>, 
        mode: RetainMode, 
        key: &String,
        state: &T,
    ) -> Result<usize, Error> {
        match mode {
            RetainMode::Debug => {
                let mut writer= CountingWriter::new(writer);
                let mut serializer = serde_json::Serializer::new(&mut writer);
                let mut map = serializer.serialize_map(Some(1)).map_err(|err| err_pass!(dbg, err))?;
                map.serialize_entry(key, state).map_err(|err| err_pass!(dbg, err))?;
                map.end().map_err(|err| err_pass!(dbg, err))?;
                writer.write_all(b"\n").map_err(|err| err_pass!(dbg, err))?;
                Ok(writer.bytes_written)
            }
            RetainMode::Release => {
                writer.write_all(&(key.len() as u32).to_le_bytes()).map_err(|err| err_pass!(dbg, err))?;
                writer.write_all(key.as_bytes()).map_err(|err| err_pass!(dbg, err))?;
                let bytes = postcard::to_allocvec(state).map_err(|err| err_pass!(dbg, err))?;
                writer.write_all(&(bytes.len() as u32).to_le_bytes()).map_err(|err| err_pass!(dbg, err))?;
                writer.write_all(&bytes).map_err(|err| err_pass!(dbg, err))?;
                Ok(4 + key.len() + 4 + bytes.len())
            }
        }
    }
    ///
    /// ### Compactation
    fn compactation(
        dbg: &Dbg,
        path: &Path,
        conf: &TaskRetainConf,
        cache: &Arc<FxSccHashMap<String, Point>>,
        mut writer: BufWriter<File>,
    ) -> Result<(u64, BufWriter<File>), (BufWriter<File>, Error)> {
        if let Err(err) = writer.flush() {
            return Err((writer, err_pass!(dbg, err, "Can't flush active {}", path.display())));
        }
        match Self::store(dbg, path, conf, cache) {
            Ok(_) => match Self::open(path, &conf.journal.flush) {
                Ok((size, w)) => Ok((size, w)),
                Err(err) => Err((writer, err_pass!(dbg, err))),
            }
            Err(err) => Err((writer, err)),
        }
    }
    ///
    /// ### Физическая запись состояния на диск (Compactation)
    /// 
    /// `RetainState` пишется через атомарную подмену файлов
    #[named]
    fn store(dbg: &Dbg, path: &Path, conf: &TaskRetainConf, cache: &Arc<FxSccHashMap<String, Point>>) -> Result<(), Error> {
        let mut snapshot = Vec::with_capacity(cache.len());
        cache.iter_sync(|key, point| {
            snapshot.push((key.clone(), point.clone()));
            true
        });
        let (tmp_path, path) = match conf.mode {
            RetainMode::Debug => (path.with_extension("json.tmp"), path.with_extension("json")),
            RetainMode::Release => (path.with_extension("dat.tmp"), path.with_extension("dat")),
        };
        let file = File::create(&tmp_path)
            .map_err(|err| err_pass!(dbg, err, "Can't open '{}'", tmp_path.display()))?;
        let mut tmp_writer = BufWriter::new(file);
        for (key, point) in snapshot {
            if let Err(err) = Self::append(&dbg, &mut tmp_writer, conf.mode, &key, &RetainState::from(&point)) {
                log::warn!("{}.store | Can't store '{}' into '{}', error: {:?}", dbg, key, tmp_path.display(), err);
            }
        }
        let file = tmp_writer.into_inner().map_err(|err| err_pass!(dbg, err, "Can't flush '{}'", tmp_path.display()))?;
        file.sync_data().map_err(|err| err_pass!(dbg, err, "Can't Sync '{}'", tmp_path.display()))?;
        drop(file);
        std::fs::rename(&tmp_path, &path).map_err(|err| err_pass!(dbg, err, "Can't Rename '{}' -> '{}'", tmp_path.display(), path.display()))?;
        log::trace!("{}.store | Compactation done to '{}'", dbg, path.display());
        Ok(())
    }
    ///
    /// ### Открывает новый файл
    /// 
    /// - Возвращает размер файла в байтах и указатель
    fn open(path: impl AsRef<Path>, conf: &FlushConf) -> Result<(u64, BufWriter<File>), std::io::Error> {
        let file = OpenOptions::new().append(true).create(true).open(&path)?;
        let size = file.metadata()?.len();
        Ok((size, BufWriter::with_capacity(conf.bytes_limit, file)))
    }
    ///
    /// ### Переименование неактивного файла в `.old`.
    /// Это необходимо что бы при следующей перезагрузке знать в каком режиме писали retain журнал
    /// 
    /// Возвращает `true` усли переименование успешно
    fn marck_inactive_old(dbg: &Dbg, path: &Path, mode: RetainMode) -> bool {
        let src_path = match mode {
            RetainMode::Debug => path.with_extension("dat"),
            RetainMode::Release => path.with_extension("json"),
        };
        let dst_path = src_path.with_added_extension("old");
        if let Err(err) = std::fs::rename(&src_path, &dst_path)
            .map_err(|err| err_pass!(dbg, err, "Can't rename '{}' -> '{}'", src_path.display(), dst_path.display())) {
            if let Err(err) = std::fs::remove_file(src_path).map_err(|err| err_pass!(dbg, err, "Can't remove '{}'", src_path.display())) {
                log::warn!("{}", err);
                return false;
            }
        }
        true
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
        let mut old_is_marked = false;
        let cache = self.cache.clone();
        let compactation_trigger = Trigger::new(Duration::from_hours(4)).with_mb_limit(conf.journal.compaction_limit_mb);
        let flush_trigger = Trigger::new(conf.journal.flush.interval);
        self.load().map_err(|err| err_pass!(dbg, err))?;
        let (mut file_size_bytes, mut writer) = Self::open(&path, &conf.journal.flush).map_err(|err| err_pass!(dbg, err))?;
        let ctx = RetainCtx {
            cache: self.cache.clone(),
            writer: todo!(),
            file_size_bytes: 0,
            compactation_trigger: super::compactation::Trigger::new(Duration::from_hours(4)).with_mb_limit(conf.journal.compaction_limit_mb),
            flush_trigger: super::compactation::Trigger::new(conf.journal.flush.interval),
            old_is_marked: false,
        };
        let retain = Compactation::new(&dbg, self.txid, &path, &conf,
            Append::new(&dbg, self.txid, &path, conf.mode,
                MarkObcolete::new(&dbg, self.txid, &path, conf.mode, 
                    LoadRetain::new(&dbg, self.txid, &path),
                )
            ),
        );
        match self.scheduler.as_ref() {
            Some(scheduler) => {
                let handle = scheduler.spawn({
                    let dbg = dbg.clone();
                    move || {
                    'main: while !exit.get() {
                        if compactation_trigger.is_exceeded(file_size_bytes) {
                            match Self::compactation(&dbg, &path, &conf, &cache, writer) {
                                Ok((size, w)) => {
                                    writer = w;
                                    file_size_bytes = size;
                                    compactation_trigger.start();
                                }
                                Err((w, err)) => {
                                    writer = w;
                                    log::warn!("{dbg}.run | Store error: {:?}", err);
                                }
                            }
                        }
                        match rx_recv.recv_timeout(RECV_TIMEOUT) {  // 100ms
                            Ok(event) => {
                                log::trace!("{dbg}.run | point '{}': {:?}", event.key, event.p);
                                let state = RetainState::from(&event.p);
                                if let Err(err) = Self::append(&dbg, &mut writer, conf.mode, &event.key, &state) {
                                    log::warn!("{dbg}.run | Can't store retain '{}' to '{}', error: {:?}", event.key, path.display(), err);
                                }
                                cache.insert_sync(event.key, event.p);
                            }
                            Err(RecvTimeoutError::Timeout) => {},
                            Err(err) => {
                                log::error!("{dbg}.run | Receiv error: {:?}", err);
                                break 'main;
                            }
                        }
                        if flush_trigger.is_exceeded(0u64) {
                            if !old_is_marked {
                                old_is_marked = Self::marck_inactive_old(&dbg, &path, conf.mode);
                            }
                            flush_trigger.start();
                            if let Err(err) = writer.flush() {
                                log::warn!("{dbg}.run | Can't flush to '{}', error: {:?}", path.display(), err);
                            }
                        }
                    }
                    let _ = writer.flush();
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
            None => {
                let handle = std::thread::spawn({
                    let dbg = dbg.clone();
                    move || {
                    'main: while !exit.get() {
                        match rx_recv.recv_timeout(RECV_TIMEOUT) {
                            Ok(event) => {
                                log::trace!("{dbg}.run | point '{}': {:?}", event.key, event.p);
                                cache.insert_sync(event.key, event.p);
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
                }});
                self.handles.push(handle);
                log::info!("{dbg}.run | Starting (Mock) - ok");
                Ok(())
            }
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
///
/// Cycle measuring
struct Trigger {
    interval: Duration,
    bytes_limit: u64,
    t: std::cell::Cell<Instant>,
}
//
impl Trigger {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            bytes_limit: 0,
            t: std::cell::Cell::new(Instant::now()),
        }
    }
    ///
    /// Maximum buffer length allowed before exceeded, MB.
    pub fn with_mb_limit(self, mb: impl Into<u64>) -> Self {
        Self {
            interval: self.interval,
            bytes_limit: mb.into() * 1024 * 1024,
            t: self.t,
        }
    }
    pub fn start(&self) {
        self.t.replace(Instant::now());
    }
    ///
    /// ### Returns `true` if time interval or bytes limit is exceeded
    /// - `bytes`: Current size in bytes
    pub fn is_exceeded(&self, bytes: impl Into<u64>) -> bool {
        if self.bytes_limit > 0 {
            return self.t.get().elapsed() >= self.interval || bytes.into() >= self.bytes_limit;
        }
        self.t.get().elapsed() >= self.interval
    }
}

///
/// 
struct CountingWriter<W: Write> {
    inner: W,
    bytes_written: usize,
}
//
impl<W: Write> CountingWriter<W> {
    pub fn new(inner: W) -> Self {
        Self { inner, bytes_written: 0 }
    }
    ///
    /// Получить текущее значение счетчика
    pub fn bytes_written(&self) -> usize {
        self.bytes_written
    }
    pub fn into_inner(self) -> W {
        self.inner
    }
}
//
impl<W: Write> Write for CountingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let result = self.inner.write(buf);
        if let Ok(bytes) = result {
            self.bytes_written += bytes;
        }
        result
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}
