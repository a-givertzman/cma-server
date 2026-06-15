use std::{fs::File, io::BufWriter, path::Path, sync::Arc, time::{Duration, Instant}};
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::services::entity::Point;
use crate::{domain::FxSccHashMap, err_pass, services::task::{RetainMode, TaskRetainConf, retain::{RetainCtx, RetainState}}};
use super::Eval;

///
/// ### Физическая запись состояния на диск (Compactation).
/// Пишется через атомарную подмену файлов
pub struct CompactateJournal {
    conf: TaskRetainConf,
    dbg: Dbg,
}
//
impl CompactateJournal {
    pub fn new(parent: impl Into<String>, conf: &TaskRetainConf) -> Self {
        let dbg = Dbg::new(parent, crate::domain::me::<Self>());
        Self {
            conf: conf.clone(),
            dbg,
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
//
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
///
/// Detection of interval or size exceeded
pub struct Trigger {
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
//
impl Default for Trigger {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(1),
            bytes_limit: 512,
            t: std::cell::Cell::new(Instant::now()),
        }
    }
}