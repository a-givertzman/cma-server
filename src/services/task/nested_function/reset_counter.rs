use std::sync::atomic::{AtomicUsize, Ordering};

///
/// Resets the AtomicXyz value to the given [val]
pub trait AtomicReset<T> {
    #[allow(dead_code)]
    ///
    /// Resets self to the given [val]
    fn reset(&self, val: T);
}

impl AtomicReset<usize> for AtomicUsize {
    fn reset(&self, val: usize) {
        self.store(val, Ordering::SeqCst)
    }
}

