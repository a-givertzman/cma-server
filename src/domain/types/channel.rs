pub type Sender<T> = sal_sync::sync::channel::Sender<T>;
pub type Receiver<T> = sal_sync::sync::channel::Receiver<T>;
pub type RecvTimeoutError = kanal::ReceiveErrorTimeout;
///
/// Creates a new sync bounded channel with the requested buffer size,
/// and returns Sender and Receiver of the channel for type T,
/// 
/// For channel with zero size queue, this channel always block until successful send/recv
pub fn bounded<T>(size: usize) -> (Sender<T>, Receiver<T>) {
    sal_sync::sync::channel::bounded(size)
}
///
/// Creates a new sync bounded channel with the requested buffer size,
/// and returns Sender and Receiver of the channel for type T,
pub fn unbounded<T>() -> (Sender<T>, Receiver<T>) {
    sal_sync::sync::channel::unbounded()
}
