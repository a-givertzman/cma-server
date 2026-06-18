use std::{io::Write, path::PathBuf, sync::Arc, time::{Duration, Instant}};
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{kernel::state::ExitNotify, services::{Service, Services, entity::{Name, Object, Point, PointConf}}, sync::{Handles, Owner}, thread_pool::Scheduler};
use crate::{domain::{FxSccHashMap, RECV_TIMEOUT, Receiver, RecvTimeoutError, Sender, bounded}, err, err_pass, services::task::{AppendJournal, CompactateJournal, FlushJournal, InitialCtx, LoadJournal, MarkOldJournal, OpenJournal, TaskRetainConf, retain::{Eval, RetainEvent}}};

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
#[allow(unused)]
impl TaskRetain {
    const BUFFER_SIZE: usize = 16 * 1024;
    ///
    /// Returns `TaskRetain` new instance
    /// - `parent` - Родительский сервис `Task`.
    /// - `txid` - Идентификатор сервиса отправителя (в данном случае родительского `Task`).
    #[named]
    pub fn new(parent: &Name, txid: usize, conf: TaskRetainConf, services: &Arc<Services>, scheduler: Scheduler) -> Result<Self, Error> {
        let name = Name::new(parent, crate::domain::me::<Self>());
        let dbg = Dbg::new(parent, crate::domain::me::<Self>());
        let Some(retain_path) = services.retain().path else {
            return Err(err!(dbg, "Retain: path - missed in Application config"));
        };
        let cw_dir = std::env::current_dir().map_err(|err| err_pass!(dbg, err))?;
        let dir = cw_dir.join(retain_path).join(parent.join().trim_start_matches('/'));
        std::fs::create_dir_all(&dir).map_err(|err| err_pass!(dbg, err, "Error creating dir: '{}'", dir.display()))?;
        let path = dir.join("retain").with_extension("json");
        let (send, recv) = bounded(Self::BUFFER_SIZE);
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
        let dbg = Dbg::new(name.parent(), crate::domain::me::<Self>());
        let (send, recv) = bounded(Self::BUFFER_SIZE);
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
        let ctx = InitialCtx::new(&dbg, self.txid, &conf, &self.path,
            LoadJournal::new(&dbg,
                MarkOldJournal::new(&dbg, conf.mode),
            ),
        ).eval(self.cache.clone())?;
        match self.scheduler.as_ref() {
            Some(scheduler) => {
                let handle = scheduler.spawn({
                    let dbg = dbg.clone();
                    let retain = OpenJournal::new(&dbg, &conf.journal.flush, ctx,
                        AppendJournal::new(&dbg, self.txid, conf.mode,
                            FlushJournal::new(&dbg,
                                CompactateJournal::new(&dbg, &conf),
                            ),
                        ),
                    );
                    move || {
                    'main: while !exit.get() {
                        match rx_recv.recv_timeout(RECV_TIMEOUT) {  // 100ms
                            Ok(event) => {
                                log::trace!("{dbg}.run | point '{}': {:?}", event.key, event.p);
                                if let Err(err) = retain.eval(Some(event)).map_err(|err| err_pass!(dbg, err)) {
                                    log::warn!("{err}");
                                }
                            }
                            Err(RecvTimeoutError::Timeout) => {
                                if let Err(err) = retain.eval(None).map_err(|err| err_pass!(dbg, err)) {
                                    log::warn!("{err}");
                                }
                            }
                            Err(err) => {
                                log::error!("{dbg}.run | Receiv error: {:?}", err);
                                break 'main;
                            }
                        }
                    }
                    if let Err(err) = retain.close() {
                        log::error!("{dbg}.run | Can't close retain journal: {:?}", err);
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
            None => {
                let handle = std::thread::spawn({
                    let dbg = dbg.clone();
                    let cache = self.cache.clone();
                    move || {
                    'main: while !exit.get() {
                        match rx_recv.recv_timeout(RECV_TIMEOUT) {
                            Ok(event) => {
                                log::trace!("{dbg}.run | point '{}': {:?}", event.key, event.p);
                                if let Err(err) = cache.insert_sync(event.key, event.p) {
                                    log::error!("{dbg}.run | Can't update retain cache: {:?}", err);
                                }
                            }
                            Err(RecvTimeoutError::Timeout) => {}
                            Err(err) => {
                                log::error!("{dbg}.run | Receiv error: {:?}", err);
                                break 'main;
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
    fn points(&self) -> Vec<PointConf> {
        self.conf.points()
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
///
/// Cycle measuring
#[allow(unused)]
struct Trigger {
    interval: Duration,
    bytes_limit: u64,
    t: std::cell::Cell<Instant>,
}
//
#[allow(unused)]
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
        if self.t.get().elapsed() >= self.interval {
            return true;
        }
        if self.bytes_limit > 0 {
            return bytes.into() >= self.bytes_limit;
        }
        false
    }
}
///
/// Wraps a writer and counts the total number of bytes written.
#[allow(unused)]
struct CountingWriter<W: Write> {
    inner: W,
    bytes_written: usize,
}
//
#[allow(unused)]
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
        match self.inner.write(buf) {
            Ok(bytes) => {
                self.bytes_written += bytes;
                Ok(bytes)
            }
            Err(err) => Err(err),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}
