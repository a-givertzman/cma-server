use std::{net::TcpStream, io::{Write, BufReader, Read}, sync::atomic::AtomicBool};
use log::{warn, LevelFilter};
use crate::core_::net::{connection_status::ConnectionStatus, protocols::jds::jds_define::JDS_END_OF_TRANSMISSION};
///
/// Wraper for the TcpStream
/// - reads all available bytes from stream
struct TcpSocket {
    id: String,
    tcpStreamR: BufReader<TcpStream>,
    tcpStreamW: TcpStream,
    isConnected: AtomicBool,
}
//
// 
impl TcpSocket {
    pub fn new(parent: impl Into<String>, tcpStream: TcpStream) -> Self {
        Self {
            id: format!("{}/JdsMessage", parent.into()),
            tcpStreamR: BufReader::new(tcpStream.try_clone().unwrap()),
            tcpStreamW: tcpStream,
            isConnected: AtomicBool::new(false),
        }
    }
    ///
    /// 
    pub fn read(&mut self) -> Result<Vec<u8>, String> {
        let mut bytes = Vec::new();
        match Self::readAll(&self.id, &mut bytes, &mut self.tcpStreamR) {
            ConnectionStatus::Active(_) => {
                Ok(bytes)
            }
            ConnectionStatus::Closed(err) => {
                Err(err)
            }
        }
    }
    /// 
    pub fn write(&mut self, bytes: &[u8]) -> Result<usize, String> {
        match &self.tcpStreamW.write(bytes) {
            Ok(len) => {
                Ok(*len)
            }
            Err(err) => {
                let message = format!("{}.write | error: {:?}", self.id, err);
                if log::max_level() == LevelFilter::Trace {
                    warn!("{}", message);
                }
                Err(message)
            }
        }
    }
    ///
    /// bytes will be read from socket until JDS_END_OF_TRANSMISSION = 4
    /// - returns Active: if read bytes non zero length without errors
    /// - returns Closed:
    ///    - if read 0 bytes
    ///    - if on error
    fn readAll(self_id: &str, bytes: &mut Vec<u8>, stream: &mut BufReader<TcpStream>) -> ConnectionStatus<(), String> {
        for byte in stream.bytes() {
            match byte {
                Ok(byte) => {
                    // debug!("{}.readAll |     read len: {:?}", self_id, len);
                    match byte {
                        JDS_END_OF_TRANSMISSION => {
                            return ConnectionStatus::Active(());
                        }
                        _ => {
                            bytes.push(byte);
                        }
                    };
                }
                Err(err) => {
                    warn!("{}.readAll | error reading from socket: {:?}", self_id, err);
                    warn!("{}.readAll | error kind: {:?}", self_id, err.kind());
                    // Self::matchErrorKind(err.kind());
                    return ConnectionStatus::Closed(format!("{}.readAll | tcp socket error : {:?}", self_id, err))
                }
            };
        };
        ConnectionStatus::Active(())
    }
    // ///
    // /// 
    // fn matchErrorKind(kind: ErrorKind) -> Status {
    //     match kind {
    //         std::io::ErrorKind::NotFound => todo!(),
    //         std::io::ErrorKind::PermissionDenied => Status::Closed,
    //         std::io::ErrorKind::ConnectionRefused => Status::Closed,
    //         std::io::ErrorKind::ConnectionReset => Status::Closed,
    //         // std::io::ErrorKind::HostUnreachable => Status::Closed,
    //         // std::io::ErrorKind::NetworkUnreachable => Status::Closed,
    //         std::io::ErrorKind::ConnectionAborted => Status::Closed,
    //         std::io::ErrorKind::NotConnected => Status::Closed,
    //         std::io::ErrorKind::AddrInUse => Status::Closed,
    //         std::io::ErrorKind::AddrNotAvailable => Status::Closed,
    //         // std::io::ErrorKind::NetworkDown => Status::Closed,
    //         std::io::ErrorKind::BrokenPipe => Status::Closed,
    //         std::io::ErrorKind::AlreadyExists => todo!(),
    //         std::io::ErrorKind::WouldBlock => Status::Closed,
    //         // std::io::ErrorKind::NotADirectory => todo!(),
    //         // std::io::ErrorKind::IsADirectory => todo!(),
    //         // std::io::ErrorKind::DirectoryNotEmpty => todo!(),
    //         // std::io::ErrorKind::ReadOnlyFilesystem => todo!(),
    //         // std::io::ErrorKind::FilesystemLoop => todo!(),
    //         // std::io::ErrorKind::StaleNetworkFileHandle => todo!(),
    //         std::io::ErrorKind::InvalidInput => todo!(),
    //         std::io::ErrorKind::InvalidData => todo!(),
    //         std::io::ErrorKind::TimedOut => todo!(),
    //         std::io::ErrorKind::WriteZero => todo!(),
    //         // std::io::ErrorKind::StorageFull => todo!(),
    //         // std::io::ErrorKind::NotSeekable => todo!(),
    //         // std::io::ErrorKind::FilesystemQuotaExceeded => todo!(),
    //         // std::io::ErrorKind::FileTooLarge => todo!(),
    //         // std::io::ErrorKind::ResourceBusy => todo!(),
    //         // std::io::ErrorKind::ExecutableFileBusy => todo!(),
    //         // std::io::ErrorKind::Deadlock => todo!(),
    //         // std::io::ErrorKind::CrossesDevices => todo!(),
    //         // std::io::ErrorKind::TooManyLinks => todo!(),
    //         // std::io::ErrorKind::InvalidFilename => todo!(),
    //         // std::io::ErrorKind::ArgumentListTooLong => todo!(),
    //         std::io::ErrorKind::Interrupted => todo!(),
    //         std::io::ErrorKind::Unsupported => todo!(),
    //         std::io::ErrorKind::UnexpectedEof => todo!(),
    //         std::io::ErrorKind::OutOfMemory => todo!(),
    //         std::io::ErrorKind::Other => todo!(),
    //         _ => Status::Closed,
    //     }
    // }    
}