use std::{path::{Path, PathBuf}, sync::Arc, time::Duration};
use function_name::named;
use sal_core::dbg::Dbg;
use sal_sync::services::entity::Point;
use crate::{domain::FxSccHashMap, err_pass, services::task::{RetainMode, TaskRetainConf, retain::EvalResult}};
use super::{Eval, RetainCtx};

///
/// Создает стартовый `RetainCtx` и передает его дальше по конвейеру
pub struct InitialCtx<Child> {
    txid: usize,
    conf: TaskRetainConf,
    path: PathBuf,
    child: Child,
    dbg: Dbg,
}
//
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
//
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
