#![allow(non_snake_case)]

use std::sync::mpsc::Sender;

use crate::core_::failure::error_string::ErrorString;

use super::queue_send::QueueSend;

///
/// Holds a reference to the specific async queue / channel implementation
/// Sharing standard interface to send data into inner queueu
#[derive(Debug)]
pub struct QueueSendMpscChannel<T> {
    send: Sender<T>
}
//
// 
impl<T> QueueSendMpscChannel<T> {
    pub fn new(send: Sender<T>) -> Self {
        Self {
            send,
        }
    }
}



impl<T: std::fmt::Debug> QueueSend<T> for QueueSendMpscChannel<T> {
    fn send(&mut self, value: T) -> Result<(), ErrorString> {
        match self.send.send(value) {
            Ok(_) => Ok(()),
            Err(err) => Err(format!("QueueSendMpscChannel.send | send error: {:?}", err)),
        }
    }
}