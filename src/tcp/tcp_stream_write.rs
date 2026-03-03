use std::{fmt::Debug, io::Write, net::TcpStream};
use log::LevelFilter;
use sal_sync::services::entity::Point;
use crate::{
    tcp::steam_read::StreamRead, 
    domain::{retain_buffer::retain_buffer::RetainBuffer, net::connection_status::ConnectionStatus, failure::RecvError},
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
                    log::debug!("{}.write | bytes[{}] to be sent: {:?}", self.id, bytes.len(), serde_json::from_slice::<Point>(&bytes[1..]));
                    log::trace!("{}.write | bytes: {:?}", self.id, bytes);
                    match tcp_stream.write_all(bytes) {
                        Ok(_) => {
                            self.buffer.pop_first();
                        }
                        Err(err) => {
                            let message = format!("{}.write | error: {:?}", self.id, err);
                            if log::max_level() == LevelFilter::Debug {
                                log::warn!("{}", message);
                            }
                            return ConnectionStatus::Closed(message);
                        }
                    };
                }
                log::trace!("{}.write | bytes: {:?}", self.id, bytes);
                match tcp_stream.write_all(&bytes) {
                    Ok(_) => {
                        match tcp_stream.flush() {
                            Ok(_) => {
                                log::debug!("{}.write | bytes[{}] sent", self.id, bytes.len());
                                ConnectionStatus::Active(OpResult::Ok(()))
                            }
                            Err(err) => {
                                self.buffer.push(bytes);
                                let message = format!("{}.write | error: {:?}", self.id, err);
                                if log::max_level() == LevelFilter::Debug {
                                    log::warn!("{}", message);
                                }
                                ConnectionStatus::Closed(message)
                            }
                        }
                    }
                    Err(err) => {
                        self.buffer.push(bytes);
                        let message = format!("{}.write | error: {:?}", self.id, err);
                        if log::max_level() == LevelFilter::Debug {
                            log::warn!("{}", message);
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
                            log::warn!("{}", message);
                        }
                        ConnectionStatus::Active(OpResult::Err(message))
                    }
                    RecvError::Disconnected => {
                        let message = format!("{}.write | channel disconnected, error: {:?}", self.id, err);
                        log::warn!("{}", message);
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
/// Result of operation with Ok / Err / Timeout  
/// Used for example for tcp_stream.read, when read timeout is specified for tcp_stream
#[derive(Debug)]
pub enum OpResult<T, E> {
    Ok(T),
    Err(E),
    Timeout(),
}