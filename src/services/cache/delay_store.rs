use std::time::{Duration, Instant};

///
/// Counts number of next call, 
/// - next returns None if presset count exceeded, then repeat
pub struct DelyStore {
    delay: Duration,
    time: Instant,
    stored: bool,
}
//
// 
impl DelyStore {
    ///
    /// Creates new instance of the [DelyStore]
    ///  - delay - Duration to delay store operation
    pub fn new(delay: Duration) -> Self {
        Self { delay, time: Instant::now(), stored: false }
    }
    ///
    /// Retirns 'true' if delay is exceeded - it's time to store, otherwisw do nothing
    /// - Also 'is_stored' flag will be reseted to false
    pub fn exceeded(&mut self) -> bool {
        self.stored = false;
        if self.time.elapsed() >= self.delay {
            self.time = Instant::now();
            true
        } else {
            false
        }
    }
    ///
    /// Retorns current state of 'is_stored' flag
    pub fn stored(&self) -> bool {
        self.stored
    }
    ///
    /// Sets current state of 'is_stored' flag to true
    pub fn set_stored(&mut self) {
        self.stored = true;
    }
}