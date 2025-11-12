use std::{fs, io::Write, net::UdpSocket, sync::{Arc, atomic::{AtomicBool, Ordering}}};
use chrono::{DateTime, Utc};
use concat_string::concat_string;
use indexmap::IndexMap;
use sal_core::{dbg::Dbg, error::{Error, ErrorLimit}};
use sal_sync::{
    kernel::state::ChangeNotify,
    services::{Service, ServiceCycle, Services, entity::{
        Name, Object, Point, PointConf, PointConfType, PointTxId, Status
    }}, sync::{Handles, channel::Sender}, thread_pool::Scheduler
};
use crate::services::udp_client::UdpClientConnect;

use super::{ParsePoint, UdpcParseU16, UdpClientConf};
///
/// 
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum State {
    Start,
    Exit,
    ReadError,
    ConnectError,
    Connected,
}
///
/// Reads data from Vibro-analytics microcontroller (Sub MC)
pub struct UdpClient {
    txid: usize,
    name: Name,
    conf: UdpClientConf,
    services: Arc<Services>,
    scheduler: Scheduler,
    handles: Handles<()>,
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
    ///
    /// Creates new instance of the [UdpClient]
    /// - app - string represents application name, for point path
    /// - parent - parent id, used for debugging
    /// - conf - configuration of the [UdpClient]
    pub fn new(conf: UdpClientConf, services: Arc<Services>, scheduler: Scheduler) -> Self {
        let txid = PointTxId::from_str(&conf.name.join());
        let dbg = Dbg::new(conf.name.parent(), conf.name.me());
        Self {
            txid,
            name: conf.name.clone(),
            conf,
            services,
            scheduler,
            handles: Handles::new(&dbg),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
    ///
    /// Writes Point's to the log file
    #[allow(unused)]
    fn log(self_id: &str, parent: &Name, point: &Point) {
        let path = concat_string!("./logs", parent.join(), "/points.log");
        match fs::OpenOptions::new().create(true).append(true).open(&path) {
            Ok(mut f) => {
                f.write_fmt(format_args!("{:?}\n", point)).unwrap();
            }
            Err(err) => {
                if log::max_level() >= log::LevelFilter::Trace {
                    log::warn!("{}.log | Error open file: '{}'\n\terror: {:?}", self_id, path, err)
                }
            }
        }
    }
    ///
    /// 
    fn parse(dbg: &Dbg, points: &mut IndexMap<u8, Box<dyn ParsePoint>>, buf: &[u8], timestamp: DateTime<Utc>, tx_send: &Sender<Point>) {
        let count: usize;
        let status = Status::Ok;
        // log::debug!("{}.parse | message: {:?}", self.id, buf);
        match buf {
            // Data message received
            &[UdpClient::DAT, addr, _type_, c1,c2,c3, c4, ..] => {
                count = u32::from_be_bytes([c1, c2, c3, c4]) as usize;
                // log::debug!("{}.parse | addr: {} type: {} count: {}  |  {:?}", self.id, addr, type_, count, &buf[UdpClient::HEAD_LEN..(UdpClient::HEAD_LEN + count)]);
                match &buf[UdpClient::HEAD_LEN..(UdpClient::HEAD_LEN + count)].try_into() {
                    Ok(data) => {
                        let bytes: &Vec<u8> = data;
                        log::trace!("{}.parse | bytes: {:?}", dbg, bytes);
                        log::trace!("{}.parse | points: {:?}", dbg, points.iter().map(|(id, point)| format!("{}[{}]", point.name(), id)).collect::<Vec<String>>());
                        match points.get_mut(&addr) {
                            Some(parse_point) => {
                                match parse_point.add(bytes, status, timestamp) {
                                    Ok(points) => {
                                        for point in points {
                                            // log::debug!("{}.parse | point: {:?}", dbg, point);
                                            if let Err(err) = tx_send.send(point) {
                                                log::warn!("{}.parse | Send error: {}", dbg, err);
                                            }
                                        }
                                    }
                                    Err(err) => log::warn!("{}.parse | Error: {}", dbg, err),
                                }
                            }
                            None => todo!(),
                        }
                    }
                    Err(err) => {
                        log::error!("{}.parse | Error message length: {}, expected {}, \n\t error: {:#?}", dbg, buf.len(), UdpClient::HEAD_LEN + count, err);
                    }
                }
            }
            _ => {
                log::warn!("{}.parse | Unknown message format: {:#?}...", dbg, &buf[..=10]);
            }
        }
    }
    ///
    /// Returns updated points from the current DB
    /// - parses raw data into the configured points
    /// - returns only points with updated value or status
    pub fn read(dbg: &Dbg, socket: &UdpSocket, points: &mut IndexMap<u8, Box<dyn ParsePoint>>, tx_send: &Sender<Point>, mtu: usize) -> Result<(), Error> {
        let error = Error::new(dbg, "read");
        let mut buf = vec![0; mtu];
        match socket.recv_from(&mut buf) {
            Ok((_, _)) => {
                let timestamp = Utc::now();
                Self::parse(dbg, points, buf.as_slice(), timestamp, tx_send);
                Ok(())
            }
            Err(err) => {
                match err.kind() {
                    std::io::ErrorKind::WouldBlock => Err(error.pass_with("Socket read timeout", err.to_string())),
                    std::io::ErrorKind::TimedOut => Err(error.pass_with("Socket read timeout", err.to_string())),
                    _ => Err(error.pass_with("Socket read timeout", err.to_string())),
                }
            }
        }
    }
    ///
    /// Sends all configured points from the current DB with the given status
    // pub fn yield_status(&mut self, status: Status, tx_send: &Sender<Point>) -> Result<(), String> {
    //     let mut message = String::new();
    //     for (_key, parse_point) in &mut self.points {
    //         if let Some(point) = parse_point.next_status(status) {
    //             match tx_send.send(point) {
    //                 Ok(_) => {}
    //                 Err(err) => {
    //                     message = format!("{}.yield_status | send error: {}", self.id, err);
    //                     log::warn!("{}", message);
    //                 }
    //             }
    //         }
    //     }
    //     if message.is_empty() {
    //         return Ok(())
    //     }
    //     Err(message)
    // }
    // ///
    // /// Writes point to the current DB
    // ///     - Returns Ok() if succeed, Err(message) on fail
    // pub fn write(&mut self, client: &S7Client, point: Point) -> Result<(), String> {
    //     let mut message = String::new();
    //     match self.points.get(&point.name()) {
    //         Some(parse_point) => {
    //             let address = parse_point.address();
    //             match point {
    //                 Point::Bool(point) => {
    //                     // !!! Not implemented because before write byte of the bool bits, that byte must be read from device
    //                     // let mut buf = [0; 16];
    //                     // let index = address.offset.unwrap() as usize;
    //                     // buf[index] = point.value.0 as u8;
    //                     // client.write(self.number, address.offset.unwrap(), 2, &mut buf)
    //                     message = format!("{}.write | Write 'Bool' to the S7 Device - not implemented, point: {:?}", self.id, point.name);
    //                     Err(message)
    //                 }
    //                 Point::Int(point) => {
    //                     client.write(self.channel, address.offset.unwrap(), 2, &mut (point.value as i16).to_be_bytes())
    //                 }
    //                 Point::Real(point) => {
    //                     client.write(self.channel, address.offset.unwrap(), 4, &mut (point.value).to_be_bytes())
    //                 }
    //                 Point::Double(point) => {
    //                     client.write(self.channel, address.offset.unwrap(), 4, &mut (point.value as f32).to_be_bytes())
    //                 }
    //                 Point::String(point) => {
    //                     message = format!("{}.write | Write 'String' to the S7 Device - not implemented, point: {:?}", self.id, point.name);
    //                     Err(message)
    //                 }
    //             }
    //         }
    //         None => {
    //             Err(message)
    //         }
    //     }
    // }
    ///
    /// Configuring ParsePoint objects depending on point configurations coming from [conf]
    fn configure_parse_points(dbg: &Dbg, tx_id: usize, conf: &[PointConf]) -> IndexMap<u8, Box<dyn ParsePoint>> {
        conf.iter().filter_map(|point_conf| {
            match point_conf.type_ {
                // PointConfType::Bool => {
                //     (point_conf.name.clone(), Self::box_bool(tx_id, point_conf.name.clone(), point_conf))
                // }
                PointConfType::Int => {
                    Some((point_conf.id as u8, Self::box_i16(tx_id, point_conf.name.clone(), point_conf)))
                }
                // PointConfType::Real => {
                //     (point_conf.name.clone(), Self::box_real(tx_id, point_conf.name.clone(), point_conf))
                // }
                // PointConfType::Double => {
                //     (point_conf.name.clone(), Self::box_real(tx_id, point_conf.name.clone(), point_conf))
                // }
                _ => {
                    log::warn!("{}.configure_parse_points | Unknown type '{:?}' for UdpClient Device", dbg, point_conf.type_);
                    None
                }
            }
        }).collect()
    }
    // ///
    // ///
    // fn box_bool(tx_id: usize, name: String, config: &PointConf) -> Box<dyn ParsePoint> {
    //     Box::new(UdpClientParseBool::new(tx_id, name, config))
    // }
    ///
    ///
    fn box_i16(tx_id: usize, name: String, config: &PointConf) -> Box<dyn ParsePoint> {
        Box::new(UdpcParseU16::new(
            tx_id,
            name,
            config,
        ))
    }
    // ///
    // ///
    // fn box_real(tx_id: usize, name: String, config: &PointConf) -> Box<dyn ParsePoint> {
    //     Box::new(S7ParseReal::new(
    //         tx_id,
    //         name,
    //         config,
    //         Self::real_filter(config.filters.clone()),
    //     ))
    // }
    // ///
    // ///
    // fn i16_filter(conf: Option<PointConfFilter>) -> Box<dyn Filter<Item = i16>> {
    //     match conf {
    //         Some(conf) => {
    //             Box::new(
    //                 FilterThreshold::new(0i16, conf.threshold, conf.factor.unwrap_or(0.0))
    //             )
    //         }
    //         None => Box::new(FilterEmpty::new(0)),
    //     }
    // }
    // ///
    // ///
    // fn real_filter(conf: Option<PointConfFilter>) -> Box<dyn Filter<Item = f32>> {
    //     match conf {
    //         Some(conf) => {
    //             Box::new(
    //                 FilterThreshold::new(0.0f32, conf.threshold, conf.factor.unwrap_or(0.0))
    //             )
    //         }
    //         None => Box::new(FilterEmpty::<f32>::new(0.0)),
    //     }
    // }
    // ///
    // ///
    // fn double_filter(conf: Option<PointConfFilter>) -> Box<dyn Filter<Item = f64>> {
    //     match conf {
    //         Some(conf) => {
    //             Box::new(
    //                 FilterThreshold::new(0.0f64, conf.threshold, conf.factor.unwrap_or(0.0))
    //             )
    //         }
    //         None => Box::new(FilterEmpty::<f64>::new(0.0)),
    //     }
    // }
}
//
//
impl Object for UdpClient {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl std::fmt::Debug for UdpClient {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UdpClient")
            .field("id", &self.dbg)
            .finish()
    }
}
//
//
// static SELF_ID: std::sync::LazyLock<RwLock<Dbg>> = std::sync::LazyLock::new(|| RwLock::new(Dbg::own("")));
//
// 
impl Service for UdpClient {
    //
    // 
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let name = self.name.clone();
        let txid = self.txid;
        let conf = self.conf.clone();
        let mut points: IndexMap<u8, Box<dyn ParsePoint>> = Self::configure_parse_points(&dbg, txid, &conf.points);
        let exit = self.exit.clone();
        let services = self.services.clone();
        log::info!("{}.run | Preparing thread...", dbg);
        // *SELF_ID.write() = dbg.clone();
        let handle = self.scheduler.spawn(move || {
            let dbg = &dbg;
            let mut notify: ChangeNotify<_, String> = ChangeNotify::new(dbg, State::Start, vec![
                (State::Start,          Box::new(|message| log::info!("{}", message))),
                (State::Connected,      Box::new(|message| log::info!("{}", message))),
                (State::Exit,           Box::new(|message| log::info!("{}", message))),
                (State::ReadError,      Box::new(|message| log::warn!("{}", message))),
                (State::ConnectError,   Box::new(|message| log::warn!("{}", message))),
            ]);
            let send = services
                .get_link(&conf.send_to)
                .unwrap_or_else(|err| panic!("{}.run | Link {} - Not found, error: {}", dbg, conf.send_to.name(), err));
            let mut reconnect = ServiceCycle::new(dbg, conf.reconnect);
            let udp_connect = UdpClientConnect::new(&name, conf.local_addr, conf.remote_addr, conf.mtu);
            'main: loop {
                notify.add(State::Start, format!("{}.run | Reading from device...", dbg));
                reconnect.start();
                match udp_connect.connect() {
                    Ok(socket) => {
                        notify.add(State::Connected, format!("{}.run | Connected to device", dbg));
                        let mut error_limit = ErrorLimit::new(3);
                        'read: loop {
                            match Self::read(dbg, &socket, &mut points, &send, conf.mtu) {
                                Ok(_) => {
                                    error_limit.reset();
                                }
                                Err(err) => {
                                    notify.add(State::Start, format!("{}.run | Can't read from device, error: {:?}", dbg, err));
                                    if error_limit.add().is_err() {
                                        notify.add(State::ConnectError, format!("{}.run | Connection error: {:?}", dbg, err));
                                        break 'read;
                                    }
                                }
                            }
                            if exit.load(Ordering::SeqCst) {
                                break 'main;
                            }
                        }
                    }
                    Err(err) => {
                        notify.add(State::ConnectError, format!("{}.run | Connection error: {:?}", dbg, err));
                    }
                }
                if exit.load(Ordering::SeqCst) {
                    break 'main;
                }
                reconnect.wait();
                if exit.load(Ordering::SeqCst) {
                    break 'main;
                }
            }
            Ok(())
        });
        match handle {
            Ok(handle) => {
                log::info!("{}.run | Starting - ok", self.dbg);
                self.handles.push(handle);
                Ok(())
            }
            Err(err) => {
                let err = Error::new(&self.dbg, "run").pass_with("Start failed", err.to_string());
                log::warn!("{}", err);
                Err(err)
            }
        }
    }
    //
    //
    fn wait(&self) -> Result<(), Error> {
        self.handles.wait()
    }
    //
    //
    fn is_finished(&self) -> bool {
        self.handles.is_finished()
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }    
}
