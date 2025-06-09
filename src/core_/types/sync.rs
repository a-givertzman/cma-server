pub type RwLock<T> = parking_lot::RwLock<T>;
pub type Mutex<T> = parking_lot::Mutex<T>;
pub type Sender<T> = kanal::Sender<T>;
pub type Receiver<T> = kanal::Receiver<T>;