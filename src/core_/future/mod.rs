use std::sync::mpsc::{Receiver, Sender};
use log::error;
use super::types::type_of::TypeOf;
///
/// Contains future callback
pub struct Future<T> {
    recv: Receiver<T>
    // exec: Box<dyn Fn() -> T>,
    // then: Box<dyn Fn(T) -> T>,
}
//
//
impl<T> Future<T> {
    ///
    /// 
    pub fn new() -> (Self, Sink<T>) {
        let (send, recv) = std::sync::mpsc::channel();
        (
            Self { recv },
            Sink::new(send),
        )
    }
    /// 
    /// Returns future callback
    pub fn then(&self, on_done: impl Fn(T) -> T, on_err: impl Fn(String) -> T) -> T {
        match self.recv.recv() {
            Ok(event) => {
                (on_done)(event)
            }
            Err(err) => {
                (on_err)(format!("{}.then | Error: {:#?}", self.type_of(), err))
            }
        }
    }
}
///
/// Contains Sender<T>
pub struct Sink<T> {
    send: Sender<T>,
}
//
//
impl<T> Sink<T> {
    ///
    /// Creates new instance of Sink<T>
    pub fn new(send: Sender<T>) -> Self {
        Self {
            send: send,
        }
    }
    ///
    /// Sends value th the corresponding Future<T>
    pub fn add(&self, value: T) {
        match self.send.send(value) {
            Ok(_) => {}
            Err(err) => error!("Sink.add | Send error: {:#?}", err),
        }
    }
}
