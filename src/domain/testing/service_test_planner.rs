use std::{str::FromStr, sync::Arc, time::Duration};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{conf::{ConfKeywd, ConfKind, ConfTree, ConfTreeGet, ServicesConf}, entity::{Name, Object, Point}, Service, Services}, sync::Owner, thread_pool::{Scheduler, ThreadPool}};
use testing::entities::test_value::Value;
use crate::{domain::{testing::{RecvService, RecvServiceConf, SendService, SendServiceConf}, RwLock}, services::ServicesFactory};

///
/// Makes easier to orgenise test of Srvice
/// - Start specified services in order
/// - Provides to inspect of each send test event
/// - Provides to inspect of each received test event
/// - Provides to inspect of all received test events
/// - Stops and await specified services in the reverse order
#[allow(unused)]
pub struct ServiceTestPlanner {
    name: Name,
    conf: ConfTree,
    tp: ThreadPool,
    services: Arc<Services>,
    services_order: Arc<RwLock<Vec<String>>>,
    event_builder: Arc<Box<dyn Fn(usize, usize, &str, &Value) -> Point + Send + Sync>>,
    events: Owner<Vec<Vec<(String, Value)>>>,
    inspect_each_sent: Vec<Arc<Box<dyn Fn(&Point) + Send + Sync>>>,
    inspect_each_received: Vec<Arc<Box<dyn Fn(&Vec<Point>) + Send + Sync>>>,
    inspect_all_received: Arc<Box<dyn Fn(Vec<Vec<Point>>) + Send + Sync>>,
    dbg: Dbg,
}
//
//
impl ServiceTestPlanner {
    ///
    /// Crteates [ServiceTestPlanner] new instance
    /// - `conf` - Yaml configuration containing all staff
    /// - `event_builder` - `Fn(txid: usize, ix: usize, val: &Value) -> Point`, where
    ///     - `txid` - Current [SendService] `Point`'s txid
    ///     - `ix` - Index of the `Value` in the `events`
    ///     - `name` - point name in the `events`
    ///     - `val` - Current value from `events`, to be sent as returned `Point`
    /// - `events` - Vector of pairs: (event-name, event-value)
    /// - `each_sent` - Inspect each sent event
    /// - `each_received` - Inspect each receiver finished, planner finished successfully only if each returns `Ok`
    /// - `all_received` - Inspect all receivers finished, planner finished successfully only if returns `Ok`
    #[allow(unused)]
    pub fn new(
        parent: impl Into<String>,
        mut conf: ConfTree,
        event_builder: impl Fn(usize, usize, &str, &Value) -> Point + Send + Sync + 'static,
        events: Vec<Vec<(impl Into<String>, Value)>>,
        each_sent: Vec<impl Fn(&Point) + Send + Sync + 'static>,
        each_received: Vec<impl Fn(&Vec<Point>) + Send + Sync + 'static>,
        all_received: impl Fn(Vec<Vec<Point>>) + Send + Sync + 'static,
    ) -> Self {
        let parent = parent.into();
        let name = Name::new(&parent, "ServiceTestPlanner");
        let dbg = Dbg::new(&parent, name.me());
        let tread_pool = conf.get("thread-pool").map(|v: u64| v as usize);
        let tp = ThreadPool::new(&parent, tread_pool);
        let services = conf.get("services").expect(&format!("{dbg}.run | `services` not foind in the config or has wrong value"));
        let services = ServicesConf::new(&parent, services);
        let services = Arc::new(Services::new(&parent, services, Some(tp.scheduler())));
        let events: Vec<Vec<(String, Value)>> = events
            .into_iter()
            .map(|events| {
                events.into_iter().rev().map(|(name, value)| (name.into(), value)).collect()
            }).collect();
        Self {
            name,
            conf,
            tp,
            services,
            services_order: Arc::new(RwLock::new(vec![])),
            event_builder: Arc::new(Box::new(event_builder)),
            events: Owner::new(events),
            inspect_each_sent: each_sent.into_iter().map(|f| {
                let f: Arc<Box<dyn Fn(&Point) + Send + Sync>> = Arc::new(Box::new(f));
                f
            }).collect(),
            inspect_each_received: each_received.into_iter().map(|f| {
                let f: Arc<Box<dyn Fn(&Vec<Point>) + Send + Sync>> = Arc::new(Box::new(f));
                f
            }).collect(),
            inspect_all_received: Arc::new(Box::new(all_received)),
            dbg,
        }
    }
    ///
    /// Returns [Scheduler] from internal [ThreadPool]
    #[allow(unused)]
    pub fn scheduler(&self) -> Scheduler {
        self.tp.scheduler()
    }
    ///
    /// Starts all service's to perform a test
    #[allow(unused)]
    pub fn run(&self) -> Result<(), Error> {
        let dbg = self.dbg.clone();
        let error = Error::new(&dbg, "run");
        let mut send_events = self.events.take().expect(&format!("{dbg}.run | `events` cant be empty, Vec<Vec<Value>> expected"));
        log::info!("{dbg}.run | Reading configuring...");
        match self.conf.sub_nodes() {
            Some(nodes) => {
                let services_factory = ServicesFactory::new(&Name::new(self.name.parent(), ""));
                let mut send_services = vec![];
                let mut recv_services = vec![];
                for conf in nodes {
                    match ConfKeywd::from_str(&conf.key) {
                        Ok(keywd) => {
                            match keywd.kind() {
                                k if k == ConfKind::Service.to_string() => {
                                    match (conf.name(), conf.title()) {
                                        (Ok(node_name), Ok(node_title)) => {
                                            log::info!("{dbg}.run | Configuring service: {} '{}'...", node_name, node_title);
                                            match node_name.as_str() {
                                                "SendService" => {
                                                    let conf = SendServiceConf::new(self.name.parent(), conf);
                                                    log::debug!("{dbg}.run | Conf: {:#?}", conf);
                                                    let events = match send_events.pop() {
                                                        Some(mut events) => {
                                                            events.reverse();
                                                            events
                                                        }
                                                        None => return Err(
                                                            error.err(&format!("{dbg}.run | SendService [{}] out of avialeble 'events' ({}), SendService's and 'events' should have same size", send_services.len() + 1, send_events.len())),
                                                        ),
                                                    };
                                                    let event_builder = self.event_builder.clone();
                                                    let service = Arc::new(SendService::new(
                                                        self.name.parent(),
                                                        conf,
                                                        Some(move |txid, ix, point_name: &str, val: &Value| {
                                                            let point = (event_builder)(txid, ix, point_name, val);
                                                            // val.to_point(txid, point_name)
                                                            point
                                                        }),
                                                        events,
                                                        self.services.clone(),
                                                        self.tp.scheduler(),
                                                    ));
                                                    self.services_order.write().push(service.name().join());
                                                    send_services.push(service.clone());
                                                    self.services.insert(service);
                                                }
                                                "RecvService" => {
                                                    let conf = RecvServiceConf::new(self.name.parent(), conf);
                                                    log::debug!("{dbg}.run | Conf: {:#?}", conf);
                                                    let service = Arc::new(RecvService::new(
                                                        self.name.parent(),
                                                        conf,
                                                        self.tp.scheduler(),
                                                    ));
                                                    self.services_order.write().push(service.name().join());
                                                    recv_services.push(service.clone());
                                                    self.services.insert(service);
                                                }
                                                _ => {
                                                    let service = services_factory.service(
                                                        &node_name,
                                                        &node_title,
                                                        conf,
                                                        self.services.clone(),
                                                        self.tp.scheduler(),
                                                    );
                                                    log::info!("{dbg}.run | Configuring service: {} - ok\n", service.name());
                                                    self.services_order.write().push(service.name().join());
                                                    self.services.insert(service);
                                                }
                                            }
                                        }
                                        (Ok(name), Err(err)) => log::warn!("{dbg}.run | Service '{name}' config `Title` not found (expected: 'service Name Title') \n\terror: {:?}, \n\tin config: {:#?}", err, conf),
                                        (Err(err), Ok(_)) => log::warn!("{dbg}.run | Service config `Name` not found (expected: 'service Name Title') \n\terror: {:?}, \n\tin config: {:#?}", err, conf),
                                        (Err(err), Err(_)) => log::warn!("{dbg}.run | Service config `Name` not found (expected: 'service Name Title') \n\terror: {:?}, \n\tin config: {:#?}", err, conf),
                                    }
                                }
                                _ => {}
                            }
                        }
                        Err(err) => {},
                    }
                }
                assert!(recv_services.len() == self.inspect_each_received.len(), "{dbg}.run | RecvService's [{}] and each_received's [{}] - are not equals", recv_services.len(), self.inspect_each_received.len());
                log::info!("{dbg}.run | All services configured\n");
                log::info!("{dbg}.run | Starting services...");
                log::info!("{dbg}.run | Services order:");
                for k in self.services_order.read().clone() {
                    log::info!("{dbg}.run |    {k}");
                }
                self.services.run()?;
                std::thread::sleep(Duration::from_millis(50));
                let services_len = self.services.all().len();
                for (ix, key) in self.services_order.read().clone().iter().enumerate() {
                    match self.services.get(key) {
                        Some(service) => {
                            if let Err(err) = service.run() {
                                return Err(error.pass_with(format!("Eror to start service '{key}' {ix} of {services_len}"), err));
                            }
                        }
                        None => return Err(error.err(format!("Service '{key}' {ix} of {services_len} - is not found"))),
                    }
                    // std::thread::sleep(Duration::from_millis(500));
                }
                log::info!("{dbg}.run | Starting services - Ok");
                let mut all_received = vec![];
                log::info!("{dbg}.run | Waiting receivers...");
                for (rcv_ix, recv) in recv_services.iter().enumerate() {
                    if let Err(err) = recv.wait() {
                        return Err(error.pass_with(format!("Eror to wait service '{}'", recv.name()), err));
                    }
                    let received = recv.received().read().clone();
                    (self.inspect_each_received[rcv_ix])(&received);
                    all_received.push(received);
                }
                log::info!("{dbg}.run | Waiting receivers - Ok");
                (self.inspect_all_received)(all_received);
                log::info!("{dbg}.run | All done");
                Ok(())
            }
            None => Err(error.err(format!("{dbg}.run | Empty or wrong config: {:#?}", self.conf))),
        }
    }
    ///
    /// Starts service's main loop in the [ThreadPool]
    ///
    /// Waits for the Service to finish.
    /// 
    /// Returns immediately if the Service has already finished.
    /// 
    /// Panics
    /// If not implemented for associated Service
    /// If specific implementation may panics internally,
    /// like std::thread::JoinHandle - may panic on some platforms
    /// if a thread attempts to join itself or otherwise may
    /// create a deadlock with joining threads.
    #[allow(unused)]
    pub fn wait(&self) -> Result<(), Error> {
        let error = Error::new(&self.dbg, "wait");
        let mut errors = vec![];
        for key in self.services_order.read().clone() {
            match self.services.get(&key) {
                Some(service) => {
                    if let Err(err) = service.wait() {
                        errors.push(error.pass_with(format!("Eror to wait service '{key}'"), err));
                    }
                }
                None => panic!("{}", error.err(format!("Service '{key}' - is not found"))),
            }
        }
        if let Err(err) = self.tp.join() {
            errors.push(err);
        }
        errors
            .is_empty()
            .then(|| ())
            .ok_or(
                Error::new(&self.dbg, "wait").pass(errors.iter().fold(String::new(), |acc, err| format!("{}\n{}", acc, err)))
            )
    }
    ///
    /// Checks if the Service has finished running.
    /// 
    /// To finish the Service call exit
    #[allow(unused)]
    pub fn is_finished(&self) -> bool {
        let mut is_finished = false;
        for (_, service) in self.services.all() {
            is_finished = is_finished & service.is_finished();
        }
        is_finished
    }
    ///
    /// Sends "exit" signal to all service's 
    /// for complettelly stop execution
    #[allow(unused)]
    pub fn exit(&self) {
        // self.exit.store(true, Ordering::Release);
        for (_, service) in self.services.all() {
            service.exit();
        }
        self.services.exit();
    }
}