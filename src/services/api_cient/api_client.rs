use concat_string::concat_string;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{entity::{Cot, Name, Object, Point, PointHlr, PointTxId, Status}, Service, ServiceCycle, Services}, sync::{channel::{self, Receiver, Sender}, Handles, Owner}, thread_pool::Scheduler};
use std::{collections::HashMap, fmt::Debug, sync::{atomic::{AtomicBool, Ordering}, Arc}, time::Duration};
use api_tools::{api::reply::api_reply::ApiReply, client::{api_query::{ApiQuery, ApiQueryKind, ApiQuerySql}, api_request::ApiRequest}};
use crate::{
    conf::api_client_conf::ApiClientConf, 
    domain::retain_buffer::retain_buffer::RetainBuffer,
};
///
/// ### Sending data to the API
/// 
/// - Holding single input queue
/// - Received string events (containig SQL) pops from the queue into the end of local buffer
/// - Sending SQL's (wrapped into ApiQuery) from the beginning of the buffer
/// - Sent SQL's immediately removed from the buffer
/// - Replies from SQL requests can be returned with same event name, if **`send-to`** is specified
pub struct ApiClient {
    name: Name,
    txid: usize,
    recv: Owner<Receiver<Point>>,
    send: HashMap<String, Sender<Point>>,
    conf: ApiClientConf,
    services: Arc<Services>,
    scheduler: Scheduler,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
//
// 
impl ApiClient {
    ///
    /// Creates new instance of [ApiClient]
    /// - [parent] - the ID if the parent entity
    pub fn new(conf: ApiClientConf, services: Arc<Services>, scheduler: Scheduler) -> Self {
        let (send, recv) = channel::unbounded();
        let dbg = Dbg::new(conf.name.parent(), conf.name.me());
        Self {
            name: conf.name.clone(),
            txid: PointTxId::from_str(&conf.name.join()),
            recv: Owner::new(recv),
            send: HashMap::from([(conf.rx.clone(), send)]),
            conf: conf.clone(),
            services,
            scheduler,
            handles: Handles::new(&dbg),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
    ///
    /// Reads all avalible at the moment items from the in-queue
    fn read_queue(dbg: &Dbg, recv: &Receiver<Point>, buffer: &mut RetainBuffer<Point>) {
        let max_read_at_once = 1000;
        if !recv.is_empty() {
            for _ in 0..max_read_at_once {
                match recv.try_recv() {
                    Ok(point) => match point {
                        Some(point) => {
                            log::trace!("{}.read_queue | point: {:?}", dbg, &point);
                            buffer.push(point);
                        }
                        None => return,
                    }
                    Err(_) => return,
                }
            }
        }
    }
    ///
    /// Sending SQL queries to the Database
    fn send(dbg: &Dbg, request: &mut ApiRequest, database: &str, sql: String, keep_alive: bool) -> Result<ApiReply, Error> {
        let error = Error::new(dbg, "send");
        let query = ApiQuery::new(
            ApiQueryKind::Sql(ApiQuerySql::new(database, sql)),
            true,
        );
        match request.fetch_with(&query, keep_alive) {
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
                        Err(error.pass_with(format!("Error parsing API reply: {:?}", reply), err.to_string()))
                    }
                }
            }
            Err(err) => {
                Err(error.pass_with("Error sending API request", err))
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
        let txid = self.txid;
        let exit = self.exit.clone();
        let conf = self.conf.clone();
        let recv = self.recv.take().unwrap();
        let (cyclic, cycle_interval) = match conf.cycle {
            Some(interval) => (interval > Duration::ZERO, interval),
            None => (false, Duration::ZERO),
        };
        let send_to = match conf.send_to {
            Some(send_to) => self.services
                .get_link(&send_to)
                .inspect_err(|err| log::warn!("{}.run | Link {} - Not found, error: {}", dbg, send_to.name(), err)).ok(),
            None => None,
        };
        // let reconnect = if conf.reconnectCycle.is_some() {conf.reconnectCycle.unwrap()} else {Duration::from_secs(3)};
        let _queue_max_length = conf.rx_max_len;
        let handle = self.scheduler.spawn(move || {
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
                                Point::Bool(_) => log::warn!("{}.run | Invalid point type 'Bool' (expected 'String' containing SQL) in: {:?}", dbg, point),
                                Point::Int(_) => log::warn!("{}.run | Invalid point type 'Int' (expected 'String' containing SQL) in: {:?}", dbg, point),
                                Point::Real(_) => log::warn!("{}.run | Invalid point type 'Real' (expected 'String' containing SQL) in: {:?}", dbg, point),
                                Point::Double(_) => log::warn!("{}.run | Invalid point type 'Double' (expected 'String' containing SQL) in: {:?}", dbg, point),
                                Point::String(point) => {
                                    let sql = point.value.clone();
                                    match Self::send(&dbg, &mut request, &conf.database, sql, api_keep_alive) {
                                        Ok(reply) => {
                                            if reply.has_error() {
                                                if let Some(send_to) = &send_to {
                                                    match serde_json::to_string(&reply.error) {
                                                        Ok(reply) => {
                                                            if let Err(err) = send_to.send(Point::String(PointHlr::new(
                                                                txid,
                                                                &point.name,
                                                                reply,
                                                                Status::Ok,
                                                                Cot::ReqErr,
                                                                chrono::Utc::now(),
                                                            ))) {
                                                                log::warn!("{}.run | Send API reply error: {:?}", dbg, err);
                                                            }
                                                        }
                                                        Err(err) => log::warn!("{}.run | Parse API reply error: {:?}", dbg, err),
                                                    }
                                                }
                                                log::warn!("{}.run | API reply has error: {:?}", dbg, reply.error);
                                            } else {
                                                if let Some(send_to) = &send_to {
                                                    match serde_json::to_string(&reply.data) {
                                                        Ok(reply) => {
                                                            if let Err(err) = send_to.send(Point::String(PointHlr::new(
                                                                txid,
                                                                &point.name,
                                                                reply,
                                                                Status::Ok,
                                                                Cot::ReqCon,
                                                                chrono::Utc::now(),
                                                            ))) {
                                                                log::warn!("{}.run | Send API reply error: {:?}", dbg, err);
                                                            }
                                                        }
                                                        Err(err) => log::warn!("{}.run | Parse API reply error: {:?}", dbg, err),
                                                    }
                                                }
                                            }
                                            buffer.pop_first();
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
            log::info!("{}.run | Exit", dbg);
            Ok(())
        });
        match handle {
            Ok(handle) => {
                log::info!("{}.run | Starting - ok", self.dbg);
                self.handles.push(handle);
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