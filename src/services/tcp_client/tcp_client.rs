use coco::Stack;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::services::{
    entity::{Name, Object, Point},
    Service, Services,
};
use std::{
    collections::HashMap, fmt::Debug,
    sync::{atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver, Sender}, Arc},
    thread::{self, JoinHandle}, time::Duration,
};
use crate::{
    conf::tcp_client_config::TcpClientConfig,
    core_::{net::protocols::jds::{
        jds_decode_message::JdsDecodeMessage, jds_deserialize::JdsDeserialize,
        jds_encode_message::JdsEncodeMessage, jds_serialize::JdsSerialize,
    }, Mutex},
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
    in_recv: Mutex<Option<Receiver<Point>>>,
    conf: TcpClientConfig,
    services: Arc<Services>,
    handle: Stack<JoinHandle<()>>,
    is_finished: Arc<AtomicBool>,
    exit: Arc<AtomicBool>,
}
//
// 
impl TcpClient {
    ///
    /// Creates new instance of [ApiClient]
    /// - [parent] - the ID if the parent entity
    pub fn new(conf: TcpClientConfig, services: Arc<Services>) -> Self {
        let (send, recv) = mpsc::channel();
        Self {
            dbg: Dbg::new(conf.name.parent(), conf.name.me()),
            name: conf.name.clone(),
            in_recv: Mutex::new(Some(recv)),
            in_send: HashMap::from([(conf.rx.clone(), send)]),
            conf: conf.clone(),
            services,
            handle: Stack::new(),
            is_finished: Arc::new(AtomicBool::new(false)),
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
        let self_id = self.dbg.clone();
        let conf = self.conf.clone();
        let exit = self.exit.clone();
        let exit_pair = Arc::new(AtomicBool::new(false));
        let tx_send = self.services.get_link(&conf.send_to).unwrap_or_else(|err| {
            panic!("{}.run | services.get_link error: {:#?}", self.dbg, err);
        });
        let buffered = conf.rx_buffered; // TODO Read this from config
        let in_recv = self.in_recv.lock().take().unwrap();
        // let (cyclic, cycleInterval) = match conf.cycle {
        //     Some(interval) => (interval > Duration::ZERO, interval),
        //     None => (false, Duration::ZERO),
        // };
        let reconnect = conf.reconnect_cycle.unwrap_or(Duration::from_secs(3));
        let mut tcp_client_connect = TcpClientConnect::new(
            self_id.clone(), 
            conf.address, 
            reconnect,
            Some(exit.clone())
        );
        let mut tcp_read_alive = TcpReadAlive::new(
            &self_id,
            Box::new(
                JdsDeserialize::new(
                    self_id.clone(),
                    JdsDecodeMessage::new(
                        &self_id,
                    ),
                ),
            ),
            tx_send,
            Some(Duration::from_millis(10)),
            Some(exit.clone()),
            Some(exit_pair.clone()),
        );
        let mut tcp_write_alive = TcpWriteAlive::new(
            &self_id,
            None,
            TcpStreamWrite::new(
                &self_id,
                buffered,
                Some(conf.rx_max_len as usize),
                Box::new(JdsEncodeMessage::new(
                    &self_id,
                    JdsSerialize::new(
                        &self_id,
                        in_recv,
                    ),
                )),
            ),
            Some(exit.clone()),
            Some(exit_pair.clone()),
        );
        log::info!("{}.run | Preparing thread...", self_id);
        let handle = thread::Builder::new().name(format!("{}.run", self_id.clone())).spawn(move || {
            log::info!("{}.run | Preparing thread - ok", self_id);
            loop {
                exit_pair.store(false, Ordering::SeqCst);
                if let Some(tcp_stream) = tcp_client_connect.connect() {
                    let h_r = tcp_read_alive.run(tcp_stream.try_clone().unwrap());
                    let h_w = tcp_write_alive.run(tcp_stream);
                    h_r.join().unwrap();
                    h_w.join().unwrap();
                };
                if exit.load(Ordering::SeqCst) {
                    break;
                }
            }
            log::info!("{}.run | Exit", self_id);
        });
        match handle {
            Ok(handle) => {
                log::info!("{}.run | Starting - ok", self.dbg);
                self.handle.push(handle);
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
    fn points(&self) -> Vec<sal_sync::services::entity::PointConfig> {
        std::vec![]
    }
    //
    //
    fn gi(&self, receiver_name: &str, points: &[sal_sync::services::SubscriptionCriteria]) -> Receiver<Point> {
        let _ = receiver_name;
        let _ = points;
        std::panic!("{}.gi | Does not supported", self.dbg)
    }
}
