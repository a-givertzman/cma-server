use std::{collections::HashMap, fmt::Debug, str::FromStr, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, Arc, RwLock}, thread::{self, JoinHandle}, time::Duration};
use coco::Stack;
use sal_core::error::Error;
use sal_sync::services::{entity::{Name, Object, Point, ToPoint, PointConfig, PointTxId}, safe_lock::rwlock::SafeLock, service::{LinkName, Service}, services::Services};
use testing::entities::test_value::Value;

///
/// 
pub struct TaskTestProducer {
    dbg: String,
    name: Name,
    send_to: LinkName, 
    cycle: Duration,
    // rxSend: HashMap<String, Sender<PointType>>,
    services: Arc<RwLock<Services>>,
    test_data: Vec<(String, Value)>,
    sent: Arc<RwLock<Vec<Point>>>,
    handle: Stack<JoinHandle<()>>,
    is_finished: Arc<AtomicBool>,
    exit: Arc<AtomicBool>,
}
//
// 
impl TaskTestProducer {
    pub fn new(parent: &str, send_to: &str, cycle: Duration, services: Arc<RwLock<Services>>, test_data: &[(String, Value)]) -> Self {
        let name = Name::new(parent, format!("TaskTestProducer{}", COUNT.fetch_add(1, Ordering::Relaxed)));
        Self {
            dbg: name.join(),
            name,
            send_to: LinkName::from_str(send_to).unwrap(),
            cycle,
            // rxSend: HashMap::new(),
            services,
            test_data: test_data.to_vec(),
            sent: Arc::new(RwLock::new(vec![])),
            handle: Stack::new(),
            is_finished: Arc::new(AtomicBool::new(false)),
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// 
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
            .field("id", &self.dbg)
            .finish()
    }
}
//
//
impl Service for TaskTestProducer {
    //
    // 
    fn run(&self) -> Result<(), Error> {
        let self_id = self.dbg.clone();
        let tx_id = PointTxId::from_str(&self_id);
        let cycle = self.cycle;
        let delayed = !cycle.is_zero();
        let tx_send = self.services.rlock(&self_id).get_link(&self.send_to).unwrap_or_else(|err| {
            panic!("{}.run | services.get_link error: {:#?}", self.dbg, err);
        });
        let sent = self.sent.clone();
        let test_data = self.test_data.clone();
        let handle = thread::Builder::new().name(self_id.clone()).spawn(move || {
            log::debug!("{}.run | calculating step...", self_id);
            for (name, value) in test_data {
                let point = value.to_point(tx_id, &name);
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
                    thread::sleep(cycle);
                }
            }
            log::info!("{}.run | All sent: {}", self_id, sent.read().unwrap().len());
            // thread::sleep(Duration::from_secs_f32(0.1));
            // debug!("TaskTestProducer({}).run | calculating step - done ({:?})", name, cycle.elapsed());
        });
        match handle {
            Ok(handle) => {
                log::info!("{}.run | Started", self.dbg);
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
    // Returns Vec<PointConfig> of points found in the test_data
    fn points(&self) -> Vec<PointConfig> {
        self.test_data
            .iter()
            .map(|(name, value)| {
                (name.clone(), value.clone())
            })
            .collect::<HashMap<String, Value>>()
            .iter()
            .map(|(name, value)| {
                let type_ = match value {
                    Value::Bool(_) => "Bool",
                    Value::Int(_) => "Int",
                    Value::Real(_) => "Real",
                    Value::Double(_) => "Double",
                    Value::String(_) => "String",
                };
                PointConfig::from_yaml(
                    &Name::new("", ""),
                    &serde_yaml::from_str(&format!(r#"{}:
                        type: {}"#, name, type_)).unwrap()
                )
            })
            .collect()
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
        self.exit.store(true, Ordering::Relaxed);
    }
}
///
/// Global static counter of FnOut instances
static COUNT: AtomicUsize = AtomicUsize::new(0);
