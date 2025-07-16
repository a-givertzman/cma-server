use std::{
    net::TcpStream, sync::{atomic::{AtomicU32, Ordering}, mpsc::{self, Sender}, Arc, Mutex, RwLock},
    thread::{self, JoinHandle}, time::Duration,
};
use log::{debug, error, info, warn};
use sal_sync::{collections::map::IndexMapFxHasher, services::{entity::{cot::Cot, point::{point::Point, point_hlr::PointHlr}, status::status::Status}, service::service_cycle::ServiceCycle, subscription::subscription_criteria::SubscriptionCriteria}};
use crate::{
    conf::slmp_client_config::slmp_client_config::SlmpClientConfig,
    core_::{
        failure::errors_limit::ErrorLimit,
        state::{change_notify::ChangeNotify, exit_notify::ExitNotify},
    },
    services::{safe_lock::rwlock::SafeLock, services::Services, slmp_client::slmp_db::SlmpDb},
};

use super::slmp_read::SlmpRead;
///
/// Cyclicaly reads SLMP data ranges (DB's) specified in the [conf]
/// - exit - external signal to stop the main read cicle and exit the thread
/// - exit_pair - exit signal from / to notify 'Write' partner to exit the thread
pub struct SlmpWrite {
    tx_id: usize,
    id: String,
    // name: Name,
    conf: SlmpClientConfig,
    dest: Sender<Point>,
    dbs: Arc<Mutex<IndexMapFxHasher<String, SlmpDb>>>,
    // diagnosis: Arc<Mutex<IndexMapFxHasher<DiagKeywd, DiagPoint>>>,
    services: Arc<RwLock<Services>>,
    status: Arc<AtomicU32>,
    exit: Arc<ExitNotify>,
}
impl SlmpWrite {
    ///
    /// Creates new instance of the SlpmRead
    pub fn new(
        parent: impl Into<String>,
        tx_id: usize,
        // name: Name,
        conf: SlmpClientConfig,
        dest: Sender<Point>,
        // diagnosis: Arc<Mutex<IndexMapFxHasher<DiagKeywd, DiagPoint>>>,
        services: Arc<RwLock<Services>>,
        status: Arc<AtomicU32>,
        exit: Arc<ExitNotify>,
    ) -> Self {
        let self_id = format!("{}/SlmpWrite", parent.into());
        let dbs = SlmpRead::build_dbs(&self_id, tx_id, &conf);
        Self {
            tx_id,
            id: self_id,
            // name,
            conf,
            dest,
            dbs: Arc::new(Mutex::new(dbs)),
            // diagnosis,
            services,
            status,
            exit,
        }
    }
    ///
    /// Writes point's to the device,
    pub fn run(&mut self, mut tcp_stream: TcpStream) -> Result<JoinHandle<()>, std::io::Error> {
        info!("{}.run | starting...", self.id);
        let self_id = self.id.clone();
        let tx_id = self.tx_id;
        let status = self.status.clone();
        let exit = self.exit.clone();
        let conf = self.conf.clone();
        let dbs = self.dbs.clone();
        // let diagnosis = self.diagnosis.clone();
        let dest = self.dest.clone();
        let services = self.services.clone();
        let cycle = conf.cycle.map_or(None, |cycle| if cycle != Duration::ZERO {Some(cycle)} else {None});
        match cycle {
            Some(cycle_interval) => {
                info!("{}.run | Preparing thread...", self_id);
                let handle = thread::Builder::new().name(format!("{}.run", self_id)).spawn(move || {
                    let mut is_connected = ChangeNotify::new(
                        &self_id,
                        false,
                        vec![
                            (true,  Box::new(|message| info!("{}", message))),
                            (false, Box::new(|message| warn!("{}", message))),
                        ],
                    );
                    let mut cycle = ServiceCycle::new(&self_id, cycle_interval);
                    let mut dbs = dbs.lock().unwrap();
                    let points = conf.points().iter().map(|point_conf| {
                        SubscriptionCriteria::new(&point_conf.name, Cot::Act)
                    }).collect::<Vec<SubscriptionCriteria>>();
                    let (_, recv) = services.wlock(&self_id).subscribe(&conf.subscribe, &self_id, &points);
                    let mut error_limit = ErrorLimit::new(3);
                    'main: while !exit.get() {
                        is_connected.add(true, format!("{}.run | Connection established", self_id));
                        cycle.start();
                        match recv.recv_timeout(cycle_interval) {
                            Ok(point) => {
                                let point_name = point.name();
                                let point_value = point.value();
                                let db_name = point_name.split('/').nth(3).unwrap();
                                debug!("{}.run | SlmpDb '{}' - writing point '{}'\t({:?})...", self_id, db_name, point_name, point_value);
                                match dbs.get_mut(db_name) {
                                    Some(db) => {
                                        match db.write(&mut tcp_stream, point.clone()) {
                                            Ok(_) => {
                                                error_limit.reset();
                                                debug!("{}.run | SlmpDb '{}' - writing point '{}'\t({:?}) - ok", self_id, db_name, point_name, point_value);
                                                let reply = Self::reply_point(tx_id, point);
                                                match dest.send(reply.clone()) {
                                                    Ok(_) => debug!("{}.run | ProfinetDb '{}' - sent reply: {:#?}", self_id, db_name, reply),
                                                    Err(err) => error!("{}.run | Error sending to queue: {:?}", self_id, err),
                                                    // break 'main;
                                                };
                                            }
                                            Err(err) => {
                                                warn!("{}.run | SlmpDb '{}' - write - error: {:?}", self_id, db_name, err);
                                                if error_limit.add().is_err() {
                                                    error!("{}.run | SlmpDb '{}' - exceeded writing errors limit, trying to reconnect...", self_id, db_name);
                                                    exit.exit_pair();
                                                    status.store(Status::Invalid.into(), Ordering::SeqCst);
                                                    if let Err(err) = dest.send(Point::String(PointHlr::new(
                                                        tx_id,
                                                        &point_name,
                                                        format!("Write error: {}", err),
                                                        Status::Ok,
                                                        Cot::ActErr,
                                                        chrono::offset::Utc::now(),
                                                    ))) {
                                                        error!("{}.run | Error sending to queue: {:?}", self_id, err);
                                                        // break 'main;
                                                    };
                                                    break 'main;
                                                }
                                            }
                                        }
                                    }
                                    None => {
                                        error!("{}.run | SlmpDb '{}' - not found", self_id, db_name);
                                    }
                                }
                            }
                            Err(err) => {
                                match err {
                                    mpsc::RecvTimeoutError::Timeout => {}
                                    mpsc::RecvTimeoutError::Disconnected => {
                                        error!("{}.run | Error receiving from queue: {:?}", self_id, err);
                                        break 'main;
                                    }
                                }
                            }
                        }
                    }
                    info!("{}.run | Exit", self_id);
                });
                info!("{}.run | Started", self.id);
                handle
            }
            None => {
                info!("{}.run | Disabled", self.id);
                thread::Builder::new().name(format!("{}.run", self_id)).spawn(move || {})
            }
        }
    }
    ///
    /// Creates confirmation reply point with the same value & Cot::ActCon
    fn reply_point(tx_id: usize, point: Point) -> Point {
        match point {
            Point::Bool(point) => {
                Point::Bool(PointHlr::new(
                    tx_id,
                    &point.name,
                    point.value,
                    Status::Ok,
                    Cot::ActCon,
                    chrono::offset::Utc::now(),
                ))
            },
            Point::Int(point) => {
                Point::Int(PointHlr::new(
                    tx_id,
                    &point.name,
                    point.value,
                    Status::Ok,
                    Cot::ActCon,
                    chrono::offset::Utc::now(),
                ))
            },
            Point::Real(point) => {
                Point::Real(PointHlr::new(
                    tx_id,
                    &point.name,
                    point.value,
                    Status::Ok,
                    Cot::ActCon,
                    chrono::offset::Utc::now(),
                ))
            },
            Point::Double(point) => {
                Point::Double(PointHlr::new(
                    tx_id,
                    &point.name,
                    point.value,
                    Status::Ok,
                    Cot::ActCon,
                    chrono::offset::Utc::now(),
                ))
            },
            Point::String(point) => {
                Point::String(PointHlr::new(
                    tx_id,
                    &point.name,
                    point.value,
                    Status::Ok,
                    Cot::ActCon,
                    chrono::offset::Utc::now(),
                ))
            },
        }
    }    
}















// ///
// /// Writes point to the current DB
// ///     - Returns Ok() if succeed, Err(message) on fail
// pub fn write(&mut self, tcp_stream: TcpStream) -> Result<JoinHandle<()>, std::io::Error> {
//     let mut message = String::new();
//     match self.points.get(&point.name()) {
//         Some(_parse_point) => {
//             let bytes = match point {
//                 PointType::Bool(point) => {
//                     // !!! Not implemented because before write byte of the bool bits, that byte must be read from device
//                     // let mut buf = [0; 16];
//                     // let index = address.offset.unwrap() as usize;
//                     // buf[index] = point.value.0 as u8;
//                     // client.write(self.number, address.offset.unwrap(), 2, &mut buf)
//                     message = format!("{}.write | Write 'Bool' to the Device - not implemented, point: {:?}", self.id, point.name);
//                     Err(message)
//                 }
//                 PointType::Int(point) => {
//                     match i16::try_from(point.value) {
//                         Ok(value) => {
//                             let write_data = value.to_le_bytes();
//                             match self.slmp_packet.write_packet(FrameType::BinReqSt, &write_data) {
//                                 Ok(write_packet) => Ok(write_packet),
//                                 Err(err) => Err(err),
//                             }
//                         }
//                         Err(err) => {
//                             message = format!("{}.write | Type 'Int' to i16 conversion error: {:#?} in the point: {:#?}", self.id, err, point.name);
//                             Err(message)
//                         }
//                     }
//                 }
//                 PointType::Real(point) => {
//                     let write_data = point.value.to_le_bytes();
//                     match self.slmp_packet.write_packet(FrameType::BinReqSt, &write_data) {
//                         Ok(write_packet) => Ok(write_packet),
//                         Err(err) => Err(err),
//                     }
//                 }
//                 PointType::Double(point) => {
//                     message = format!("{}.write | Write 'Double' to the Device - not implemented, point: {:?}", self.id, point.name);
//                     Err(message)
//                 }
//                 PointType::String(point) => {
//                     message = format!("{}.write | Write 'String' to the Device - not implemented, point: {:?}", self.id, point.name);
//                     Err(message)
//                 }
//             };
//             match bytes {
//                 Ok(bytes) => {
//                     match tcp_stream.write_all(&bytes) {
//                         Ok(_) => Ok(()),
//                         Err(err) => Err(format!("{}.write | Write to socket error: {:#?}", self.id, err)),
//                     }
//                 }
//                 Err(err) => Err(err),
//             }

//         }
//         None => {
//             Err(message)
//         }
//     }
// }