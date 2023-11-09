#![allow(non_snake_case)]

use std::sync::mpsc::{Receiver, Sender, self};

use crate::core_::point::point_type::PointType;

///
/// - Holding single input queue
/// - Received string messages pops from the queue into the end of local buffer
/// - Sending messages (wrapped into ApiQuery) from the beginning of the buffer
/// - Sent messages immediately removed from the buffer
pub struct ApiClient {
    inQueue: Receiver<PointType>,
    send: Sender<PointType>,
}
///
/// 
impl ApiClient {
    pub fn new() -> Self {
        let (send, recv): (Sender<PointType>, Receiver<PointType>) = mpsc::channel();
        Self {
            inQueue: recv,
            send: send,
        }
    }
    ///
    /// 
    pub fn getLink(&self, _name: &str) -> Sender<PointType> {
        self.send.clone()
    }
}