use coco::Stack;
use sal_core::error::Error;
use sal_sync::services::{entity::{Name, Object, {{Point, ToPoint}, PointTxId}}, safe_lock::rwlock::SafeLock, service::{LinkName, Service, ServiceCycle}, services::Services};
use std::{fmt::Debug, str::FromStr, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, Arc, RwLock}, thread::{self, JoinHandle}, time::Duration};
use testing::entities::test_value::Value;
///
/// 
pub struct TaskTestProducer {
    id: String,
    name: Name,
    send_to: LinkName, 
    cycle: Duration,
    // rxSend: HashMap<String, Sender<PointType>>,
    services: Arc<RwLock<Services>>,
    test_data: Vec<Value>,
    sent: Arc<RwLock<Vec<Point>>>,
    handle: Stack<JoinHandle<()>>,
    exit: Arc<AtomicBool>,
}
//
// 
impl TaskTestProducer {
    /// Creates new instance TaskTestReceiver
    /// - `send_to` - name of the link (Service.in-queue) used for sending Point's
    /// - `cycle` - Duration of each send cycle
    /// - `test_data` - Value's to be produced
    #[allow(unused)]
    pub fn new(parent: &str, send_to: &str, cycle: Duration, services: Arc<RwLock<Services>>, test_data: Vec<Value>) -> Self {
        let name = Name::new(parent, format!("TaskTestProducer{}", COUNT.fetch_add(1, Ordering::Relaxed)));
        Self {
            id: name.join(),
            name,
            send_to: LinkName::from_str(send_to).unwrap(),
            cycle,
            // rxSend: HashMap::new(),
            services,
            test_data,
            sent: Arc::new(RwLock::new(vec![])),
            handle: Stack::new(),
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// Returns vector of sent Pont's
    #[allow(unused)]
    pub fn sent(&self) -> Arc<RwLock<Vec<Point>>> {
        self.sent.clone()
    }
}
//
// 
impl Object for TaskTestProducer {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl Debug for TaskTestProducer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TaskTestProducer")
            .field("id", &self.id)
            .finish()
    }
}
//
//
impl Service for TaskTestProducer {
    //
    // 
    fn run(&mut self) -> Result<(), Error> {
        let self_id = self.id.clone();
        let tx_id = PointTxId::from_str(&self_id);
        let mut cycle = ServiceCycle::new(&self_id, self.cycle);
        let delayed = !cycle.interval().is_zero();
        let tx_send = self.services.rlock(&self_id).get_link(&self.send_to).unwrap_or_else(|err| {
            panic!("{}.run | services.get_link error: {:#?}", self.id, err);
        });
        let sent = self.sent.clone();
        let test_data = self.test_data.clone();
        let handle = thread::Builder::new().name(self_id.clone()).spawn(move || {
            log::debug!("{}.run | calculating step...", self_id);
            for value in test_data {
                cycle.start();
                let point = value.to_point(tx_id, "/path/Point.Name");
                match tx_send.send(point.clone()) {
                    Ok(_) => {
                        sent.write().unwrap().push(point.clone());
                        log::trace!("{}.run | sent points: {:?}", self_id, sent.read().unwrap().len());
                    }
                    Err(err) => {
                        log::warn!("{}.run | Error write to queue: {:?}", self_id, err);
                    }
                }
                if delayed {
                    cycle.wait();
                }
            }
            log::info!("{}.run | All sent: {}", self_id, sent.read().unwrap().len());
            // thread::sleep(Duration::from_secs_f32(0.1));
            // debug!("TaskTestProducer({}).run | calculating step - done ({:?})", name, cycle.elapsed());
        });
        match handle {
            Ok(handle) => {
                log::info!("{}.run | Started", self.id);
                self.handle.push(handle);
                Ok(())
                        }
            Err(err) => {
                let err = Error::new(&self.id, "run").pass_with("Start failed", err.to_string());
                log::warn!("{}", err);
                Err(err)
            }
        }
    }
    //
    //
    fn wait(&self) -> sal_sync::services::future::Future<()> {
        let dbg = self.id.clone();
        let (future, sink) = sal_sync::services::future::Future::new();
        if let Some(handle) = self.handle.pop() {
            std::thread::spawn(move|| {
                if let Err(err) = handle.join() {
                    log::warn!("{dbg}.wait | Error: {:?}", err);
                }
                sink.add(());
            });
        }
        future
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::Relaxed);
    }
}
///
/// Global static counter of FnOut instances
static COUNT: AtomicUsize = AtomicUsize::new(0);
