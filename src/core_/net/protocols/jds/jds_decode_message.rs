use std::io::Read;
use log::trace;
use crate::{
    core_::net::{connection_status::{ConnectionStatus, SocketState}, protocols::jds::jds_define::JDS_END_OF_TRANSMISSION},
    tcp::tcp_stream_write::OpResult,
};
///
/// Reads bytes from TcpStream
/// splits bytes sequence with Jds.endOfTransmission = 4 separator
/// returns Result<Vec, Err>
#[derive(Debug)]
pub struct JdsDecodeMessage {
    id: String,
    // tcpStream: BufReader<TcpStream>,
    remainder: Vec<u8>,
}
//
// 
impl JdsDecodeMessage {
    ///
    /// Creates new instance of the JdsDecodeMessage
    pub fn new(parent: impl Into<String>) -> Self {
        Self {
            id: format!("{}/JdsDecodeMessage", parent.into()),
            // tcpStream: BufReader::new(tcpStream),
            remainder: Vec::new(),
        }
    }
    ///
    /// Reads sequence of bytes from TcpStream
    pub fn read(&mut self, tcp_stream: impl Read) -> ConnectionStatus<OpResult<Vec<u8>, String>, String> {
        let mut bytes = self.remainder.clone();
        match Self::read_all(&self.id, &mut bytes, tcp_stream) {
            ConnectionStatus::Active(result) => {
                match result {
                    OpResult::Ok(_) => {
                        self.remainder.clear();
                        ConnectionStatus::Active(OpResult::Ok(bytes))
                    }
                    OpResult::Err(err) => ConnectionStatus::Active(OpResult::Err(err)),
                    OpResult::Timeout() => ConnectionStatus::Active(OpResult::Timeout()),
                }
            }
            ConnectionStatus::Closed(err) => {
                if !bytes.is_empty() {
                    self.remainder = bytes;
                }
                ConnectionStatus::Closed(err)
            }
        }
    }
    ///
    /// bytes will be read from socket until JDS_END_OF_TRANSMISSION = 4
    /// - returns Active: if read bytes non zero length without errors
    /// - returns Closed:
    ///    - if read 0 bytes
    ///    - if on error
    fn read_all(self_id: &str, bytes: &mut Vec<u8>, stream: impl Read) -> ConnectionStatus<OpResult<(), String>, String> {
        for byte in stream.bytes() {
            match byte {
                Ok(byte) => {
                    // debug!("{}.read_all |     read len: {:?}", self_id, len);
                    match byte {
                        JDS_END_OF_TRANSMISSION => {
                            return ConnectionStatus::Active(OpResult::Ok(()));
                        }
                        _ => {
                            bytes.push(byte);
                        }
                    };
                }
                Err(err) => {
                    // warn!("{}.read_all | error reading from socket: {:?}", self_id, err);
                    // warn!("{}.read_all | error kind: {:?}", self_id, err.kind());
                    return match SocketState::match_error_kind(err.kind()) {
                        SocketState::Active => {
                            ConnectionStatus::Active(OpResult::Err(format!("{}.read_all | tcp stream is empty", self_id)))
                        }
                        SocketState::Closed => {
                            ConnectionStatus::Closed(format!("{}.read_all | tcp stream is closed, error: {:?}", self_id, err))
                        }
                        SocketState::Timeout => {
                            ConnectionStatus::Active(OpResult::Timeout())
                        }
                    }
                }
            };
        };
        trace!("{}.read_all | read bytes: {:?}", self_id, bytes);
        ConnectionStatus::Closed(format!("{}.read_all | tcp stream is closed", self_id))
    }
}
