use coco::Stack;
use concat_string::concat_string;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::services::{entity::{Name, Object, Point}, service::{Service, ServiceCycle}};
use std::{collections::HashMap, fmt::Debug, sync::{atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver, Sender}, Arc, Mutex}, thread::{self, JoinHandle}, time::Duration};
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
    dbg: Dbg,
    name: Name,
    recv: Mutex<Option<Receiver<Point>>>,
    send: HashMap<String, Sender<Point>>,
    conf: ApiClientConfig,
    handle: Stack<JoinHandle<()>>,
    is_finished: Arc<AtomicBool>,
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
            dbg: Dbg::new(conf.name.parent(), conf.name.me()),
            name: conf.name.clone(),
            recv: Mutex::new(Some(recv)),
            send: HashMap::from([(conf.rx.clone(), send)]),
            conf: conf.clone(),
            handle: Stack::new(),
            is_finished: Arc::new(AtomicBool::new(false)),
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// Reads all avalible at the moment items from the in-queue
    fn read_queue(dbg: &Dbg, recv: &Receiver<Point>, buffer: &mut RetainBuffer<Point>) {
        let max_read_at_once = 1000;
        for (index, point) in recv.try_iter().enumerate() {   
            log::debug!("{}.read_queue | point: {:?}", dbg, &point);
            buffer.push(point);
            if index > max_read_at_once {
                break;
            }                 
        }
    }
    ///
    /// Writing sql string to the TcpStream
    fn send(dbg: &Dbg, request: &mut ApiRequest, database: &str, sql: String, keep_alive: bool) -> Result<ApiReply, String> {
        let query = ApiQuery::new(
            ApiQueryKind::Sql(ApiQuerySql::new(database, sql)),
            true,
        );
        match request.fetch(&query, keep_alive) {
            Ok(reply) => {
                if log::max_level() > log::LevelFilter::Info {
                    let reply_str = std::str::from_utf8(&reply).unwrap();
                    log::trace!("{}.send | reply str: {:?}", dbg, reply_str);
                }
                match serde_json::from_slice(&reply) {
                    Ok(reply) => Ok(reply),
                    Err(err) => {
                        let reply = match std::str::from_utf8(&reply) {
                            Ok(reply) => reply.to_string(),
                            Err(err) => concat_string!(dbg, ".send | Error parsing reply to utf8 string: ", err.to_string()),
                        };
                        let message = concat_string!(dbg, ".send | Error parsing API reply: {:?} \n\t reply was: {:?}", err.to_string(), reply);
                        log::warn!("{}", message);
                        Err(message)
                    }
                }
            }
            Err(err) => {
                let message = concat_string!(dbg, ".send | Error sending API request: {:?}", err);
                log::warn!("{}", message);
                Err(message)
            }
        }
    }
}
//
// 
impl Object for ApiClient {
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
            .field("id", &self.dbg)
            .finish()
    }
}
//
//
impl Service for ApiClient {
    //
    //
    fn get_link(&self, name: &str) -> Sender<Point> {
        match self.send.get(name) {
            Some(send) => send.clone(),
            None => panic!("{}.run | link '{:?}' - not found", self.dbg, name),
        }
    }
    //
    // 
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let is_finished = self.is_finished.clone();
        let exit = self.exit.clone();
        let conf = self.conf.clone();
        let recv = self.recv.lock().unwrap().take().unwrap();
        let (cyclic, cycle_interval) = match conf.cycle {
            Some(interval) => (interval > Duration::ZERO, interval),
            None => (false, Duration::ZERO),
        };
        // let reconnect = if conf.reconnectCycle.is_some() {conf.reconnectCycle.unwrap()} else {Duration::from_secs(3)};
        let _queue_max_length = conf.rx_max_len;
        let handle = thread::Builder::new().name(dbg.to_string()).spawn(move || {
            let mut buffer = RetainBuffer::new(&dbg, "", Some(conf.rx_max_len as usize));
            let mut cycle = ServiceCycle::new(&dbg, cycle_interval);
            // let mut connect = TcpClientConnect::new(self_id.clone() + "/TcpSocketClientConnect", conf.address, reconnect);
            let api_keep_alive = true;
            let sql_keep_alive = true;
            let mut request = ApiRequest::new(
                &dbg, 
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
                log::trace!("{}.run | Step...", dbg);
                Self::read_queue(&dbg, &recv, &mut buffer);
                log::trace!("{}.run | Beffer.len: {}", dbg, buffer.len());
                let mut count = buffer.len();
                while count > 0 {
                    match buffer.first() {
                        Some(point) => {
                            match point {
                                Point::Bool(_) => log::warn!("{}.run | Invalid point type 'Bool' in: {:?}", dbg, point),
                                Point::Int(_) => log::warn!("{}.run | Invalid point type 'Int' in: {:?}", dbg, point),
                                Point::Real(_) => log::warn!("{}.run | Invalid point type 'Real' in: {:?}", dbg, point),
                                Point::Double(_) => log::warn!("{}.run | Invalid point type 'Double' in: {:?}", dbg, point),
                                Point::String(point) => {
                                    let sql = point.value.clone();
                                    match Self::send(&dbg, &mut request, &conf.database, sql, api_keep_alive) {
                                        Ok(reply) => {
                                            if reply.has_error() {
                                                log::warn!("{}.run | API reply has error: {:?}", dbg, reply.error);
                                            } else {
                                                buffer.pop_first();
                                            }
                                        }
                                        Err(err) => {
                                            log::warn!("{}.run | Error: {:?}", dbg, err);
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
                log::trace!("{}.run | Step - done ({:?})", dbg, cycle.elapsed());
                if cyclic {
                    cycle.wait();
                }
            };
            is_finished.store(true, Ordering::SeqCst);
            log::info!("{}.run | Exit", dbg);
        });
        match handle {
            Ok(handle) => {
                log::info!("{}.run | Starting - ok", self.dbg);
                self.handle.push(handle);
                Ok(())
            }
            Err(err) => {
                let message = format!("{}.run | Start failed: {:#?}", self.dbg, err);
                log::warn!("{}", message);
                Err(Error::new(&self.dbg, "run").pass_with("Start failed", err.to_string()))
            }
        }
    }
    //
    //
    fn wait(&self) -> Result<(), Error> {
        while !self.handle.is_empty() {
            if let Some(handle) = self.handle.pop() {
                if let Err(err) = handle.join() {
                    log::warn!("{}.wait | Error: {:?}", self.dbg, err);
                    return Err(Error::new(&self.dbg, "wait").pass(format!("{:?}", err)));
                }
            }
        }
        self.is_finished.store(true, Ordering::SeqCst);
        Ok(())
    }
    //
    //
    fn is_finished(&self) -> bool {
        self.is_finished.load(Ordering::SeqCst)
    }
    //
    // 
    fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }
}