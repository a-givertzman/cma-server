use std::{time::{Duration, Instant}, thread::{JoinHandle, self}, sync::{Arc, atomic::{AtomicBool, Ordering}}};
use log::error;
///
/// If maximum test turation will be exceeded - the panics throwed
pub struct LockTimer {
    id: String,
    type_: String,
    duration: Duration,
    exit: Arc<AtomicBool>,
}
//
// 
impl LockTimer {
    ///
    /// If maximum lock duration exceeded - error message printed
    /// - parent: String - name of the parent entity
    /// - duration - maximum lock duration
    pub fn new(parent: impl Into<String>, type_: impl Into<String>, duration: Duration) -> Self {
        Self {
            id: format!("{}/LockTimer", parent.into()),
            type_: type_.into(),
            duration,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// The countdown begins, exiting process with error code 70 / error message - if duration exceeded
    pub fn run(&self) -> Result<JoinHandle<()>, std::io::Error> {
        let self_id = self.id.clone();
        let type_ = self.type_.clone();
        let exit = self.exit.clone();
        let duration = self.duration;
        thread::Builder::new().name(format!("{}.run", self_id)).spawn(move || {
            let timer = Instant::now();
            loop {
                if exit.load(Ordering::SeqCst) {
                    break;
                }
                thread::sleep(Duration::from_millis(100));
                if exit.load(Ordering::SeqCst) {
                    break;
                }
                if timer.elapsed() > duration {
                    error!("{}.run | Maximum lock duration ({:?}) exceeded for type: '{}'", self_id, duration, type_);
                    // std::process::exit(80);   // SOFTWARE: ExitCode = 80
                }

            }
        })
    }
    ///
    /// Normal completion, must be called before the duration has expired
    pub fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }
}