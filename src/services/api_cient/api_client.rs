use concat_string::concat_string;
use log::{info, debug, trace, warn};
use sal_sync::services::{entity::{name::Name, object::Object, point::point::Point}, service::{service::Service, service_cycle::ServiceCycle, service_handles::ServiceHandles}};
use std::{collections::HashMap, fmt::Debug, sync::{atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver, Sender}, Arc, Mutex}, thread, time::Duration};
use api_tools::{api::reply::api_reply::ApiReply, client::{api_query::{ApiQuery, ApiQueryKind, ApiQuerySql}, api_request::ApiRequest}};
use crate::{
    conf::api_client_config::ApiClientConfig, 
    core_::retain_buffer::retain_buffer::RetainBuffer,
};
///
/// - Holding single input queue
/// - Received string messages pops from the queue into the end of local buffer
/// - Sending messages (wrapped into ApiQuery) from the beginning of the buffer
/// - Sent messages immediately removed from the buffer
pub struct ApiClient {
    id: String,
    name: Name,
    recv: Mutex<Option<Receiver<Point>>>,
    send: HashMap<String, Sender<Point>>,
    conf: ApiClientConfig,
    exit: Arc<AtomicBool>,
}
//
// 
impl ApiClient {
    ///
    /// Creates new instance of [ApiClient]
    /// - [parent] - the ID if the parent entity
    pub fn new(conf: ApiClientConfig) -> Self {
        let (send, recv) = mpsc::channel();
        Self {
            id: format!("{}", conf.name),
            name: conf.name.clone(),
            recv: Mutex::new(Some(recv)),
            send: HashMap::from([(conf.rx.clone(), send)]),
            conf: conf.clone(),
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// Reads all avalible at the moment items from the in-queue
    fn read_queue(self_id: &str, recv: &Receiver<Point>, buffer: &mut RetainBuffer<Point>) {
        let max_read_at_once = 1000;
        for (index, point) in recv.try_iter().enumerate() {   
            debug!("{}.read_queue | point: {:?}", self_id, &point);
            buffer.push(point);
            if index > max_read_at_once {
                break;
            }                 
        }
    }
    ///
    /// Writing sql string to the TcpStream
    fn send(self_id: &str, request: &mut ApiRequest, database: &str, sql: String, keep_alive: bool) -> Result<ApiReply, String> {
        let query = ApiQuery::new(
            ApiQueryKind::Sql(ApiQuerySql::new(database, sql)),
            true,
        );
        match request.fetch(&query, keep_alive) {
            Ok(reply) => {
                if log::max_level() > log::LevelFilter::Info {
                    let reply_str = std::str::from_utf8(&reply).unwrap();
                    trace!("{}.send | reply str: {:?}", self_id, reply_str);
                }
                match serde_json::from_slice(&reply) {
                    Ok(reply) => Ok(reply),
                    Err(err) => {
                        let reply = match std::str::from_utf8(&reply) {
                            Ok(reply) => reply.to_string(),
                            Err(err) => concat_string!(self_id, ".send | Error parsing reply to utf8 string: ", err.to_string()),
                        };
                        let message = concat_string!(self_id, ".send | Error parsing API reply: {:?} \n\t reply was: {:?}", err.to_string(), reply);
                        warn!("{}", message);
                        Err(message)
                    }
                }
            }
            Err(err) => {
                let message = concat_string!(self_id, ".send | Error sending API request: {:?}", err);
                warn!("{}", message);
                Err(message)
            }
        }
    }
}
//
// 
impl Object for ApiClient {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl Debug for ApiClient {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ApiClient")
            .field("id", &self.id)
            .finish()
    }
}
//
//
impl Service for ApiClient {
    //
    //
    fn get_link(&mut self, name: &str) -> Sender<Point> {
        match self.send.get(name) {
            Some(send) => send.clone(),
            None => panic!("{}.run | link '{:?}' - not found", self.id, name),
        }
    }
    //
    // 
    fn run(&mut self) -> Result<ServiceHandles<()>, String> {
        info!("{}.run | Starting...", self.id);
        let self_id = self.id.clone();
        let exit = self.exit.clone();
        let conf = self.conf.clone();
        let recv = self.recv.lock().unwrap().take().unwrap();
        let (cyclic, cycle_interval) = match conf.cycle {
            Some(interval) => (interval > Duration::ZERO, interval),
            None => (false, Duration::ZERO),
        };
        // let reconnect = if conf.reconnectCycle.is_some() {conf.reconnectCycle.unwrap()} else {Duration::from_secs(3)};
        let _queue_max_length = conf.rx_max_len;
        let handle = thread::Builder::new().name(self_id.clone()).spawn(move || {
            let mut buffer = RetainBuffer::new(&self_id, "", Some(conf.rx_max_len as usize));
            let mut cycle = ServiceCycle::new(&self_id, cycle_interval);
            // let mut connect = TcpClientConnect::new(self_id.clone() + "/TcpSocketClientConnect", conf.address, reconnect);
            let api_keep_alive = true;
            let sql_keep_alive = true;
            let mut request = ApiRequest::new(
                &self_id, 
                conf.address, 
                conf.auth_token, 
                ApiQuery::new(
                    ApiQueryKind::Sql(ApiQuerySql::new(&conf.database, "select 1;")), 
                    sql_keep_alive,
                ),
                api_keep_alive, 
                conf.debug,
            );
            'send: loop {
                cycle.start();
                trace!("{}.run | Step...", self_id);
                Self::read_queue(&self_id, &recv, &mut buffer);
                trace!("{}.run | Beffer.len: {}", self_id, buffer.len());
                let mut count = buffer.len();
                while count > 0 {
                    match buffer.first() {
                        Some(point) => {
                            match point {
                                Point::Bool(_) => warn!("{}.run | Invalid point type 'Bool' in: {:?}", self_id, point),
                                Point::Int(_) => warn!("{}.run | Invalid point type 'Int' in: {:?}", self_id, point),
                                Point::Real(_) => warn!("{}.run | Invalid point type 'Real' in: {:?}", self_id, point),
                                Point::Double(_) => warn!("{}.run | Invalid point type 'Double' in: {:?}", self_id, point),
                                Point::String(point) => {
                                    let sql = point.value.clone();
                                    match Self::send(&self_id, &mut request, &conf.database, sql, api_keep_alive) {
                                        Ok(reply) => {
                                            if reply.has_error() {
                                                warn!("{}.run | API reply has error: {:?}", self_id, reply.error);
                                            } else {
                                                buffer.pop_first();
                                            }
                                        }
                                        Err(err) => {
                                            warn!("{}.run | Error: {:?}", self_id, err);
                                        }
                                    }
                                }
                            }
                        }
                        None => {break;}
                    };
                    count -=1;
                }
                if exit.load(Ordering::SeqCst) {
                    break 'send;
                }
                trace!("{}.run | Step - done ({:?})", self_id, cycle.elapsed());
                if cyclic {
                    cycle.wait();
                }
            };            
            info!("{}.run | Exit", self_id);
        });
        match handle {
            Ok(handle) => {
                info!("{}.run | Starting - ok", self.id);
                Ok(ServiceHandles::new(vec![(self.id.clone(), handle)]))
            }
            Err(err) => {
                let message = format!("{}.run | Start failed: {:#?}", self.id, err);
                warn!("{}", message);
                Err(message)
            }
        }
    }
    //
    // 
    fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }
}