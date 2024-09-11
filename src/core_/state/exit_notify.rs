use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
///
/// - Contains parents's [exit] signal
/// - Contains partner's [exit_pair] signal
/// - If [exit] is true, service exits main thread
/// - Rase [exit_pair] to true when partner service must exit main thread
pub struct ExitNotify {
    exit: Arc<AtomicBool>,
    exit_pair: Arc<AtomicBool>,
    id: String,
}
//
//
impl ExitNotify {
    ///
    /// Creates new instance of the ExitNotify
    pub fn new(
        parent: impl Into<String>,
        exit: Option<Arc<AtomicBool>>,
        exit_pair: Option<Arc<AtomicBool>>,
    ) -> Self {
        Self {
            id: format!("{}/ExitNotify", parent.into()),
            exit: exit.unwrap_or(Arc::new(AtomicBool::new(false))),
            exit_pair: exit_pair.unwrap_or(Arc::new(AtomicBool::new(false))),
        }
    }
    ///
    /// Returns true if exit signal exists
    pub fn get(&self) -> bool {
        self.exit.load(Ordering::SeqCst) || self.exit_pair.load(Ordering::SeqCst)
    }
    ///
    /// Returns true if exit signal exists on parent
    pub fn get_parent(&self) -> bool {
        self.exit.load(Ordering::SeqCst)
    }
    ///
    /// Sends exit signal to the parent
    pub fn exit_parent(&self) {
        self.exit_pair.store(true, Ordering::SeqCst);
    }
    ///
    /// Sends exit signal to the partner
    pub fn exit_pair(&self) {
        self.exit_pair.store(true, Ordering::SeqCst);
    }
    ///
    /// Sends exit signal to all
    pub fn exit_all(&self) {
        self.exit_pair.store(true, Ordering::SeqCst);
        self.exit.store(true, Ordering::SeqCst);
    }
    ///
    /// Resets all exit signals
    pub fn reset(&self) {
        self.exit_pair.store(false, Ordering::SeqCst);
        self.exit.store(false, Ordering::SeqCst);
    }
    ///
    /// Resets parent exit signal
    pub fn reset_parent(&self) {
        self.exit.store(false, Ordering::SeqCst);
    }
    ///
    /// Resets partner exit signal
    pub fn reset_pair(&self) {
        self.exit_pair.store(false, Ordering::SeqCst);
    }
}
