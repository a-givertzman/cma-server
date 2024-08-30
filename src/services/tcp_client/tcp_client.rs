use sal_sync::services::{entity::{name::Name, object::Object, point::point::Point}, service::{service::Service, service_handles::ServiceHandles}};
use std::{collections::HashMap, fmt::Debug, sync::{atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver, Sender}, Arc, Mutex, RwLock}, thread, time::Duration};
use log::{info, warn};
use testing::stuff::wait::WaitTread;
use crate::{
    conf::tcp_client_config::TcpClientConfig,
    core_::net::protocols::jds::{jds_decode_message::JdsDecodeMessage, jds_deserialize::JdsDeserialize, jds_encode_message::JdsEncodeMessage, jds_serialize::JdsSerialize},
    services::{safe_lock::SafeLock, services::Services}, tcp::{
        tcp_client_connect::TcpClientConnect, tcp_read_alive::TcpReadAlive, tcp_stream_write::TcpStreamWrite, tcp_write_alive::TcpWriteAlive
    } 
};
///
/// - Holding single input queue
/// - Received string messages pops from the queue into the end of local buffer
/// - Sending messages (wrapped into ApiQuery) from the beginning of the buffer
/// - Sent messages immediately removed from the buffer
pub struct TcpClient {
    id: String,
    name: Name,
    in_send: HashMap<String, Sender<Point>>,
    in_recv: Vec<Receiver<Point>>,
    conf: TcpClientConfig,
    services: Arc<RwLock<Services>>,
    tcp_recv_alive: Option<Arc<Mutex<TcpReadAlive>>>,
    tcp_send_alive: Option<Arc<Mutex<TcpWriteAlive>>>,
    exit: Arc<AtomicBool>,
}
//
// 
impl TcpClient {
    ///
    /// Creates new instance of [ApiClient]
    /// - [parent] - the ID if the parent entity
    pub fn new(conf: TcpClientConfig, services: Arc<RwLock<Services>>) -> Self {
        let (send, recv) = mpsc::channel();
        Self {
            id: conf.name.join(),
            name: conf.name.clone(),
            in_recv: vec![recv],
            in_send: HashMap::from([(conf.rx.clone(), send)]),
            conf: conf.clone(),
            services,
            tcp_recv_alive: None,
            tcp_send_alive: None,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
}
impl Object for TcpClient {
    fn id(&self) -> &str {
        &self.id
    }
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
            .field("id", &self.id)
            .finish()
    }
}
//
//
unsafe impl Send for TcpClient {}
unsafe impl Sync for TcpClient {}
//
// 
impl Service for TcpClient {
    //
    // 
    fn get_link(&mut self, name: &str) -> Sender<Point> {
        match self.in_send.get(name) {
            Some(send) => send.clone(),
            None => panic!("{}.run | link '{:?}' - not found", self.id, name),
        }
    }
    //
    //
    fn run(&mut self) -> Result<ServiceHandles<()>, String> {
        info!("{}.run | Starting...", self.id);
        let self_id = self.id.clone();
        let conf = self.conf.clone();
        let exit = self.exit.clone();
        let exit_pair = Arc::new(AtomicBool::new(false));
        let tx_send = self.services.rlock(&self_id).get_link(&conf.send_to).unwrap_or_else(|err| {
            panic!("{}.run | services.get_link error: {:#?}", self.id, err);
        });
        let buffered = conf.rx_buffered; // TODO Read this from config
        let in_recv = self.in_recv.pop().unwrap();
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
            Arc::new(Mutex::new(
                JdsDeserialize::new(
                    self_id.clone(),
                    JdsDecodeMessage::new(
                        &self_id,
                    ),
                ),
            )),
            tx_send,
            Some(Duration::from_millis(10)),
            Some(exit.clone()),
            Some(exit_pair.clone()),
        );
        let tcp_write_alive = TcpWriteAlive::new(
            &self_id,
            None,
            Arc::new(Mutex::new(TcpStreamWrite::new(
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
            ))),
            Some(exit.clone()),
            Some(exit_pair.clone()),
        );
        info!("{}.run | Preparing thread...", self_id);
        let handle = thread::Builder::new().name(format!("{}.run", self_id.clone())).spawn(move || {
            info!("{}.run | Preparing thread - ok", self_id);
            loop {
                exit_pair.store(false, Ordering::SeqCst);
                if let Some(tcp_stream) = tcp_client_connect.connect() {
                    let h_r = tcp_read_alive.run(tcp_stream.try_clone().unwrap());
                    let h_w = tcp_write_alive.run(tcp_stream);
                    h_r.wait().unwrap();
                    h_w.wait().unwrap();
                };
                if exit.load(Ordering::SeqCst) {
                    break;
                }
            }
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
        match &self.tcp_recv_alive {
            Some(tcp_recv_alive) => {
                tcp_recv_alive.slock(&self.id).exit()
            }
            None => {}
        }
        match &self.tcp_send_alive {
            Some(tcp_send_alive) => {
                tcp_send_alive.slock(&self.id).exit()
            }
            None => {}
        }
    }
    //
    //
    fn subscribe(&mut self, receiver_id: &str, points: &[sal_sync::services::subscription::subscription_criteria::SubscriptionCriteria]) -> (Sender<Point>, Receiver<Point>) {
        let _ = receiver_id;
        let _ = points;
        std::panic!("{}.subscribe | Does not supported", self.id())
    }
    //
    //
    fn extend_subscription(&mut self, receiver_name: &str, points: &[sal_sync::services::subscription::subscription_criteria::SubscriptionCriteria]) -> Result<(), String> {
        let _ = receiver_name;
        let _ = points;
        std::panic!("{}.extend_subscription | Does not supported", self.id())
    }
    //
    //
    fn unsubscribe(&mut self, receiver_name: &str, points: &[sal_sync::services::subscription::subscription_criteria::SubscriptionCriteria]) -> Result<(), String> {
        let _ = receiver_name;
        let _ = points;
        std::panic!("{}.unsubscribe | Does not supported", self.id())
    }
    //
    //
    fn points(&self) -> Vec<sal_sync::services::entity::point::point_config::PointConfig> {
        std::vec![]
    }
    //
    //
    fn gi(&self, receiver_name: &str, points: &[sal_sync::services::subscription::subscription_criteria::SubscriptionCriteria]) -> Receiver<Point> {
        let _ = receiver_name;
        let _ = points;
        std::panic!("{}.gi | Does not supported", self.id())
    }
}
