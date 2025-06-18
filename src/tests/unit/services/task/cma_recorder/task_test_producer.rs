use std::{collections::HashMap, fmt::Debug, str::FromStr, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, Arc}, thread::{self}, time::Duration};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{entity::{Name, Object, Point, PointConfig, PointTxId, ToPoint}, LinkName, Service, Services}, sync::Handles};
use testing::entities::test_value::Value;
use crate::core_::RwLock;

///
/// 
pub struct TaskTestProducer {
    dbg: Dbg,
    name: Name,
    send_to: LinkName, 
    cycle: Duration,
    services: Arc<Services>,
    test_data: Vec<(String, Value)>,
    sent: Arc<RwLock<Vec<Point>>>,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
}
//
// 
impl TaskTestProducer {
    pub fn new(parent: &str, send_to: &str, cycle: Duration, services: Arc<Services>, test_data: &[(String, Value)]) -> Self {
        let name = Name::new(parent, format!("TaskTestProducer{}", COUNT.fetch_add(1, Ordering::Relaxed)));
        let dbg = Dbg::new(name.parent(), name.me());
        Self {
            name,
            send_to: LinkName::from_str(send_to).unwrap(),
            cycle,
            services,
            test_data: test_data.to_vec(),
            sent: Arc::new(RwLock::new(vec![])),
            handles: Handles::new(&dbg),
            dbg,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// 
    #[allow(unused)]
    pub fn sent(&self) -> Vec<Point> {
        self.sent.read().clone()
    }
    ///
    /// 
    pub fn sent_len(&self) -> usize {
        self.sent.read().len()
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
        let dbg = self.dbg.clone();
        let tx_id = PointTxId::from_str(&self.name.join());
        let cycle = self.cycle;
        let delayed = !cycle.is_zero();
        let tx_send = self.services.get_link(&self.send_to).unwrap_or_else(|err| {
            panic!("{}.run | services.get_link error: {:#?}", self.dbg, err);
        });
        let sent = self.sent.clone();
        let test_data = self.test_data.clone();
        let handle = thread::Builder::new().name(dbg.to_string()).spawn(move || {
            log::debug!("{}.run | calculating step...", dbg);
            for (name, value) in test_data {
                let point = value.to_point(tx_id, &name);
                match tx_send.send(point.clone()) {
                    Ok(_) => {
                        sent.write().push(point.clone());
                        log::trace!("{}.run | sent points: {:?}", dbg, sent.read().len());
                    }
                    Err(err) => {
                        log::warn!("{}.run | Error write to queue: {:?}", dbg, err);
                    }
                }
                if delayed {
                    thread::sleep(cycle);
                }
            }
            log::info!("{}.run | All sent: {}", dbg, sent.read().len());
            // thread::sleep(Duration::from_secs_f32(0.1));
            // debug!("TaskTestProducer({}).run | calculating step - done ({:?})", name, cycle.elapsed());
        });
        match handle {
            Ok(handle) => {
                log::info!("{}.run | Started", self.dbg);
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
        self.exit.store(true, Ordering::Relaxed);
    }
}
///
/// Global static counter of FnOut instances
static COUNT: AtomicUsize = AtomicUsize::new(0);
