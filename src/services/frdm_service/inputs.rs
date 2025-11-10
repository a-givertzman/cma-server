use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{Service, ServiceWaiting, Services, SubscriptionCriteria, conf::ServicesConf, entity::{Cot, Name, Object, Point}, retain::RetainConf}, sync::{AtomicUsizeOption, Handles}, thread_pool::{Scheduler, ThreadPool}};
use crate::{domain::{constants::constants::RECV_TIMEOUT, unbounded, FxDashMap, Receiver, Sender}, services::frdm_service::{FrdmServiceConf, Rope}};

///
/// Stores all last incoming events, specified in the `subscriptions`
pub struct Inputs {
    name: Name,
    inputs: Arc<FxDashMap<String, Option<f64>>>,
    listeners: Arc<FxDashMap<String, Sender<Point>>>,
    conf: FrdmServiceConf,
    rope: Arc<Rope>,
    rope_pos: Arc<AtomicUsizeOption>,
    cam_segment_ix: Arc<AtomicUsizeOption>,
    services: Arc<Services>,
    scheduler: Scheduler,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
//
//
impl Inputs {
    ///
    /// Returns [Inputs] new instance
    pub fn new(
        parent: impl Into<String>,
        conf: &FrdmServiceConf,
        services: Arc<Services>,
        scheduler: Scheduler,
        exit: Arc<AtomicBool>,
    ) -> Self {
        let name = Name::new(parent, "Inputs");
        let dbg = Dbg::new(name.parent(), name.me());
        let rope = Arc::new(Rope::new(&name, conf.rope_defect.camera_offset, conf.rope_defect.segment, conf.rope_defect.segment_threshold));
        Self {
            name,
            inputs: Arc::new(FxDashMap::default()),
            listeners: Arc::new(FxDashMap::default()),
            conf: conf.clone(),
            rope,
            rope_pos: Arc::new(AtomicUsizeOption::new(None)),
            cam_segment_ix: Arc::new(AtomicUsizeOption::new(None)),
            services,
            scheduler,
            handles: Handles::new(&dbg),
            exit,
            dbg,
        }
    }
    ///
    /// Returns fake [Inputs] new instance for testing purposes
    /// - `data` - a array with pairs key - value, contains in the keys - names of required events, in values - corresponding values
    #[allow(unused)]
    pub(crate) fn fake(
        parent: impl Into<String>,
        conf: &FrdmServiceConf,
        data: impl IntoIterator<Item = (impl Into<String>, f64)>,
        exit: Arc<AtomicBool>,
    ) -> Self {
        let name = Name::new(parent, "Inputs");
        let dbg = Dbg::new(name.parent(), name.me());
        let rope = Arc::new(Rope::new(&name, conf.rope_defect.camera_offset, conf.rope_defect.segment, conf.rope_defect.segment_threshold));
        let tp = ThreadPool::new(&dbg, Some(4));
        let inputs = Arc::new(FxDashMap::default());
        for (key, val) in data {
            _ = inputs.insert(key.into(), val);
        }
        Self {
            name: name.clone(),
            inputs: Arc::new(FxDashMap::default()),
            listeners: Arc::new(FxDashMap::default()),
            conf: conf.clone(),
            rope,
            rope_pos: Arc::new(AtomicUsizeOption::new(None)),
            cam_segment_ix: Arc::new(AtomicUsizeOption::new(None)),
            services: Arc::new(Services::new(
                &dbg,
                ServicesConf { name, retain: RetainConf { path: None, point: None } },
                None,
            )),
            scheduler: tp.scheduler(),
            handles: Handles::new(&dbg),
            exit,
            dbg,
        }
    }
    ///
    /// Adds new value into the current state
    /// 
    /// Used for internal or testing purposes only
    /// 
    /// In nornal operation events should be received by the subscription
    #[allow(unused)]
    pub(crate) fn insert(&self, key: impl Into<String>, val: f64) {
        let key = key.into();
        if key == self.conf.rope_deprecation.crane.rope.pos {
            let pos = val * 1000.0;
            log::warn!("{}.insert | Rope position '{}' mm: {:?}", self.dbg, key, pos);
            self.inputs.insert(key.clone(), Some(pos));
            let pos = pos.round() as usize;
            self.rope_pos.store(Some(pos));
            self.cam_segment_ix.store(self.rope.segment_index(pos));
        } else {
            self.inputs.insert(key.clone(), Some(val));
        }
    }
    ///
    /// Returns срфттуд with all internal events
    pub fn listen(&self) -> Receiver<Point> {
        let key = format!("listener-{}", self.listeners.len());
        let (send, recv) = unbounded();
        _ = self.listeners.insert(key, send);
        recv
    }
    ///
    /// Add a subscription, as a name of the event, which later can be requested via `get()`
    pub fn subscribe(&self, name: impl Into<String>) {
        self.inputs.insert(name.into(), None);
    }
    ///
    /// Returns current value from inputs by the key if exists
    pub fn get(&self, key: &str) -> Option<f64> {
        match self.inputs.get(key) {
            Some(input) => *input.value(),
            None => {
                log::warn!("{}.get | Unexpected Event '{}' requested", self.dbg, key);
                None
            }
        }
    }
    ///
    /// Rope position, mm
    /// 
    /// Rope position is set to zero when crane in the parking position
    /// 
    /// Rope position increments as it's unwound from the winch
    pub fn rope_pos(&self) -> Option<f64> {
        match self.inputs.get(&self.conf.rope_deprecation.crane.rope.pos) {
            Some(entry) => *entry.value(),
            None => None,
        }
    }
    ///
    /// Returns cerrent rope position (mm) under the camera
    #[allow(unused)]
    pub fn pos_at_cam(&self) -> Option<f64> {
        self.rope_pos.load().map(|pos| self.rope.pos_at_cam(pos) as f64)
    }
    ///
    /// Returns Rope segment index under the camera, from 0
    /// 
    /// Returns `Some(ix)` if camera position aligned to the beginning of segment with acceptable accuracy,
    /// otherwise returns `None`
    pub fn cam_segment_ix(&self) -> Arc<AtomicUsizeOption> {
        self.cam_segment_ix.clone()
    }
}
//
//
impl Object for Inputs {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl std::fmt::Debug for Inputs {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Inputs")
            .field("dbg", &self.dbg)
            .finish()
    }
}
//
// 
impl Service for Inputs where {
    //
    // 
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let name = self.name.clone();
        let conf = self.conf.clone();
        let inputs = self.inputs.clone();
        let listeners = self.listeners.clone();
        let rope = self.rope.clone();
        let rope_pos = self.rope_pos.clone();
        let cam_segment_ix = self.cam_segment_ix.clone();
        let service_waiting = ServiceWaiting::new(&name, conf.wait_started);
        let service_release = service_waiting.release();
        let services = self.services.clone();
        let exit = self.exit.clone();
        let points: Vec<SubscriptionCriteria> = self.inputs.iter().map(|point| {
            let subscription = SubscriptionCriteria::new(point.key(), Cot::Inf);
            log::trace!("{dbg}.run | Subscription: {:?}", subscription);
            subscription
        }).collect();
        log::debug!("{}.run | Preparing thread...", dbg);
        let handle = self.scheduler.spawn(move || {
            let dbg = &dbg;
            let (_, recv) = services.subscribe(&conf.subscribe, &name.join(), &points);
            service_release.add(Ok(()));
            loop {
                log::trace!("{dbg}.run | Receiving points...");
                match recv.recv_timeout(RECV_TIMEOUT) {
                    Ok(event) => {
                        let name = event.name();
                        match inputs.get_mut(&name) {
                            Some(mut input) => {
                                log::debug!("{dbg}.run | Event '{}', value: {:?}", name, event.value());
                                if name == conf.rope_deprecation.crane.rope.pos {
                                    let pos = (event.to_int().as_int().value * 1000) as usize;
                                    rope_pos.store(Some(pos));
                                    cam_segment_ix.store(rope.segment_index(pos));
                                }
                                match &event {
                                    Point::Bool(_) => log::warn!("{dbg}.run | Event '{}' - expected numeric type, but has 'Bool'", name),
                                    Point::Int(point) => _ = input.replace(point.value as f64),
                                    Point::Real(point) => _ = input.replace(point.value as f64),
                                    Point::Double(point) => _ = input.replace(point.value),
                                    Point::String(_) => log::warn!("{dbg}.run | Event '{}' - expected numeric type, but has 'String'", name),
                                    Point::Bytes(_) => log::warn!("{dbg}.run | Event '{}' - expected numeric type, but has 'Bytes'", name),
                                }
                                for send in listeners.iter() {
                                    if let Err(err) = send.value().send(event.clone()) {
                                        log::warn!("{dbg}.run | Send error {:?}", err);
                                    }
                                }
                            }
                            None => log::warn!("{dbg}.run | Unexpected Event '{}'", name),
                        }
                        // Self::add_(dbg, &inputs, &listeners, &event);
                    }
                    Err(err) => match err {
                        crate::domain::RecvTimeoutError::Timeout => {}
                        _ => {
                            log::warn!("{dbg}.run | Receive error: {:?}", err);
                            break;
                        }
                    },
                }
                if exit.load(Ordering::Acquire) {
                    break;
                }
            }
            log::info!("{dbg}.run | Exit");
            Ok(())
        });
        match handle {
            Ok(handle) => {
                self.handles.push(handle);
            }
            Err(err) => {
                let err = Error::new(&self.dbg, "run").pass_with("Start failed", err.to_string());
                log::warn!("{}", err);
                return Err(err);
            }
        }
        let r = match conf.wait_started {
            Some(_) => {
                log::info!("{}.run | Waiting while starting...", self.dbg);
                service_waiting.wait()
            }
            None => Ok(()),
        };
        log::info!("{}.run | Starting - ok", self.dbg);
        r
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
        self.exit.store(true, Ordering::Release);
    }    
}
