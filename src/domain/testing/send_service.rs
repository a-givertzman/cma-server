use std::{fmt::Debug, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, Arc}};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{entity::{Name, Object, Point, PointTxId, ToPoint}, Service, ServiceCycle, Services}, sync::{channel::Sender, Handles}, thread_pool::Scheduler};
use testing::entities::test_value::Value;
use crate::domain::{testing::SendServiceConf, RwLock};
///
/// Service simply sends specified `events` into the specified `send-to` link and exits
pub struct SendService {
    name: Name,
    conf: SendServiceConf,
    services: Arc<Services>,
    event_builder: Option<Arc<Box<dyn Fn(usize, usize, &str, &Value) -> Point + Send + Sync>>>,
    events: Vec<(String, Value)>,
    sent: Arc<RwLock<Vec<Point>>>,
    scheduler: Scheduler,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
//
// 
impl SendService { 
    // EventBuilder: Fn(&Value) -> Point + Send + Sync + 'static {
    ///
    /// Returns [SendService] new instance
    /// - `send_to` - Service name to send specified `events` to
    /// - `event_builder` - `Fn(txid: usize, ix: usize, val: &Value) -> Point`, where
    ///     - `txid` - Current [SendService] `Point`'s txid
    ///     - `ix` - Index of the `Value` in the `events`
    ///     - `name` - The `name` of the current `value` from `events`
    ///     - `val` - Current value from `events`, to be sent as returned `Point`
    /// - `events` - [Value]'s will be sent with associated `name`'s to the specified in the `send_to` service
    #[allow(unused)]
    pub fn new(
        parent: impl Into<String>,
        conf: SendServiceConf,
        event_builder: Option<impl Fn(usize, usize, &str, &Value) -> Point + Send + Sync + 'static>,
        events: Vec<(impl Into<String>, Value)>,
        services: Arc<Services>,
        scheduler: Scheduler,
    ) -> Self {
        let name = Name::new(parent, format!("SendService{}", COUNT.fetch_add(1, Ordering::Relaxed)));
        let dbg = Dbg::new(name.parent(), name.me());
        Self {
            name,
            conf,
            services,
            event_builder: event_builder.map(|b| {
                let b: Box<dyn Fn(usize, usize, &str, &Value) -> Point + Send + Sync> = Box::new(b);
                Arc::new(b)
            }),
            events: events.into_iter().map(|(n, v)| (n.into(), v)).collect(),
            sent: Arc::new(RwLock::new(vec![])),
            scheduler,
            handles: Handles::new(&dbg),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
    ///
    /// 
    pub fn id(&self) -> String {
        self.dbg.to_string()
    }
    ///
    /// 
    #[allow(unused)]
    pub fn sent(&self) -> Vec<Point> {
        self.sent.read().clone()
    }
    ///
    /// 
    #[allow(unused)]
    pub fn sent_len(&self) -> usize {
        self.sent.read().len()
    }
}
//
// 
impl Object for SendService {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl Debug for SendService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SendService")
            .field("id", &self.dbg)
            .finish()
    }
}
//
//
impl Service for SendService {
    //
    //
    fn get_link(&self, _name: &str) -> Sender<Point> {
        panic!("{}.get_link | Does not support get_link", self.id())
        // match self.rxSend.get(name) {
        //     Some(send) => send.clone(),
        //     None => panic!("{}.run | link '{:?}' - not found", self.id, name),
        // }
    }
    //
    //
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let txid = PointTxId::from_str(&self.name.join());
        let name = self.name.join();
        let exit = self.exit.clone();
        let event_builder = self.event_builder.clone();
        let send_to = self.services.get_link(&self.conf.send_to)
            .map_err(|err| format!("{}.run | services.get_link error: {:#?}", self.dbg, err)).unwrap();
        let events = self.events.clone();
        let sent = self.sent.clone();
        let interval = self.conf.cycle.clone();
        let cycle = interval.map(|interval| ServiceCycle::new(&name, interval));
        let handle = self.scheduler.spawn(move || {
            log::info!("{}.run | Preparing thread - ok", dbg);
            for (ix, (point_name, value)) in events.iter().enumerate() {
                let point = match &event_builder {
                    Some(builder) => (builder)(txid, ix, point_name, value),
                    None => value.to_point(txid, &format!("{name}/Test.val-{ix}")),
                };
                match send_to.send(point.clone()) {
                    Ok(_) => {
                        log::trace!("{}.run | send: {:?}", dbg, point);
                        sent.write().push(point);
                    }
                    Err(err) => {
                        log::warn!("{}.run | send error: {:?}", dbg, err);
                    }
                }
                if exit.load(Ordering::SeqCst) {
                    break;
                }
                if let Some(cycle) = &cycle {
                    cycle.wait();
                }
            }
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
}
///
/// Global static counter of FnOut instances
pub static COUNT: AtomicUsize = AtomicUsize::new(0);
