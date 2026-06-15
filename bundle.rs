mod initial_ctx {
use std::{path::{Path, PathBuf}, sync::Arc, time::Duration};
use function_name::named;
use sal_core::dbg::Dbg;
use sal_sync::services::entity::Point;
use crate::{domain::FxSccHashMap, err_pass, services::task::{RetainMode, TaskRetainConf, retain::EvalResult}};
use super::{Eval, RetainCtx};
pub struct InitialCtx<Child> {
    txid: usize,
    conf: TaskRetainConf,
    path: PathBuf,
    child: Child,
    dbg: Dbg,
}
impl<Child> InitialCtx<Child> {
    pub fn new(parent: impl Into<String>, txid: usize, conf: &TaskRetainConf, path: impl AsRef<Path>, child: Child) -> Self {
        let dbg = Dbg::new(parent, crate::domain::me::<Self>());
        Self {
            txid,
            conf: conf.clone(),
            path: path.as_ref().to_path_buf(),
            child,
            dbg,
        }
    }
}
impl<Child> Eval<Arc<FxSccHashMap<String, Point>>, EvalResult> for InitialCtx<Child>
where
    Child: Eval<RetainCtx, EvalResult>, {
    #[named]
    fn eval(&self, cache: Arc<FxSccHashMap<String, Point>>) -> EvalResult {
        let ctx = RetainCtx {
            txid: self.txid,
            path: match self.conf.mode {
                RetainMode::Debug => self.path.with_extension("json"),
                RetainMode::Release => self.path.with_extension("dat"),
            },
            cache,
            writer: None,
            file_size_bytes: 0,
            compactation_trigger: super::compactate_journal::Trigger::new(Duration::from_hours(4)).with_mb_limit(self.conf.journal.compaction_limit_mb),
            compacted: false,
            flush_trigger: super::compactate_journal::Trigger::new(self.conf.journal.flush.interval),
            error: None,
        };
        self.child.eval(ctx).map_err(|err| err_pass!(self.dbg, err))
    }
}
}
pub(super) use initial_ctx::*;
mod append_journal {
use std::{cell::Cell, collections::VecDeque, fs::File, io::{BufWriter, Write}};
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use serde::{Serialize, Serializer, ser::SerializeMap};
use crate::{err_pass, services::task::{RetainEvent, RetainMode, retain::RetainState}};
use super::{Eval, RetainCtx};
pub struct AppendJournal<Child> {
    txid: usize,
    mode: RetainMode,
    buffer: Cell<VecDeque<RetainEvent>>,
    child: Child,
    dbg: Dbg,
}
impl<Child> AppendJournal<Child> {
    pub fn new(parent: impl Into<String>, txid: usize, mode: RetainMode, child: Child) -> Self {
        let dbg = Dbg::new(parent, crate::domain::me::<Self>());
        Self {
            txid,
            mode,
            buffer: Cell::new(VecDeque::new()),
            child,
            dbg,
        }
    }
}
impl<Child> Eval<(Option<RetainEvent>, RetainCtx), RetainCtx> for AppendJournal<Child>
where
    Child: Eval<RetainCtx, RetainCtx>, {
    #[named]
    fn eval(&self, (event, mut ctx): (Option<RetainEvent>, RetainCtx)) -> RetainCtx {
        let mut buf = if ctx.compacted {
            ctx.compacted = false;
            VecDeque::new()
        } else {
            self.buffer.take()
        };
        if let Some(event) = event {
            ctx.cache.insert_sync(event.key.clone(), event.p.clone());
            buf.push_back(event);
        }
        if let Some(writer) = &mut ctx.writer {
            let mut not_retained = Vec::new();
            for (i, event) in buf.iter().enumerate() {
                let state = RetainState::from(&event.p);
                match append(&self.dbg, writer, &self.mode, &event.key, &state) {
                    Ok(bytes) => {
                        ctx.file_size_bytes = ctx.file_size_bytes + bytes as u64;
                    }
                    Err(err) => {
                        log::warn!("{}.run | Can't store retain '{}' to '{}', error: {:?}", self.dbg, event.key, ctx.path.display(), err);
                        not_retained.push(i);
                    }
                }
            }
            if not_retained.is_empty() {
                buf.clear();
            } else {
                let mut current_idx = 0;
                buf.retain(|_| {
                    let keep = not_retained.contains(&current_idx);
                    current_idx += 1;
                    keep
                });
            }
        }
        self.buffer.set(buf);
        let mut ctx = self.child.eval(ctx);
        ctx.error = ctx.error.map(|err| err_pass!(self.dbg, err));
        ctx
    }
}
#[named]
pub fn append<T: Serialize>(
    dbg: &Dbg,
    writer: &mut BufWriter<File>,
    mode: &RetainMode,
    key: &String,
    state: &T,
) -> Result<usize, Error> {
    match mode {
        RetainMode::Debug => {
            let mut writer = CountingWriter::new(writer);
            let mut serializer = serde_json::Serializer::new(&mut writer);
            let mut map = serializer.serialize_map(Some(1)).map_err(|err| err_pass!(dbg, err))?;
            map.serialize_entry(key, state).map_err(|err| err_pass!(dbg, err))?;
            map.end().map_err(|err| err_pass!(dbg, err))?;
            writer.write_all(b"\n").map_err(|err| err_pass!(dbg, err))?;
            Ok(writer.bytes_written())
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
struct CountingWriter<W: Write> {
    inner: W,
    bytes_written: usize,
}
impl<W: Write> CountingWriter<W> {
    pub fn new(inner: W) -> Self {
        Self { inner, bytes_written: 0 }
    }
    pub fn bytes_written(&self) -> usize {
        self.bytes_written
    }
    pub fn into_inner(self) -> W {
        self.inner
    }
}
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
}
pub(super) use append_journal::*;
mod compactate_journal {
use std::{fs::File, io::BufWriter, path::Path, sync::Arc, time::{Duration, Instant}};
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::services::entity::Point;
use crate::{domain::FxSccHashMap, err_pass, services::task::{RetainMode, TaskRetainConf, retain::{RetainCtx, RetainState}}};
use super::Eval;
pub struct CompactateJournal {
    conf: TaskRetainConf,
    dbg: Dbg,
}
impl CompactateJournal {
    pub fn new(parent: impl Into<String>, conf: &TaskRetainConf) -> Self {
        let dbg = Dbg::new(parent, crate::domain::me::<Self>());
        Self {
            conf: conf.clone(),
            dbg,
        }
    }
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
            if let Err(err) = super::append(dbg, &mut tmp_writer, &conf.mode, &key, &RetainState::from(&point)) {
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
}
impl Eval<RetainCtx, RetainCtx> for CompactateJournal {
    fn eval(&self, mut ctx: RetainCtx) -> RetainCtx {
        if ctx.compactation_trigger.is_exceeded(ctx.file_size_bytes) {
            match Self::store(&self.dbg, &ctx.path, &self.conf, &ctx.cache) {
                Ok(_) => {
                    ctx.compacted = true;
                    ctx.writer = None;
                    ctx.compactation_trigger.start();
                }
                Err(err) => {
                    log::warn!("{}.run | Store error: {:?}", self.dbg, err);
                }
            }
        }
        ctx
    }
}
pub struct Trigger {
    interval: Duration,
    bytes_limit: u64,
    t: std::cell::Cell<Instant>,
}
impl Trigger {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            bytes_limit: 0,
            t: std::cell::Cell::new(Instant::now()),
        }
    }
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
    pub fn is_exceeded(&self, bytes: impl Into<u64>) -> bool {
        if self.bytes_limit > 0 {
            return self.t.get().elapsed() >= self.interval || bytes.into() >= self.bytes_limit;
        }
        self.t.get().elapsed() >= self.interval
    }
}
impl Default for Trigger {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(1),
            bytes_limit: 512,
            t: std::cell::Cell::new(Instant::now()),
        }
    }
}}
pub(super) use compactate_journal::*;
mod flush_journal {
use std::io::Write;
use function_name::named;
use sal_core::dbg::Dbg;
use crate::err_pass;
use super::{Eval, RetainCtx};
pub struct FlushJournal<Child> {
    child: Child,
    dbg: Dbg,
}
impl<Child> FlushJournal<Child> {
    pub fn new(parent: impl Into<String>, child: Child) -> Self {
        let dbg = Dbg::new(parent, crate::domain::me::<Self>());
        Self {
            child,
            dbg,
        }
    }
}
impl<Child> Eval<RetainCtx, RetainCtx> for FlushJournal<Child>
where
    Child: Eval<RetainCtx, RetainCtx>,
{
    #[named]
    fn eval(&self, mut ctx: RetainCtx) -> RetainCtx {
        if ctx.flush_trigger.is_exceeded(0u64) {
            ctx.flush_trigger.start();
            if let Some(writer) = &mut ctx.writer {
                if let Err(err) = writer.flush() {
                    log::warn!("{}.run | Can't flush to '{:?}', error: {:?}", self.dbg, ctx.path.display(), err);
                }
            }
        }
        let mut ctx = self.child.eval(ctx);
        ctx.error = ctx.error.map(|err| err_pass!(self.dbg, err));
        ctx
    }
}
}
pub(super) use flush_journal::*;
mod load_journal {
use std::{fs::File, io::{BufRead, BufReader, Read}, path::Path, sync::Arc};
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::services::{entity::{Cot, Point, PointHlr}, types::Bool};
use crate::{domain::FxSccHashMap, err, err_pass, services::task::retain::{RetainState, RetainValue}};
use super::{EvalResult, Eval, RetainCtx};
enum IoState {
    Continue((String, RetainState)),
    Done,
}
pub struct LoadJournal<Child> {
    child: Child,
    dbg: Dbg,
}
impl<Child> LoadJournal<Child> {
    pub fn new(parent: impl Into<String>, child: Child) -> Self {
        let dbg = Dbg::new(parent, crate::domain::me::<Self>());
        Self {
            child,
            dbg,
        }
    }
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
    #[named]
    fn decode_entry(dbg: &Dbg, reader: &mut BufReader<File>, len_buf: &mut [u8; 4], key_buf: &mut Vec<u8>, state_buf: &mut Vec<u8>) -> Result<IoState, Error> {
        if reader.read_exact(len_buf).is_err() { return Ok(IoState::Done); }
        let len = u32::from_le_bytes(*len_buf);
        if len > 1024 {
            return Err(err!(dbg, "Размер ключа retain-записи: {} - превышает 1KB, файл кэша поврежден", len));
        }
        key_buf.resize(len as usize, 0u8);
        reader.read_exact(key_buf).map_err(|err| err_pass!(dbg, err))?;
        let key = std::str::from_utf8(key_buf).map_err(|err| err_pass!(dbg, err))?;
        reader.read_exact(len_buf).map_err(|err| err_pass!(dbg, err))?;
        let len = u32::from_le_bytes(*len_buf);
        if len > 10 * 1024 * 1024 {
            return Err(err!(dbg, "Размер значения retain-записи: {} - превышает 100MB, файл кэша поврежден", len));
        }
        state_buf.resize(len as usize, 0u8);
        reader.read_exact(state_buf).map_err(|err| err_pass!(dbg, err))?;
        let state = postcard::from_bytes::<RetainState>(state_buf).map_err(|err| err_pass!(dbg, err))?;
        Ok(IoState::Continue((key.to_owned(), state)))
    }
    #[named]
    fn load(&self, path: &Path, txid: usize, cache: &Arc<FxSccHashMap<String, Point>>) -> Result<(), Error> {
        let json_path = path.with_extension("json");
        let dat_path = path.with_extension("dat");
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
                    Ok(IoState::Continue((name, state))) => {
                        let val = Self::point(&state, txid, &name);
                        if let Err(err) = cache.insert_sync(name, val) {
                            log::warn!("{}.load | Can't extend cache: {:?}", self.dbg, err);
                        }
                    }
                    Err(err) => {
                        log::error!("{}.load | Retain файл журнала оборван или поврежден '{}'.\n\tОшибка: {:?}.\n\tТолько часть данных загружено: {:#?}.",
                            self.dbg, path.display(), err, cache);
                        break;
                    }
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
                                let val = Self::point(&state, txid, &key);
                                if let Err(err) = cache.insert_sync(key, val) {
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
}
impl<Child> Eval<RetainCtx, EvalResult> for LoadJournal<Child>
where
    Child: Eval<RetainCtx, EvalResult>, {
    #[named]
    fn eval(&self, ctx: RetainCtx) -> EvalResult {
        self.load(&ctx.path, ctx.txid, &ctx.cache).map_err(|err| err_pass!(self.dbg, err))?;
        self.child.eval(ctx).map_err(|err: Error| err_pass!(self.dbg, err))
    }
}
}
pub(super) use load_journal::*;
mod mark_old_journal {
use std::path::Path ;
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use crate::{err_pass, services::task::RetainMode };
use super::{EvalResult, Eval, RetainCtx};
pub struct MarkOldJournal {
    mode: RetainMode,
    dbg: Dbg,
}
impl MarkOldJournal {
    pub fn new(parent: impl Into<String>, mode: RetainMode) -> Self {
        let dbg = Dbg::new(parent, crate::domain::me::<Self>());
        Self {
            mode,
            dbg,
        }
    }
    #[named]
    fn mark_inactive_old(dbg: &Dbg, path: &Path, mode: RetainMode) -> Result<(), Error> {
        let src_path = match mode {
            RetainMode::Debug => path.with_extension("dat"),
            RetainMode::Release => path.with_extension("json"),
        };
        let dst_path = src_path.with_added_extension("old");
        if let Err(err) = std::fs::rename(&src_path, &dst_path) {
            if err.kind() != std::io::ErrorKind::NotFound {
                std::fs::remove_file(&src_path).map_err(|err| err_pass!(dbg, err, "Can't rename/remove '{}'", src_path.display()))?;
            }
        }
        Ok(())
    }
}
impl Eval<RetainCtx, EvalResult> for MarkOldJournal {
    fn eval(&self, ctx: RetainCtx) -> EvalResult {
        Self::mark_inactive_old(&self.dbg, &ctx.path, self.mode)?;
        Ok(ctx)
    }
}
}
pub(super) use mark_old_journal::*;
mod open_journal {
use std::{cell::RefCell, fs::OpenOptions, io::{BufWriter, Write}};
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use crate::{err, err_pass, services::task::{FlushConf, RetainEvent}};
use super::{Eval, RetainCtx};
pub struct OpenJournal<Child> {
    conf: FlushConf,
    ctx: RefCell<Option<RetainCtx>>,
    child: Child,
    dbg: Dbg,
}
impl<Child> OpenJournal<Child> {
    pub fn new(parent: impl Into<String>, conf: &FlushConf, ctx: RetainCtx, child: Child) -> Self {
        let dbg = Dbg::new(parent, crate::domain::me::<Self>());
        Self {
            conf: conf.clone(),
            ctx: RefCell::new(Some(ctx)),
            child,
            dbg,
        }
    }
    #[named]
    pub fn close(&self) -> Result<(), Error> {
        if let Some(ctx) = self.ctx.borrow_mut().as_mut() {
            if let Some(w) = &mut ctx.writer {
                w.flush().map_err(|err| err_pass!(self.dbg, err))?;
                w.get_ref().sync_all().map_err(|err| err_pass!(self.dbg, err))?;
            }
        }
        Ok(())
    }
}
impl<Child> Eval<Option<RetainEvent>, Result<(), Error>> for OpenJournal<Child>
where
    Child: Eval<(Option<RetainEvent>, RetainCtx), RetainCtx>, {
    #[named]
    fn eval(&self, event: Option<RetainEvent>) -> Result<(), Error> {
        match self.ctx.borrow_mut().as_mut() {
            Some(ctx) => {
                let path = ctx.path.clone();
                if ctx.writer.is_none() {
                    if let Some(parent) = std::path::Path::new(&path).parent() {
                        std::fs::create_dir_all(parent).map_err(|err| err_pass!(self.dbg, err))?;
                    }
                    let file = OpenOptions::new().append(true).create(true).open(&path).map_err(|err| err_pass!(self.dbg, err))?;
                    ctx.file_size_bytes = file.metadata()
                        .map_err(|err| err_pass!(self.dbg, err))?
                        .len();
                    ctx.writer = Some(BufWriter::with_capacity(self.conf.bytes_limit, file));
                }
            }
            None => return Err(err!(self.dbg, "Context not found")),
        }
        match self.ctx.replace(None) {
            Some(ctx) => {
                let mut ctx = self.child.eval((event, ctx));
                let result = ctx.error.take();
                self.ctx.replace(Some(ctx));
                match result {
                    Some(err) => Err(err_pass!(self.dbg, err)),
                    None => Ok(()),
                }
            }
            None => Err(err!(self.dbg, "Context not found")),
        }
    }
}
}
pub(super) use open_journal::*;
mod retain_state {
use sal_sync::services::entity::{Point, Status};
use serde::{Deserialize, Serialize};
#[derive(Clone)]
pub(crate) struct RetainEvent {
    pub key: String,
    pub p: Point,
}
impl RetainEvent {
    pub fn new(key: String, p: Point) -> Self {
        Self { key, p }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(super) enum RetainValue {
    Bool(bool),
    Int(i64),
    Real(f32),
    Double(f64),
    String(String),
    Bytes(Vec<u8>),
}
impl From<sal_sync::services::entity::Point> for RetainValue {
    fn from(p: sal_sync::services::entity::Point) -> Self {
        match p {
            sal_sync::services::entity::Point::Bool(p) => Self::Bool(p.value.0),
            sal_sync::services::entity::Point::Int(p) => Self::Int(p.value),
            sal_sync::services::entity::Point::Real(p) => Self::Real(p.value),
            sal_sync::services::entity::Point::Double(p) => Self::Double(p.value),
            sal_sync::services::entity::Point::String(p) => Self::String(p.value),
            sal_sync::services::entity::Point::Bytes(p) => Self::Bytes(p.value),
        }
    }
}
impl From<&sal_sync::services::entity::Point> for RetainValue {
    fn from(p: &sal_sync::services::entity::Point) -> Self {
        match p {
            sal_sync::services::entity::Point::Bool(p) => Self::Bool(p.value.0),
            sal_sync::services::entity::Point::Int(p) => Self::Int(p.value),
            sal_sync::services::entity::Point::Real(p) => Self::Real(p.value),
            sal_sync::services::entity::Point::Double(p) => Self::Double(p.value),
            sal_sync::services::entity::Point::String(p) => Self::String(p.value.clone()),
            sal_sync::services::entity::Point::Bytes(p) => Self::Bytes(p.value.clone()),
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub(super) struct RetainState {
    pub value: RetainValue,
    pub status: Status,
    pub ts: chrono::DateTime<chrono::Utc>,
}
impl From<&sal_sync::services::entity::Point> for RetainState {
    fn from(p: &sal_sync::services::entity::Point) -> Self {
        Self { value: RetainValue::from(p), status: p.status(), ts: p.timestamp() }
    }
}
impl From<sal_sync::services::entity::Point> for RetainState {
    fn from(p: sal_sync::services::entity::Point) -> Self {
        Self { status: p.status(), ts: p.timestamp(), value: RetainValue::from(p) }
    }
}
}
pub use retain_state::*;
mod task_retain_conf {
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::services::{conf::{ConfTree, ConfTreeGet}, entity::{Name, PointConf}};
use serde::{Deserialize, Serialize};
use std::{fs, time::Duration};
use crate::{err, err_pass, domain::me};
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum RetainMode {
    #[serde(alias = "debug")]
    Debug,
    #[serde(alias = "release")]
    Release,
}
impl Default for RetainMode {
    fn default() -> Self {
        Self::Release
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct JournalConf {
    pub flush: FlushConf,
    pub compaction_limit_mb: u64,
}
#[derive(Debug, Clone, PartialEq)]
pub struct FlushConf {
    pub bytes_limit: usize,
    pub interval: Duration,
}
#[derive(Debug, PartialEq, Clone)]
pub struct TaskRetainConf {
    pub name: Name,
    pub journal: JournalConf,
    pub mode: RetainMode,
}
impl TaskRetainConf {
    #[named]
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Result<TaskRetainConf, Error> {
        let me = me::<Self>();
        let parent = parent.into();
        let name = Name::new(&parent, me);
        let dbg = Dbg::new(parent, "TaskRetainConf");
        log::trace!("{}.new | conf: {:?}", dbg, conf);
        log::trace!("{}.new | name: {:?}", dbg, name);
        let mode = conf.get("mode").map(|v: serde_yaml::Value| serde_yaml::from_value(v)).unwrap_or(Ok(RetainMode::Release))
            .map_err(|err| err_pass!(dbg, err, "'mode' - wrong config, 'release' / 'debug' expected"))?;
        log::trace!("{}.new | mode: {:#?}", dbg, mode);
        Ok(TaskRetainConf {
            name,
            journal: JournalConf {
                flush: FlushConf {
                    bytes_limit: 16 * 1024,
                    interval: Duration::from_secs(16),
                },
                compaction_limit_mb: 32 * 1024 * 1024,
            },
            mode,
        })
    }
    #[named]
    pub(crate) fn from_yaml(parent: impl Into<String>, value: &serde_yaml::Value) -> Result<TaskRetainConf, Error> {
        let (key, value) = value.as_mapping().unwrap().into_iter().next()
            .ok_or_else(|| err!(Self, "Wrong or empty conf: {:#?}", value))?;
        let key = key.as_str().ok_or_else(|| err!(Self, "Wrong conf: {:#?}", value))?;
        Self::new(parent, ConfTree::new(key, value.clone()))
    }
    #[allow(unused)]
    #[named]
    pub fn read(parent: impl Into<String>, path: &str) -> Result<TaskRetainConf, Error> {
        let yaml_string = fs::read_to_string(path)
            .map_err(|err| err_pass!(Self, err, "Can't read file '{}'", path))?;
        let conf = serde_yaml::from_str(&yaml_string)
            .map_err(|err| err_pass!(Self, err, "Can't parse conf '{:?}'", yaml_string))?;
        TaskRetainConf::from_yaml(parent, &conf)
    }
    pub fn points(&self) -> Vec<PointConf> {
        vec![]
    }
}
impl Default for TaskRetainConf {
    fn default() -> Self {
        Self {
            name: Name::new("", crate::domain::me::<Self>()),
            journal: JournalConf {
                flush: FlushConf {
                    bytes_limit: 16 * 1024,
                    interval: Duration::from_secs(16),
                },
                compaction_limit_mb: 32 * 1024 * 1024,
            },
            mode: RetainMode::Release
        }
    }
}}
pub(super) use task_retain_conf::*;
mod task_retain {
use std::{io::Write, path::PathBuf, sync::Arc, time::{Duration, Instant}};
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{kernel::state::ExitNotify, services::{Service, Services, entity::{Name, Object, Point, PointConf}}, sync::{Handles, Owner}, thread_pool::Scheduler};
use crate::{domain::{FxSccHashMap, RECV_TIMEOUT, Receiver, RecvTimeoutError, Sender, bounded}, err, err_pass, services::task::{AppendJournal, CompactateJournal, FlushJournal, InitialCtx, LoadJournal, MarkOldJournal, OpenJournal, TaskRetainConf, retain::{Eval, RetainEvent, RetainState}}};
enum IoState {
    Continue((String, RetainState)),
    Done,
}
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
impl TaskRetain {
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
    pub fn mock(parent: impl Into<String>, cache: impl IntoIterator<Item = (String, Point)>) -> Self {
        let name = Name::new(parent, crate::domain::me::<Self>());
        let dbg = Dbg::new(name.parent(), crate::domain::me::<Self>());
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
    pub fn link(&self) -> Sender<RetainEvent> {
        self.send.clone()
    }
    pub fn get(&self, key: &str) -> Option<Point> {
        self.cache.read_sync(key, |_, p| p.clone())
    }
}
impl Object for TaskRetain {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
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
                            },
                            Err(err) => {
                                log::error!("{dbg}.run | Receiv error: {:?}", err);
                                break 'main;
                            }
                        }
                    }
                    retain.close();
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
}
pub use task_retain::*;
pub(self) type EvalResult = Result<RetainCtx, sal_core::error::Error>;
///
/// `TaskRetain` evaluation
pub(self) trait Eval<In, Out> {
    fn eval(&self, _: In) -> Out;
}
///
/// Context provides tranfer data in the `TaskRetain` evaluation
pub(self) struct RetainCtx {
    txid: usize,
    cache: std::sync::Arc<crate::domain::FxSccHashMap<String, sal_sync::services::entity::Point>>,
    path: std::path::PathBuf,
    writer: Option<std::io::BufWriter<std::fs::File>>,
    file_size_bytes: u64,
    compactation_trigger: compactate_journal::Trigger,
    /// Весь retain cache только что был записан надиск, необходимо очистить буфер в `AppendJournal`
    compacted: bool,
    flush_trigger: compactate_journal::Trigger,
    error: Option<sal_core::error::Error>,
}
