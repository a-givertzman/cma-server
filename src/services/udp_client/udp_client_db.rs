use std::{fs, io::Write, net::UdpSocket, sync::mpsc::Sender};
use chrono::Utc;
use concat_string::concat_string;
use indexmap::IndexMap;
use log::{trace, warn};
use sal_sync::services::entity::{
    name::Name, point::{point::Point, point_config::PointConfig, point_config_filters::PointConfigFilter, point_config_type::PointConfigType},
    status::status::Status
};
use crate::{
    conf::udp_client_config::udp_client_db_config::UdpClientDbConfig,
    core_::{filter::{filter::{Filter, FilterEmpty}, filter_threshold::FilterThreshold}, state::change_notify::ChangeNotify},
};
use super::{parse_point::ParsePoint, udp_client::UdpClient, udpc_parse_i16::UdpcParseI16};
///
/// 
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum State {
    Start,
    Exit,
    UdpBindError,
    UdpRecvError,
}
///
/// Reads data from Vibro-analytics microcontroller (Sub MC)
pub struct UdpClientDb {
    id: String,
    pub name: Name,
    pub description: String,
    pub size: u32,
    pub points: IndexMap<String, Box<dyn ParsePoint>>,
    notify: ChangeNotify<State, String>,
}
//
//
impl UdpClientDb {
    ///
    /// Creates new instance of the [UdpClientDb]
    /// - app - string represents application name, for point path
    /// - parent - parent id, used for debugging
    /// - conf - configuration of the [ProfinetDB]
    pub fn new(parent_id: impl Into<String>, tx_id: usize, conf: &UdpClientDbConfig) -> Self {
        let self_id = format!("{}/UdpClientDb({})", parent_id.into(), conf.name);
        Self {
            id: self_id.clone(),
            name: conf.name.clone(),
            description: conf.description.clone(),
            size: conf.size as u32,
            points: Self::configure_parse_points(&self_id, tx_id, conf),
            notify: ChangeNotify::new(self_id, State::Start, vec![
                (State::Start,          Box::new(|message| log::info!("{}", message))),
                (State::Exit,           Box::new(|message| log::info!("{}", message))),
                (State::UdpBindError,   Box::new(|message| log::error!("{}", message))),
                (State::UdpRecvError,   Box::new(|message| log::error!("{}", message))),
            ]),
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
                    warn!("{}.log | Error open file: '{}'\n\terror: {:?}", self_id, path, err)
                }
            }
        }
    }
    ///
    /// Returns updated points from the current DB
    ///     - parses raw data into the configured points
    ///     - returns only points with updated value or status
    pub fn read(&mut self, socket: &UdpSocket, tx_send: &Sender<Point>) -> Result<(), String> {
        let mtu = 4096;
        let mut buf = vec![0; mtu];
        let mut count: usize;
        let mut message = String::new();
        let status = Status::Ok;
        match socket.recv_from(&mut buf) {
            Ok((_, src_addr)) => {
                let timestamp = Utc::now();
                match buf.as_slice() {
                    // Empty message received
                    &[] => {
                        log::debug!("{}.run | {}: Empty message received", self.id, src_addr);
                    }
                    // Start ACK received
                    &[UdpClient::SYN, UdpClient::EOT, ..] => {
                        log::debug!("{}.run | {}: Start message ACK received", self.id, src_addr);
                    }
                    // Data message received
                    &[UdpClient::SYN, addr, type_, c1,c2,c3, c4, ..] => {
                        count = u32::from_be_bytes([c1, c2, c3, c4]) as usize;
                        log::debug!("{}.run | {}: addr: {} type: {} count: {}", self.id, src_addr, addr, type_, count);
                        match &buf[UdpClient::HEAD_LEN..(UdpClient::HEAD_LEN + count)].try_into() {
                            Ok(data) => {
                                let bytes: &Vec<u8> = data;
                                for (_, parse_point) in &mut self.points {
                                    if let Some(point) = parse_point.next(bytes, status, timestamp) {
                                        // debug!("{}.read | point: {:?}", self.id, point);
                                        match tx_send.send(point) {
                                            Ok(_) => {}
                                            Err(err) => {
                                                message = format!("{}.read | send error: {}", self.id, err);
                                                warn!("{}", message);
                                            }
                                        }
                                    }
                                }

                            }
                            Err(err) => {
                                log::error!("{}.run | {}: Error message length: {}, expected {}, \n\t error: {:#?}", self.id, src_addr, buf.len(), UdpClient::HEAD_LEN + count, err);
                            }
                        }
                    }
                    _ => {
                        log::warn!("{}.run | {}: Unknown message format: {:#?}...", self.id, src_addr, &buf[..=10]);
                    }
                }
            }
            Err(err) => self.notify.add(State::UdpRecvError, format!("{}.run | UdpSocket recv error: {:#?}", self.id, err)),
        }

        // trace!("{}.read | bytes: {:?}", self.id, bytes);
        // let timestamp = Utc::now();
        // let mut message = String::new();
        match message.is_empty() {
            true => Ok(()),
            false => Err(message),
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
    //                     warn!("{}", message);
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
    fn configure_parse_points(self_id: &str, tx_id: usize, conf: &UdpClientDbConfig) -> IndexMap<String, Box<dyn ParsePoint>> {
        conf.points.iter().map(|point_conf| {
            match point_conf.type_ {
                // PointConfigType::Bool => {
                //     (point_conf.name.clone(), Self::box_bool(tx_id, point_conf.name.clone(), point_conf))
                // }
                PointConfigType::Int => {
                    (point_conf.name.clone(), Self::box_i16(tx_id, point_conf.name.clone(), conf.size as usize, point_conf))
                }
                // PointConfigType::Real => {
                //     (point_conf.name.clone(), Self::box_real(tx_id, point_conf.name.clone(), point_conf))
                // }
                // PointConfigType::Double => {
                //     (point_conf.name.clone(), Self::box_real(tx_id, point_conf.name.clone(), point_conf))
                // }
                _ => panic!("{}.configureParsePoints | Unknown type '{:?}' for S7 Device", self_id, point_conf.type_)
            }
        }).collect()
    }
    // ///
    // ///
    // fn box_bool(tx_id: usize, name: String, config: &PointConfig) -> Box<dyn ParsePoint> {
    //     Box::new(UdpClientParseBool::new(tx_id, name, config))
    // }
    ///
    ///
    fn box_i16(tx_id: usize, name: String, size: usize, config: &PointConfig) -> Box<dyn ParsePoint> {
        Box::new(UdpcParseI16::new(
            tx_id,
            name,
            size,
            config,
        ))
    }
    // ///
    // ///
    // fn box_real(tx_id: usize, name: String, config: &PointConfig) -> Box<dyn ParsePoint> {
    //     Box::new(S7ParseReal::new(
    //         tx_id,
    //         name,
    //         config,
    //         Self::real_filter(config.filters.clone()),
    //     ))
    // }
    // ///
    // ///
    // fn i16_filter(conf: Option<PointConfigFilter>) -> Box<dyn Filter<Item = i16>> {
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
    // fn real_filter(conf: Option<PointConfigFilter>) -> Box<dyn Filter<Item = f32>> {
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
    // fn double_filter(conf: Option<PointConfigFilter>) -> Box<dyn Filter<Item = f64>> {
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
