mod initial_ctx;
pub(super) use initial_ctx::*;
mod append_journal;
pub(super) use append_journal::*;
mod compactate_journal;
pub(super) use compactate_journal::*;
mod flush_journal;
pub(super) use flush_journal::*;
mod load_journal;
pub(super) use load_journal::*;
mod mark_old_journal;
pub(super) use mark_old_journal::*;
mod open_journal;
pub(super) use open_journal::*;

mod retain_state;
pub use retain_state::*;
mod task_retain_conf;
pub(super) use task_retain_conf::*;
mod task_retain;
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
