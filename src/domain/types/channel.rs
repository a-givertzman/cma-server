pub type Sender<T> = sal_sync::sync::channel::Sender<T>;
pub type Receiver<T> = sal_sync::sync::channel::Receiver<T>;
pub type RecvTimeoutError = kanal::ReceiveErrorTimeout;