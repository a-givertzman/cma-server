use std::{str::FromStr, sync::{atomic::AtomicBool, Arc}};
use dashmap::DashMap;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{conf::{ConfKeywd, ConfKind, ConfTree, ConfTreeGet, ServicesConf}, entity::{Name, Object, Point}, LinkName, Service, Services}, sync::Owner, thread_pool::{Scheduler, ThreadPool}};
use testing::entities::test_value::Value;

use crate::{domain::testing::{SendService, SendServiceConf}, services::ServicesFactory};

///
/// Makes easier to orgenise test of Srvice
/// - Start specified services in order
/// - Provides to inspect of each send test event
/// - Provides to inspect of each received test event
/// - Provides to inspect of all received test events
/// - Stops and await specified services in the reverse order
/// 
pub struct ServiceTestPlanner<InspectEachSent> {
    name: Name,
    conf: ConfTree,
    produce_to: LinkName,
    inspect_each_sent: InspectEachSent,
    tp: ThreadPool,
    services: Arc<Services>,
    tasks: Arc<DashMap<String, Arc<dyn Service>>>,
    events: Owner<Vec<Vec<Value>>>,
    // exit: Arc<AtomicBool>,
    dbg: Dbg,
}
//
//
impl<InspectEachSent> ServiceTestPlanner<InspectEachSent> where 
    InspectEachSent: Fn(Point) + Send + Sync + 'static {
    ///
    /// Crteates [ServiceTestPlanner] new instance
    /// - `conf` - Yaml configuration containing all staff
    pub fn new(
        parent: impl Into<String>,
        produce_to: LinkName,
        conf: ConfTree,
        inspect_each_sent: InspectEachSent,
        services: Arc<Services>,
        mut events: Vec<Vec<Value>>,
    ) -> Self {
        let parent = parent.into();
        let name = Name::new(&parent, "ServiceTestPlanner");
        let dbg = Dbg::new(&parent, name.me());
        let tread_pool = conf.get("tread_pool").map(|v: u64| v as usize);
        let tp = ThreadPool::new(parent, tread_pool);
        events.reverse();
        Self {
            name,
            conf,
            produce_to,
            inspect_each_sent,
            tp,
            services,
            tasks: Arc::new(DashMap::new()),
            events: Owner::new(events),
            // exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
    ///
    /// Returns [Scheduler] from internal [ThreadPool]
    pub fn scheduler(&self) -> Scheduler {
        self.tp.scheduler()
    }
    ///
    /// Starts all service's to perform a test
    pub fn run(&self) -> Result<(), Error> {
        let dbg = self.dbg.clone();
        let error = Error::new(&dbg, "run");
        let services = self.conf.get("services").expect(&format!("{dbg}.run | `services` not foind in the config or has wrong value"));
        let services = ServicesConf::new(&dbg, services);
        let services = Arc::new(Services::new(&dbg, services, Some(self.tp.scheduler())));
        let mut send_events = self.events.take().expect(&format!("{dbg}.run | `events` vant be empty, Vec<Vec<Value>> expected"));
        match self.conf.sub_nodes() {
            Some(nodes) => {
                let services_factory = ServicesFactory::new(&self.name);
                let mut send_services = vec![];
                let mut services_order = vec![];
                for conf in nodes {
                    match ConfKeywd::from_str(&conf.key) {
                        Ok(keywd) => {
                            match keywd.kind() {
                                k if k == ConfKind::Service.to_string() => {
                                    match (conf.name(), conf.title()) {
                                        (Ok(node_name), Ok(node_title)) => {
                                            log::info!("{dbg}.run | Configuring service: {} '{}'...", node_name, node_title);
                                            log::trace!("{dbg}.run | Config: {:#?}", conf);
                                            match node_name.as_str() {
                                                "SendService" => {
                                                    let conf = SendServiceConf::new(&self.name, conf);
                                                    let events = match send_events.pop() {
                                                        Some(events) => events,
                                                        None => return Err(
                                                            error.err(&format!("{dbg}.run | SendService [{}] out of avialeble 'events' ({}), SendService's and 'events' should have same size", send_services.len() + 1, send_events.len())),
                                                        ),
                                                    };
                                                    let service = Arc::new(SendService::new(
                                                        &self.name,
                                                        conf,
                                                        events,
                                                        services.clone(),
                                                        self.tp.scheduler(),
                                                    ));
                                                    self.tasks.insert(service.name().join(), service.clone());
                                                    send_services.push(service.clone());
                                                    services.insert(service);
                                                }
                                                "RecvService" => {}
                                                _ => {
                                                    let service = services_factory.service(
                                                        &node_name,
                                                        &node_title,
                                                        conf,
                                                        services.clone(),
                                                        self.tp.scheduler(),
                                                    );
                                                    self.tasks.insert(service.name().join(), service.clone());
                                                    services_order.push(service.name().join());
                                                    services.insert(service);
                                                }
                                            }
                                            log::info!("{dbg}.run | Configuring service: {} '{}' - ok\n", node_name, node_title);
                                        }
                                        (Ok(name), Err(err)) => log::warn!("{dbg}.run | Service '{name}' config `Title` not found (expected: 'service Name Title') \n\terror: {:?}, \n\tin config: {:#?}", err, conf),
                                        (Err(err), Ok(_)) => log::warn!("{dbg}.run | Service config `Name` not found (expected: 'service Name Title') \n\terror: {:?}, \n\tin config: {:#?}", err, conf),
                                        (Err(err), Err(_)) => log::warn!("{dbg}.run | Service config `Name` not found (expected: 'service Name Title') \n\terror: {:?}, \n\tin config: {:#?}", err, conf),
                                    }
                                }
                                _ => {
                                    return Err(error.err(format!("{dbg}.run | Node kind '{:?}' - Is not allowed in the root of the config, 'service' expected", keywd)));
                                }
                            }
                        }
                        Err(err) => return Err(error.pass_with(format!("{dbg}.run | Unsupported keword '{}' in config: {:#?}", conf.key, conf), err)),
                    }
                }
                for key in services_order.iter() {
                    if let Some(service) = self.tasks.get(key) {
                        if let Err(err) = service.run() {
                            return Err(error.pass_with("Eror to start service", err));
                        }
                    }
                }
                Ok(())
            }
            None => Err(error.err(format!("{dbg}.run | Empty or wrong config: {:#?}", self.conf))),
        }
        // let prodocer = SendService::new(parent,
        //     send_to: self.p,
        //     services,
        //     test_data,
        //     delay,
        //     self.tp.scheduler(),
        // )
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
    fn wait(&self) -> Result<(), Error> {
        let mut errors = vec![];
        for item in self.tasks.iter() {
            let service = item.value();
            if let Err(err) = service.wait() {
                errors.push(err);
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
    fn is_finished(&self) -> bool {
        let mut is_finished = false;
        for item in self.tasks.iter() {
            let service = item.value();
            is_finished = is_finished & service.is_finished();
        }
        is_finished
    }
    ///
    /// Sends "exit" signal to all service's 
    /// for complettelly stop execution
    fn exit(&self) {
        // self.exit.store(true, Ordering::Release);
        for item in self.tasks.iter() {
            let service = item.value();
            service.exit();
        }
    }    
}