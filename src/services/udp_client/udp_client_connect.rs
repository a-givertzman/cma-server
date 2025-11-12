//!
//! # Communication with Vibro-analytics microcontroller (Sub MC) over udp simple protocol
//! 
//! Default port number 15180
//! 
//! ## Message structure
//! 
//!     |Field name:   | FUN | ADDR | TYPE | COUNT | DATA        |
//!     |---           | --- | ---- | ---- | ----- | ----        |
//!     |Data type:    | u8  | u8   | u8   | u32   | [T; COUNT]    | 
//!     |Example value:| 22  | 0    | 16   | 512  | [u16; 512] |
//!     
//!     - `FUN` Functional byte, 
//!         - `0x22` - Initialization message
//!         - `0x02` - Data message
//!         - `0x05` - Command message
//!         - `0x07` - Error message
//!     - `ADDR` = 0...255 - Index of the input channel (0 - first input channel)
//!     - `TYPE` - type of values in the array in `DATA` field
//!         - 8 - u8, 1 byte unsigned integer value
//!         - 9 - i8, 1 byte signed integer value
//!         - 16 - u16, 2 byte unsigned integer value
//!         - 17 - i16, 2 byte signed integer value
//!         - 32 - u32, 4 byte unsigned integer value
//!         - 33 - i32, 4 byte signed integer value
//!         - 132 - f32, 4 bytes float value
//!     - `COUNT` - length of the array in the `DATA` field, number of values of type specified in the `TYPE` field
//!     - `DATA` - array of values of type specified in the `TYPE` field
//! 
//! ## Error codes
//!     `0x01` - System error
//!     `0x02` - ADC Error
//!     `0x03` - DMA Error
//!     `0x04` - Network error
//!     `...` - To be extended if necessary
//! 
//! ## Basic configuration parameters:
//! 
//! ```yaml
//! service UdpClientConnect Id:
//!     parameter: value    # meaning
//!     parameter: value    # meaning
//! ```

//! Message in the UDP has fallowing fiels
//! 
//! |Field name:   | SYN | ADDR | TYPE | COUNT | DATA        |
//! |---           | --- | ---- | ---- | ----- | ----        |
//! |Data type:    | u8  | u8   | u8   | u32   | u8[1024]    | 
//! |Example value:| 22  | 0    | 16   | 1024  | [u16; 1024] |
//! - `SYN` = 22 - message starts with
//! - `ADDR` = 0...255 - an address of the input channel (0 - first input channel)
//! - `TYPE` - type of values in the array in `DATA` field
//!     - 8 - 1 byte integer value
//!     - 16 - 2 byte float value
//!     - 32 - u16[1024] an array of 2 byte values of length 512
//! - `COUNT` - length of the array in the `DATA` field
//! - `DATA` - array of values of type specified in the `TYPE` field
//! 
use std::{net::UdpSocket, time::Duration};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::services::entity::{Name, Object};
use crate::services::udp_client::UdpClient;
///
/// 
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Units {
    Data,
    Unknown(String),
}
///
/// Establish a connection with Vibro-analytics microcontroller (Sub MC) over udp simple protocol
pub struct UdpClientConnect {
    name: Name,
    local_addr: String,
    remote_addr: String,
    mtu: usize,
    dbg: Dbg,
}
//
//
impl UdpClientConnect {
    ///
    /// Crteates new instance of the [UdpClientConnect] 
    pub fn new(parent: impl Into<String>, local_addr: String, remote_addr: String, mtu: usize) -> Self {
        let name = Name::new(parent, "UdpClientConnect");
        let dbg = Dbg::new(name.parent(), name.me());
        Self {
            name,
            local_addr,
            remote_addr,
            mtu,
            dbg,
        }
    }
    ///
    /// Returns the socket ready to receive data messages
    /// - Connected to the remote address
    /// - Hanshaked - Start message sent and acknowledged
    fn handshake(&self, socket: &UdpSocket) -> Result<(), Error> {
        let error = Error::new(&self.name, "handshake");
        let mut buf = vec![0; self.mtu];
        match socket.send_to(&[UdpClient::SYN, UdpClient::EOT], &self.remote_addr) {
            Ok(_) => {
                log::debug!("{}.handshake | Start message sent to'{}'", self.dbg, self.remote_addr);
                match socket.recv_from(&mut buf) {
                    Ok((_, src_addr)) => {
                        match buf.as_slice() {
                            // Start ACK received
                            &[UdpClient::SYN, UdpClient::EOT] | &[UdpClient::SYN, UdpClient::EOT, ..] => {
                                log::trace!("{}.handshake | {}: Start message ACK - Ok", self.dbg, src_addr);
                                Ok(())
                            }
                            // Unexpected Data message received, but Start message expected
                            &[UdpClient::DAT, _addr, _type_, _c1,_c2,_c3, _c4, ..] => {
                                Err(error.err(format!("Start message ACK expected, but Data message received: {:?}...", &buf[..=10])))
                            }
                            &[UdpClient::ERR, err] | &[UdpClient::ERR, err, ..] => {
                                Err(error.err(format!("Start message ACK expected, but error received: {:?}", err)))
                            }
                            // Empty message received
                            &[] => Err(error.err("Start message ACK expected, but empty message received")),
                            // Unknown message received
                            _ => Err(error.err(format!("Start message ACK expected, but unknown message received: {:?}", &buf[..=10]))),
                        }
                    }
                    Err(err) => {
                        // notify.add(State::UdpRecvError, format!("{}.handshake | UdpSocket recv error: {:#?}", self_id, err)),
                        match err.kind() {
                            std::io::ErrorKind::WouldBlock => Err(error.pass_with(format!("Socket read timeout"), err.to_string())),
                            std::io::ErrorKind::TimedOut => Err(error.pass_with(format!("Socket read timeout"), err.to_string())),
                            _ => Err(error.pass_with(format!("Socket error"), err.to_string())),
                        }
                    }
                }
            }
            Err(err) => Err(error.pass_with("Can't send Start message", err.to_string())),
        }
    }
    ///
    /// Returns the socket ready to receive data messages
    /// - Connected to the remote address
    /// - Hanshaked - Start message sent and acknowledged
    pub fn connect(&self) -> Result<UdpSocket, Error> {
        let error = Error::new(&self.name, "connect");
        match UdpSocket::bind(&self.local_addr) {
            Ok(socket) => {
                match socket.connect(&self.remote_addr) {
                    Ok(_) => {
                        if let Err(err) = socket.set_read_timeout(Some(Duration::from_millis(512))) {
                            log::error!("{}.connect | Socket Set read timeout error: {:?}", self.dbg, err);
                        }
                        if let Err(err) = socket.set_write_timeout(Some(Duration::from_millis(512))) {
                            log::error!("{}.connect | Socket Set write timeout error: {:?}", self.dbg, err);
                        }
                        match self.handshake(&socket) {
                            Ok(_) => Ok(socket),
                            Err(err) => Err(error.pass(err)),
                        }
                    }
                    Err(err) => {
                        Err(error.pass_with("UdpSocket.connect error", err.to_string()))
                    }
                }
            }
            Err(err) => Err(error.pass_with("UdpSocket::bind error", err.to_string()))
        }        
    }
}
//
//
impl Object for UdpClientConnect {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl std::fmt::Debug for UdpClientConnect {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UdpClientConnect")
            .field("id", &self.dbg)
            .finish()
    }
}
