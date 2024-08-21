use std::{fmt::Debug, io::Write, net::TcpStream};
use log::{trace, warn, LevelFilter};
use crate::{
    tcp::steam_read::StreamRead, 
    core_::{retain_buffer::retain_buffer::RetainBuffer, net::connection_status::ConnectionStatus, failure::recv_error::RecvError},
};

///
/// Received from in queue sequences of bites adds into the end of local buffer
/// Sends sequences of bites from the beginning of the buffer
/// Sent sequences of bites immediately removed from the buffer
/// Buffering - is optional
pub struct TcpStreamWrite {
    id: String,
    stream: Box<dyn StreamRead<Vec<u8>, RecvError> + Send>,
    buffer: RetainBuffer<Vec<u8>>,
}
//
// 
impl TcpStreamWrite {
    ///
    /// Creates new instance of [TcpStreamWrite]
    pub fn new(parent: impl Into<String>, buffered: bool, buffer_length: Option<usize>, stream: Box<dyn StreamRead<Vec<u8>, RecvError> + Send>) -> Self {
        let self_id = format!("{}/TcpStreamWrite", parent.into());
        let buffer = match buffered {
            true => RetainBuffer::new(&self_id, "", buffer_length),
            false => RetainBuffer::new(&self_id, "", Some(0))
        };
        Self {
            id: self_id,
            stream,
            buffer,
        }
    }
    ///
    /// 
    pub fn write(&mut self, mut tcp_stream: &TcpStream) -> ConnectionStatus<OpResult<(), String>, String> {
        match self.stream.read() {
            Ok(bytes) => {
                while let Some(bytes) = self.buffer.first() {
                    trace!("{}.write | bytes: {:?}", self.id, bytes);
                    match tcp_stream.write_all(bytes) {
                        Ok(_) => {
                            self.buffer.pop_first();
                        }
                        Err(err) => {
                            let message = format!("{}.write | error: {:?}", self.id, err);
                            if log::max_level() == LevelFilter::Debug {
                                warn!("{}", message);
                            }
                            return ConnectionStatus::Closed(message);
                        }
                    };
                }
                trace!("{}.write | bytes: {:?}", self.id, bytes);
                match tcp_stream.write_all(&bytes) {
                    Ok(_) => {
                        match tcp_stream.flush() {
                            Ok(_) => {
                                ConnectionStatus::Active(OpResult::Ok(()))
                            }
                            Err(err) => {
                                self.buffer.push(bytes);
                                let message = format!("{}.write | error: {:?}", self.id, err);
                                if log::max_level() == LevelFilter::Debug {
                                    warn!("{}", message);
                                }
                                ConnectionStatus::Closed(message)
                            }
                        }
                    }
                    Err(err) => {
                        self.buffer.push(bytes);
                        let message = format!("{}.write | error: {:?}", self.id, err);
                        if log::max_level() == LevelFilter::Debug {
                            warn!("{}", message);
                        }
                        ConnectionStatus::Closed(message)
                    }
                }
            }
            Err(err) => {
                match err {
                    RecvError::Error(err) => {
                        let message = format!("{}.write | error: {:?}", self.id, err);
                        if log::max_level() == LevelFilter::Trace {
                            warn!("{}", message);
                        }
                        ConnectionStatus::Active(OpResult::Err(message))
                    }
                    RecvError::Disconnected => {
                        let message = format!("{}.write | channel disconnected, error: {:?}", self.id, err);
                        warn!("{}", message);
                        ConnectionStatus::Active(OpResult::Err(message))
                    }
                    RecvError::Timeout => ConnectionStatus::Active(OpResult::Timeout()),
                }
            }
        }
    }
}
//
// 
impl Debug for TcpStreamWrite {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TcpStreamWrite")
            .field("id", &self.id)
            .finish()
    }
}
///
/// 
unsafe impl Sync for TcpStreamWrite {}
///
/// 
#[derive(Debug)]
pub enum OpResult<T, E> {
    Ok(T),
    Err(E),
    Timeout(),
}