use crate::domain::Receiver;

///
/// Receives events from channel, stores and returns las event
pub struct LastEvent<T> {
    val: Receiver<T>,
}
//
//
impl<T> LastEvent<T> {
    pub fn new(val: Receiver<T>) -> Self {
        Self { val }
    }
    pub fn last(&self) -> T {
        self.val.peekable().
    }
}