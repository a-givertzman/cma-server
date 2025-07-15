use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{
    entity::{Name, Object, Point}, future::Future, Service, Services
}, sync::{channel::{self, Receiver, Sender}, Handles, Owner}, thread_pool::Scheduler};
use std::{
    collections::HashMap, fmt::Debug,
    sync::{atomic::{AtomicBool, Ordering}, Arc},
    time::Duration,
};
use crate::{
    conf::tcp_client_config::TcpClientConfig,
    domain::net::protocols::jds::{
        jds_decode_message::JdsDecodeMessage, jds_deserialize::JdsDeserialize,
        jds_encode_message::JdsEncodeMessage, jds_serialize::JdsSerialize,
    },
    tcp::{
        tcp_client_connect::TcpClientConnect, tcp_read_alive::TcpReadAlive,
        tcp_stream_write::TcpStreamWrite, tcp_write_alive::TcpWriteAlive,
    } 
};
///
/// - Holding single input queue
/// - Received string messages pops from the queue into the end of local buffer
/// - Sending messages (wrapped into ApiQuery) from the beginning of the buffer
/// - Sent messages immediately removed from the buffer
pub struct TcpClient {
    dbg: Dbg,
    name: Name,
    in_send: HashMap<String, Sender<Point>>,
    in_recv: Owner<Receiver<Point>>,
    conf: TcpClientConfig,
    services: Arc<Services>,
    scheduler: Scheduler,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
}
//
// 
impl TcpClient {
    ///
    /// Creates new instance of [ApiClient]
    /// - [parent] - the ID if the parent entity
    pub fn new(conf: TcpClientConfig, services: Arc<Services>, scheduler: Scheduler) -> Self {
        let (send, recv) = channel::unbounded();
        let dbg = Dbg::new(conf.name.parent(), conf.name.me());
        Self {
            name: conf.name.clone(),
            in_recv: Owner::new(recv),
            in_send: HashMap::from([(conf.rx.clone(), send)]),
            conf: conf.clone(),
            services,
            scheduler,
            handles: Handles::new(&dbg),
            dbg,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
}
impl Object for TcpClient {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl Debug for TcpClient {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TcpClient")
            .field("id", &self.dbg)
            .finish()
    }
}
//
//
impl Service for TcpClient {
    //
    // 
    fn get_link(&self, name: &str) -> Sender<Point> {
        match self.in_send.get(name) {
            Some(send) => send.clone(),
            None => panic!("{}.run | link '{:?}' - not found", self.dbg, name),
        }
    }
    //
    //
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let conf = self.conf.clone();
        let exit = self.exit.clone();
        let exit_pair = Arc::new(AtomicBool::new(false));
        let tx_send = self.services.get_link(&conf.send_to).unwrap_or_else(|err| {
            panic!("{}.run | services.get_link error: {:#?}", self.dbg, err);
        });
        let buffered = conf.rx_buffered; // TODO Read this from config
        let in_recv = self.in_recv.take().unwrap();
        // let (cyclic, cycleInterval) = match conf.cycle {
        //     Some(interval) => (interval > Duration::ZERO, interval),
        //     None => (false, Duration::ZERO),
        // };
        let reconnect = conf.reconnect_cycle.unwrap_or(Duration::from_secs(3));
        let mut tcp_client_connect = TcpClientConnect::new(
            dbg.clone(), 
            conf.address, 
            reconnect,
            Some(exit.clone())
        );
        let tcp_read_alive = TcpReadAlive::new(
            &dbg,
            Box::new(
                JdsDeserialize::new(
                    dbg.clone(),
                    JdsDecodeMessage::new(
                        &dbg,
                    ),
                ),
            ),
            tx_send,
            Some(Duration::from_millis(10)),
            Some(exit.clone()),
            Some(exit_pair.clone()),
            Some(self.scheduler.clone()),
        );
        let tcp_write_alive = TcpWriteAlive::new(
            &dbg,
            None,
            TcpStreamWrite::new(
                &dbg,
                buffered,
                Some(conf.rx_max_len as usize),
                Box::new(JdsEncodeMessage::new(
                    &dbg,
                    JdsSerialize::new(
                        &dbg,
                        in_recv,
                    ),
                )),
            ),
            Some(exit.clone()),
            Some(exit_pair.clone()),
            Some(self.scheduler.clone()),
        );
        log::info!("{}.run | Preparing thread...", dbg);
        let handle = self.scheduler.spawn(move || {
            log::info!("{}.run | Preparing thread - ok", dbg);
            loop {
                exit_pair.store(false, Ordering::SeqCst);
                if let Some(tcp_stream) = tcp_client_connect.connect() {
                    let read = tcp_read_alive.run(tcp_stream.try_clone().unwrap());
                    let write = tcp_write_alive.run(tcp_stream);
                    match (read, write) {
                        (Ok(_), Ok(_)) => {}
                        (Ok(_), Err(err)) => log::error!("{}.run | Error: {:?}", dbg, err),
                        (Err(err), Ok(_)) => log::error!("{}.run | Error: {:?}", dbg, err),
                        (Err(err1), Err(err2)) => log::error!("{}.run | Errors: \n\t{:?},\n\t{:?}", dbg, err1, err2),
                    }
                    if let Err(err) = tcp_read_alive.wait() {
                        log::error!("{}.run | Error wait for TcpReadAlive: {:?}", dbg, err);
                    }
                    if let Err(err) = tcp_write_alive.wait() {
                        log::error!("{}.run | Error wait for TcpWriteAlive: {:?}", dbg, err);
                    }
                };
                if exit.load(Ordering::SeqCst) {
                    break;
                }
            }
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
    //
    //
    fn subscribe(&self, receiver_id: &str, points: &[sal_sync::services::SubscriptionCriteria]) -> (Sender<Point>, Receiver<Point>) {
        let _ = receiver_id;
        let _ = points;
        std::panic!("{}.subscribe | Does not supported", self.dbg)
    }
    //
    //
    fn extend_subscription(&self, receiver_name: &str, points: &[sal_sync::services::SubscriptionCriteria]) -> Result<(), Error> {
        let _ = receiver_name;
        let _ = points;
        std::panic!("{}.extend_subscription | Does not supported", self.dbg)
    }
    //
    //
    fn unsubscribe(&self, receiver_name: &str, points: &[sal_sync::services::SubscriptionCriteria]) -> Result<(), Error> {
        let _ = receiver_name;
        let _ = points;
        std::panic!("{}.unsubscribe | Does not supported", self.dbg)
    }
    //
    //
    fn points(&self) -> Vec<sal_sync::services::entity::PointConf> {
        std::vec![]
    }
    //
    //
    fn gi(&self, receiver_name: &str, points: &[sal_sync::services::SubscriptionCriteria]) -> Future<Vec<Point>> {
        let _ = receiver_name;
        let _ = points;
        std::panic!("{}.gi | Does not supported", self.dbg)
    }
}
