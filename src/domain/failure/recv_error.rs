///
/// Used for return result of the mpsc::Receiver.read()
#[derive(Debug)]
pub enum RecvError {
    Error(String),
    Timeout,
    Disconnected,
}