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
///     - `CHANNELS` = 0...255 - Count of the input channels (0 - first input channel)
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
    /// Читает то АЦП пакеты из сети (UDP) и распаковывает в сэмплы `u16`
    /// - `values` Сырые сэмплы из АЦП, разложенные по каналам
    #[named]
    pub fn read(&self, values: &mut Vec<Vec<u16>>) -> Result<(), Error> {
        let len = self.receive().map_err(|err| err_pass!(self.dbg, err))?;
        self.parse(values, len).map_err(|err| err_pass!(self.dbg, err))?;
        Ok(())
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
            values.resize(full_length, 0);
        }
        let mut index = 0;
        let values = &mut values[..full_length];
        for word in words.iter().skip(channel).step_by(channels) {
            // log::trace!("{}.convert | index: {}  |  word: {:?}", self.dbg, index, word);
            values[index] = u16::from_le_bytes(*word);
            index += 1;
        }
        log::trace!("{}.convert | values: {:?}", self.dbg, values);
        Ok(())
    }
    ///
    /// Читает то АЦП пакеты из сети (UDP) и распаковывает в сэмплы `u16`
    /// - `values` Сырые сэмплы из АЦП, разложенные по каналам
    #[named]
    #[inline]
    fn parse(&self, values: &mut Vec<Vec<u16>>, len: usize) -> Result<(), Error> {
        let buff = &self.buff.borrow()[..len];
        match buff {
            [UdpClient::DAT, channels, typ, c1, c2, c3, c4, ..] => {
                let count = u32::from_le_bytes([*c1, *c2, *c3, *c4]) as usize;
                // log::debug!("{dbg}.parse | channels: {}, count: {}", channels, count);
                let typ = InputType::try_from(*typ)
                    .map_err(|err| err_pass!(self.dbg, err, "Wrong value type {}", typ))?;
                // log::debug!("{dbg}.parse | channels: {}, count: {} values of type {}", channels, count, typ);
                // log::debug!("{dbg}.parse | channels: {} type: {} count: {}  |  {:?}", channels, typ, count, &buf[UdpClient::HEAD_LEN..(if buf.len() < 10 {buf.len()} else {10})]);
                let bytes = buff.get(UdpClient::HEAD_LEN..(UdpClient::HEAD_LEN + count))
                    .ok_or(err!(self.dbg, "Wrong message length: {}, expected {}", buff.len(), UdpClient::HEAD_LEN + count))?;
                // let bytes: &Vec<u8> = bytes;
                // log::trace!("{}.parse | bytes: {:?}", dbg, bytes);
                // log::trace!("{}.parse | points: {:?}", dbg, points.iter().map(|(id, point)| format!("{}[{}]", point.name(), id)).collect::<Vec<String>>());
                if *channels as usize > values.len() {
                    values.resize_with(*channels as usize, Vec::new);
                }
                for channel in 0..*channels {
                    let channel_values = &mut values[channel as usize];
                    if let Err(err) = self.convert(channel as usize, *channels as usize, bytes, channel_values) {
                        return Err(err_pass!(self.dbg, err));
                    }
                }
                Ok(())
            }
            [UdpClient::ERR, err] | [UdpClient::ERR, err, ..] => {
                Err(err_pass!(self.dbg, err, "Error received from ADC"))
            }
            [UdpClient::SYN] | [UdpClient::SYN, ..] => {
                let len = std::cmp::min(buff.len(), 12);
                Err(err!(self.dbg, "Data message expected, but SYN received: {:?}...", &buff[..len]))
            }
            [] => Err(err!(self.dbg, "Empty message received")),
            _ => {
                let len = std::cmp::min(buff.len(), 12);
                Err(err!(self.dbg, "Unknown message format: {:?}...", &buff[..len]))
            }
        }
    }
    /// ### Reads buffer from UDP.
    /// Returns len points from the current DB.
    #[named]
    #[inline]
    fn receive(&self) -> Result<usize, Error> {
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
            Err(err) => match err.kind() {
                std::io::ErrorKind::WouldBlock => Err(err_pass!(self.dbg, err, "Socket read timeout")),
                std::io::ErrorKind::TimedOut => Err(err_pass!(self.dbg, err, "Socket read timeout")),
                _ => Err(err_pass!(self.dbg, err, "Socket read timeout")),
            }
        }
    }
    ///
    pub fn exit(&self) {
        if let Some(s) = self.socket.borrow_mut().take() {
            drop(s)
        }
    }
}
///
/// Basic Tests
mod tests {
    use sal_sync::services::conf::{ConfDuration, ConfDurationUnit};
    use super::*;
    fn mock_conf() -> UdpClientConf {
        UdpClientConf {
            description: Some("test_convert_empty_bytes_returns_error".into()),
            reconnect: ConfDuration::new(1000, ConfDurationUnit::Millis),
            protocol: "udp-raw".into(),
            local_addr: "0.0.0.0".into(),
            remote_addr: "0.0.0.0".into(),
            mtu: 1500,
        }
    }
    // Вспомогательный метод для быстрой сборки правильного заголовка UDP-пакета
    fn make_udp_header(fun: u8, channels: u8, data_type: u8, data_bytes_count: u32) -> Vec<u8> {
        let mut header = vec![fun, channels, data_type];
        header.extend_from_slice(&data_bytes_count.to_le_bytes());
        header
    }
    #[test]
    fn test_convert_empty_bytes_returns_error() {
        let client = UdpClient::new("test_convert_empty_bytes_returns_error", mock_conf());
        let mut values = Vec::new();
        let result = client.convert(0, 2, &[], &mut values);
        assert!(result.is_err());
    }
    #[test]
    fn test_convert_demux_logic() {
        // Имитируем структуру АЦП, у которой SAMPLE_SIZE = 2
        // Передаем поток байт для 2-х каналов (по 2 сэмпла на каждый канал, итого 4 слова = 8 байт)
        // Порядок в сети: CH0_S1, CH1_S1, CH0_S2, CH1_S2
        let ch0_s1 = 111u16.to_le_bytes();
        let ch1_s1 = 222u16.to_le_bytes();
        let ch0_s2 = 333u16.to_le_bytes();
        let ch1_s2 = 444u16.to_le_bytes();
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&ch0_s1);
        bytes.extend_from_slice(&ch1_s1);
        bytes.extend_from_slice(&ch0_s2);
        bytes.extend_from_slice(&ch1_s2);
        let client = UdpClient::new("test_convert_demux_logic", mock_conf());
        // Проверяем извлечение для нулевого канала (CH0)
        let mut values_ch0 = Vec::new();
        let res_ch0 = client.convert(0, 2, &bytes, &mut values_ch0);
        assert!(res_ch0.is_ok());
        assert_eq!(values_ch0, vec![111, 333]);
        // Проверяем извлечение для первого канала (CH1)
        let mut values_ch1 = Vec::new();
        let res_ch1 = client.convert(1, 2, &bytes, &mut values_ch1);
        assert!(res_ch1.is_ok());
        assert_eq!(values_ch1, vec![222, 444]);
    }
    #[test]
    fn test_convert_resizes_existing_vector() {
        let client = UdpClient::new("test_convert_resizes_existing_vector", mock_conf());
        // Вектор изначально имеет емкость 1 элемент
        let mut values = vec![999]; 
        let sample1 = 42u16.to_le_bytes();
        let sample2 = 84u16.to_le_bytes();
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&sample1);
        bytes.extend_from_slice(&sample2);
        // 2 сэмпла, 1 канал -> ожидаемая длина 2. Вектор должен расшириться до 2 элементов.
        let res = client.convert(0, 1, &bytes, &mut values);
        assert!(res.is_ok());
        assert_eq!(values.len(), 2);
        assert_eq!(values, vec![42, 84]); // Старое значение 999 затерлось новыми данными с начала
    }
    // ТЕСТЫ ДЛЯ МЕТОДА PARSE
    #[test]
    fn test_parse_empty_udp_packet_returns_error() {
        let client = UdpClient::new("test_parse_empty_udp_packet_returns_error", mock_conf());
        let mut values = Vec::new();
        // Имитируем пустое чтение из сети
        client.buff.borrow_mut().clear(); 
        let len = 0;
        let result = client.parse(&mut values, len);
        assert!(result.is_err());
    }
    #[test]
    fn test_parse_error_packet_returns_err() {
        let client = UdpClient::new("test_parse_error_packet_returns_err", mock_conf());
        let mut values = Vec::new();
        // Записываем в пакет маркер ошибки (0x02 — имитирует UdpClient::ERR)
        client.buff.borrow_mut().extend_from_slice(&[0x02, 0x15]); 
        let result = client.parse(&mut values, 2);
        assert!(result.is_err());
    }
    #[test]
    fn test_parse_err_packet_returns_error_result() {
        let client = UdpClient::new("test_parse_err_packet_returns_error_result", mock_conf());
        let mut values = Vec::new();
        // Формируем пакет ошибки: FUN(0x07) + код ошибки АЦП (например, 0x02 - ADC Error)
        let packet = vec![UdpClient::ERR, 0x02];
        let len = packet.len();
        *client.buff.borrow_mut() = packet;
        let result = client.parse(&mut values, len);
        // Метод parse() должен перехватить паттерн UdpClient::ERR и вернуть Err(...)
        assert!(result.is_err());
    }
    #[test]
    fn test_parse_wrong_head_len_returns_error() {
        let client = UdpClient::new("test_parse_wrong_head_len_returns_error", mock_conf());
        let mut values = Vec::new();
        // Посылаем обрезанный пакет DAT, в котором указано, что данных 100 байт, а физически их нет
        let packet = make_udp_header(UdpClient::DAT, 2, 16, 100);
        let len = packet.len();
        *client.buff.borrow_mut() = packet;
        let result = client.parse(&mut values, len);
        assert!(result.is_err()); // buff.get(HEAD_LEN..HEAD_LEN+count) должен вернуть None и выдать ошибку
    }
    #[test]
    fn test_parse_valid_dat_packet() {
        let client = UdpClient::new("test_parse_valid_dat_packet", mock_conf());
        let mut values: Vec<Vec<u16>> = Vec::new();
        // Настройка данных: 3 канала, тип 16 (u16), по 1 сэмплу на канал (всего 3 сэмпла = 6 байт)
        let channels_count = 3u8;
        let data_type = 16u8;
        let data_len_bytes = 6u32;
        let s0 = 10u16.to_le_bytes();
        let s1 = 20u16.to_le_bytes();
        let s2 = 30u16.to_le_bytes();
        // Собираем пакет согласно спецификации: FUN(0x02) + CHANNELS(3) + TYPE(16) + COUNT(6) + DATA
        let mut packet = make_udp_header(UdpClient::DAT, channels_count, data_type, data_len_bytes);
        packet.extend_from_slice(&s0);
        packet.extend_from_slice(&s1);
        packet.extend_from_slice(&s2);
        // Помещаем пакет в буфер вашего мок-сокета / структуры
        let len = packet.len();
        *client.buff.borrow_mut() = packet;
        let result = client.parse(&mut values, len);
        assert!(result.is_ok());
        assert_eq!(values.len(), 3); // Проверяем, что создалось ровно 3 канала
        assert_eq!(values[0], vec![10]); // Данные CH0
        assert_eq!(values[1], vec![20]); // Данные CH1
        assert_eq!(values[2], vec![30]); // Данные CH2
    }
}
