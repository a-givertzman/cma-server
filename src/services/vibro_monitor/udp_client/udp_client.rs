use std::{cell::RefCell, fs, io::Write, net::UdpSocket, sync::{Arc, atomic::{AtomicBool, Ordering}}};
use chrono::{DateTime, Utc};
use concat_string::concat_string;
use function_name::named;
use indexmap::IndexMap;
use sal_core::{dbg::Dbg, error::{Error, ErrorLimit}};
use sal_sync::{
    kernel::state::ChangeNotify,
    services::{Service, ServiceCycle, Services, entity::{
        Name, Object,
    }}, sync::{Handles, channel::Sender}, thread_pool::Scheduler
};
use crate::{err, err_pass};

use super::{InputType, UdpClientConnect, UdpClientConf};
///
/// 
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum State {
    None,
    Start,
    Exit,
    ReadError,
    ConnectError,
    Connected,
}
///
/// Reads data from Vibro-analytics microcontroller (Sub MC)
/// 
/// - **Message structure**
/// 
///     |Field name:   | FUN | CHANNELS | TYPE | COUNT | DATA        |
///     |---           | --- | ----     | ---- | ----- | ----        |
///     |Data type:    | u8  | u8       | u8   | u32   | [T; COUNT]  | 
///     |Example value:| 22  | 0        | 16   | 512   | [u16; 512]  |
///     
///     - `FUN` Functional byte, 
///         - `0x22` - Initialization message
///         - `0x02` - Data message
///         - `0x05` - Command message
///         - `0x07` - Error message
///     - `CHANNELS` = 0...255 - Index of the input channel (0 - first input channel)
///     - `TYPE` - type of values in the array in `DATA` field
///         - 8 - u8, 1 byte unsigned integer value
///         - 9 - i8, 1 byte signed integer value
///         - 16 - u16, 2 byte unsigned integer value
///         - 17 - i16, 2 byte signed integer value
///         - 32 - u32, 4 byte unsigned integer value
///         - 33 - i32, 4 byte signed integer value
///         - 132 - f32, 4 bytes float value
///     - `COUNT` - length of the `DATA` field in bytes
///     - `DATA` - array of values of type specified in the `TYPE` field
/// 
/// - **Error codes**
///     `0x01` - System error
///     `0x02` - ADC Error
///     `0x03` - DMA Error
///     `0x04` - Network error
///     `...` - To be extended if necessary
pub struct UdpClient {
    name: Name,
    conf: UdpClientConf,
    /// Связь с устройством по сети
    socket: RefCell<Option<UdpSocket>>,
    /// Буфер для чтения из сокета
    buff: RefCell<Vec<u8>>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
//
//
impl UdpClient {
    /// Message starts with
    pub const SYN: u8 = 0x22;
    /// Start message ends with
    pub const EOT: u8 = 0x04;
    pub const DAT: u8 = 0x02;
    pub const CMD: u8 = 0x05;
    pub const ERR: u8 = 0x07;
    /// Message header length in bytes
    pub const HEAD_LEN: usize = 7;
    /// Sample size, `2 bytes` for `u16`
    const SAMPLE_SIZE: usize = 2;
    ///
    /// Creates new instance of the [UdpClient]
    /// - app - string represents application name, for point path
    /// - parent - parent id, used for debugging
    /// - conf - configuration of the [UdpClient]
    pub fn new(parent: impl Into<String>, conf: UdpClientConf) -> Self {
        let name = Name::new(parent, crate::me::<Self>());
        let dbg = Dbg::new(name.parent(), name.me());
        let mtu = conf.mtu;
        Self {
            name,
            conf,
            socket: RefCell::new(None),
            buff: RefCell::new(vec![0; mtu]),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
    ///
    /// Returns `u16` values converted from `butes` or `Err`
    #[inline]
    fn convert(&self, channel: usize, channels: usize, bytes: &[u8], values: &mut Vec<u16>) -> Result<(), Error> {
        log::trace!("{}.convert | bytes: {:?}", self.dbg, bytes);
        if bytes.is_empty() {
            return Err(Error::new(&self.name, "convert").err("Input is empty"));
        }
        let (words, remainder) = bytes.as_chunks::<{ Self::SAMPLE_SIZE }>();
        log::trace!("{}.convert | words: {:?}", self.dbg, words.len());
        if remainder.len() > 0 {
            log::warn!("{}. convert | Wrong input len {}, must be divisible by 2", self.dbg, remainder.len());
        }
        let full_length = (bytes.len() / channels) / Self::SAMPLE_SIZE;
        if values.len() < full_length {
            let target_remaind = full_length - values.len();
            values.extend(vec![0; target_remaind]);
        }
        for (index, word) in words.iter().enumerate().skip(channel).step_by(channels) {
            log::trace!("{}.convert | index: {}  |  word: {:?}", self.dbg, index, word);
            values[index] = u16::from_le_bytes(*word);
        }
        log::trace!("{}.convert | values: {:?}", self.dbg, values);
        Ok(())
    }
    ///
    /// Читает то АЦП пакеты из сети (UDP) и распаковывает в сэмплы `u16`
    #[named]
    fn parse(&self, values: &mut Vec<u16>, ts: DateTime<Utc>) -> Result<(), Error> {
        // log::debug!("{}.parse | message: {:?}", self.id, buf);
        match self.read() {
            Err(err) => {
                // log::warn!("{}.parse | message: {:?}", self.id, buf);
                return Err(err_pass!(self.dbg, err, "Can't read message from UDP"));
            }
            Ok(len) => {
                let buff = self.buff.borrow();
                match buff[..len] {
                    // Data message received
                    [UdpClient::DAT, channels, typ, c1,c2,c3, c4, ..] => {
                        let count = u32::from_le_bytes([c1, c2, c3, c4]) as usize;
                        // log::debug!("{dbg}.parse | channels: {}, count: {}", channels, count);
                        let typ = InputType::try_from(typ)
                            .map_err(|err| err_pass!(self.dbg, err, "Wrong value type {}", typ))?;
                        // log::debug!("{dbg}.parse | channels: {}, count: {} values of type {}", channels, count, typ);
                        // log::debug!("{dbg}.parse | channels: {} type: {} count: {}  |  {:?}", channels, typ, count, &buf[UdpClient::HEAD_LEN..(if buf.len() < 10 {buf.len()} else {10})]);
                        let bytes = buff.get(UdpClient::HEAD_LEN..(UdpClient::HEAD_LEN + count))
                            .ok_or(err!(self.dbg, "Wrong message length: {}, expected {}", buff.len(), UdpClient::HEAD_LEN + count))?;
                        // let bytes: &Vec<u8> = bytes;
                        // log::trace!("{}.parse | bytes: {:?}", dbg, bytes);
                        // log::trace!("{}.parse | points: {:?}", dbg, points.iter().map(|(id, point)| format!("{}[{}]", point.name(), id)).collect::<Vec<String>>());
                        for channel in 0..channels {
                            if let Err(err) = self.convert(channel as usize, channels as usize, bytes, values) {
                                return Err(err_pass!(self.dbg, err));
                            }
                        }
                        Ok(())
                    }
                    [UdpClient::ERR, err] | [UdpClient::ERR, err, ..] => {
                        Err(err_pass!(self.dbg, err, "Error received from ADC"))
                    }
                    [UdpClient::SYN] | [UdpClient::SYN, ..] => {
                        Err(err!(self.dbg, "Data message expected, but SYN received: {:?}...", &buff[..=10]))
                    }
                    [] => {
                        Err(err!(self.dbg, "Empty message received"))
                    }
                    _ => {
                        Err(err!(self.dbg, "Unknown message format: {:?}...", &buff[..=10]))
                    }
                }
            }
        }
    }
    ///
    /// Reads buffer from UDP
    /// Returns len points from the current DB
    /// - parses raw data into the configured points
    /// - returns only points with updated value or status
    #[named]
    fn read(&self) -> Result<usize, Error> {
        if self.socket.borrow().is_none() {
            let udp_connect = UdpClientConnect::new(&self.name, &self.conf.local_addr, &self.conf.remote_addr, self.conf.mtu);
            match udp_connect.connect() {
                Err(err) => return Err(err_pass!(self.dbg, err, "Socket is not connected")),
                Ok(socket) => {
                    _ = self.socket.borrow_mut().replace(socket);
                }
            }
        }
        let socket = self.socket.borrow();
        let Some(socket) = socket.as_ref() else  {
            return Err(err!(self.dbg, "Socket is not connected"));
        };
        let mut buff = self.buff.borrow_mut();
        buff.fill(0);
        match socket.recv_from(&mut buff) {
            Ok((len, _)) => {
                // let count = u32::from_le_bytes(buf.get(3..=6).unwrap_or(&[0,0,0,0]).try_into().unwrap()) as usize;
                // log::debug!("{dbg}.read | Received buffer {} bytes, \n\t | {}, {}, {}, {} | {}, {}, {}, {}", len,
                //     buf.get(0).unwrap_or(&0),
                //     buf.get(1).unwrap_or(&0),
                //     buf.get(2).unwrap_or(&0),
                //     count,
                //     u16::from_le_bytes(buf.get(07..=08).unwrap_or(&[0,0]).try_into().unwrap()),
                //     u16::from_le_bytes(buf.get(09..=10).unwrap_or(&[0,0]).try_into().unwrap()),
                //     u16::from_le_bytes(buf.get(11..=12).unwrap_or(&[0,0]).try_into().unwrap()),
                //     u16::from_le_bytes(buf.get(13..=14).unwrap_or(&[0,0]).try_into().unwrap()),
                // );
                // self.parse(buf.as_slice(), Utc::now());
                Ok(len)
            }
            Err(err) => {
                match err.kind() {
                    std::io::ErrorKind::WouldBlock => Err(err_pass!(self.dbg, err, "Socket read timeout")),
                    std::io::ErrorKind::TimedOut => Err(err_pass!(self.dbg, err, "Socket read timeout")),
                    _ => Err(err_pass!(self.dbg, err, "Socket read timeout")),
                }
            }
        }
    }
}
