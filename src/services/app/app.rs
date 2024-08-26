use linked_hash_map::LinkedHashMap;
use log::{error, info, trace};
use sal_sync::services::{conf::conf_tree::ConfTree, entity::{name::Name, object::Object}, service::{service::Service, service_handles::ServiceHandles}};
use std::{path::Path, process::exit, sync::{Arc, Mutex, RwLock}, thread, time::Duration};
use libc::{
    SIGABRT, SIGHUP, SIGINT, SIGKILL, SIGQUIT, SIGTERM, SIGUSR1, SIGUSR2,
    // SIGFPE, SIGILL, SIGSEGV, 
};
use signal_hook::iterator::Signals;
use testing::stuff::wait::WaitTread;
use crate::{
    conf::{
        api_client_config::ApiClientConfig, app::app_config::AppConfig, cache_service_config::CacheServiceConfig,
        multi_queue_config::MultiQueueConfig, profinet_client_config::profinet_client_config::ProfinetClientConfig,
        slmp_client_config::slmp_client_config::SlmpClientConfig, task_config::TaskConfig,
        tcp_client_config::TcpClientConfig, tcp_server_config::TcpServerConfig
    }, services::{
        api_cient::api_client::ApiClient, cache::cache_service::CacheService,
        history::{producer_service::ProducerService, producer_service_config::ProducerServiceConfig},
        multi_queue::multi_queue::MultiQueue, profinet_client::profinet_client::ProfinetClient,
        safe_lock::SafeLock, server::tcp_server::TcpServer,
        services::Services, slmp_client::slmp_client::SlmpClient, task::task::Task, tcp_client::tcp_client::TcpClient,
    }
};

pub struct App {
    id: String,
    handles: LinkedHashMap<String, ServiceHandles<()>>,
    conf: AppConfig,
}
//
// 
impl App {
    ///
    /// Creates new instance of the ReatinBuffer
    ///     - path - path to the application configuration
    pub fn new(path: Vec<impl AsRef<Path>>) -> Self {
        let self_id = "App".to_owned();
        path.iter().for_each(|p| {
            info!("{}.run | Configuration path: '{}'", self_id, p.as_ref().display());
        });
        let conf: AppConfig = AppConfig::read(path);
        Self {
            id: self_id,
            handles: LinkedHashMap::new(),
            conf,
        }
    }
    ///
    /// Executes all services
    pub fn run(self) -> Result<(), String>  {
        let self_id = self.id.clone();
        info!("{}.run | Starting application...", self_id);
        let conf = self.conf.clone();
        let self_name = Name::new("", conf.name);
        let app = Arc::new(RwLock::new(self));
        let services = Arc::new(RwLock::new(Services::new(&self_id)));
        info!("{}.run |     Configuring services...", self_id);
        for (node_keywd, mut node_conf) in conf.nodes {
            let node_name = node_keywd.name();
            let node_sufix = node_keywd.sufix();
            info!("{}.run |         Configuring service: {}({})...", self_id, node_name, node_sufix);
            trace!("{}.run |         Config: {:#?}", self_id, node_conf);
            services.wlock(&self_id).insert(
                Self::build_service(&self_id, &self_name, &node_name, &node_sufix, &mut node_conf, services.clone()),
            );
            info!("{}.run |         Configuring service: {}({}) - ok\n", self_id, node_name, node_sufix);
        }
        info!("{}.run |     All services configured\n", self_id);
        thread::sleep(Duration::from_millis(100));
        let handles = services.wlock(&self_id).run().unwrap();
        let name = services.rlock(&self_id).id().to_owned();
        app.write().unwrap().insert_handles(&name, handles);
        thread::sleep(Duration::from_millis(100));
        info!("{}.run |     Starting services...", self_id);
        let services_iter = services.rlock(&self_id).all();
        for (name, service) in services_iter {
            info!("{}.run |         Starting service: {}...", self_id, name);
            let handles = service.slock(&self_id).run();
            match handles {
                Ok(handles) => {
                    app.write().unwrap().insert_handles(&name, handles);
                    info!("{}.run |         Starting service: {} - ok", self_id, name);
                }
                Err(err) => {
                    error!("{}.run |         Error starting service '{}': {:#?}", self_id, name, err);
                }
            };
            thread::sleep(Duration::from_millis(100));
        }
        info!("{}.run |     All services started\n", self_id);
        info!("{}.run | Application started\n", self_id);
        Self::listen_sys_signals(self_id.clone(), services.clone());
        loop {
            let servece_ids: Vec<String> = app.read().unwrap().handles.keys().cloned().collect();
            match servece_ids.first() {
                Some(service_name) => {
                    info!("{}.run | Waiting for service '{}' being finished...", self_id, service_name);
                    let handles = app.write().unwrap().handles.remove(service_name).unwrap();
                    handles.wait().unwrap();
                    info!("{}.run | Waiting for service '{}' being finished - Ok", self_id, service_name);
                }
                None => {
                    break;
                }
            }
        }
        info!("{}.run | Application exit - Ok\n", self_id);
        Ok(())
    }    
    ///
    /// Returns service by it's name
    fn build_service(self_id: &str, parent: &Name, node_name: &str, node_sufix: &str, node_conf: &mut ConfTree, services: Arc<RwLock<Services>>) -> Arc<Mutex<dyn Service + Send>> {
        match node_name {
            Services::API_CLIENT => Arc::new(Mutex::new(
                ApiClient::new(ApiClientConfig::new(parent, node_conf))
            )),
            Services::MULTI_QUEUE => Arc::new(Mutex::new(
                MultiQueue::new(MultiQueueConfig::new(parent, node_conf), services)
            )),
            Services::PROFINET_CLIENT => Arc::new(Mutex::new(
                ProfinetClient::new(ProfinetClientConfig::new(parent, node_conf), services)
            )),
            Services::TASK => Arc::new(Mutex::new(
                Task::new(TaskConfig::new(parent, node_conf), services.clone())
            )),
            Services::TCP_CLIENT => Arc::new(Mutex::new(
                TcpClient::new(TcpClientConfig::new(parent, node_conf), services.clone())
            )),
            Services::TCP_SERVER => Arc::new(Mutex::new(
                TcpServer::new(TcpServerConfig::new(parent, node_conf), services.clone())
            )),
            Services::PRODUCER_SERVICE => Arc::new(Mutex::new(
                ProducerService::new(ProducerServiceConfig::new(parent, node_conf), services.clone())
            )),
            Services::CACHE_SERVICE => Arc::new(Mutex::new(
                CacheService::new(CacheServiceConfig::new(parent, node_conf), services.clone())
            )),
            Services::SLMP_CLIENT => Arc::new(Mutex::new(
                SlmpClient::new(SlmpClientConfig::new(parent, node_conf), services)
            )),
            _ => {
                panic!("{}.run | Unknown service: {}({})", self_id, node_name, node_sufix);
            }
        }
    }
    ///
    /// Inserts new pair service_id & service_join_handle
    fn insert_handles(&mut self, id:&str, handles: ServiceHandles<()>) {
        if self.handles.contains_key(id) {
            panic!("{}.insert | Duplicated service name '{:?}'", self.id, id);
        }
        self.handles.insert(id.to_string(), handles);
    }
    ///
    /// Listening for signals from the operating system
    fn listen_sys_signals(self_id: String, services: Arc<RwLock<Services>>) {
        let signals = Signals::new([
            SIGHUP,     // code: 1	This signal is sent to a process when its controlling terminal is closed or disconnected
            SIGINT,     // code: 2	This signal is sent to a process when the user presses Control+C to interrupt its execution
            SIGQUIT,    // code: 3	This signal is similar to SIGINT but is used to initiate a core dump of the process, which is useful for debugging
            // SIGILL,     // code: 4	This signal is sent to a process when it attempts to execute an illegal instruction
            SIGABRT,    // code: 6	This signal is sent to a process when it calls the abort() function
            // SIGFPE,     // code: 8	This signal is sent to a process when it attempts to perform an arithmetic operation that is not allowed, such as division by zero
            // SIGKILL,    // code: 9	This signal is used to terminate a process immediately and cannot be caught or ignored
            // SIGSEGV,    // code: 11	This signal is sent to a process when it attempts to access memory that is not allocated to it
            SIGTERM,    // Code: 15	This signal is sent to a process to request that it terminate gracefully.
            SIGUSR1,    // code: 10	These signals can be used by a process for custom purposes
            SIGUSR2,    // code: 12	Same as SIGUSR1, code: 10
        ]);
        match signals {
            Ok(mut signals) => {
                thread::spawn(move || {
                    let signals_handle = signals.handle();
                    let handle = thread::Builder::new().name(format!("{}.run", self_id)).spawn(move || {
                        for signal in signals.forever() {
                            println!("{}.run Received signal {:?}", self_id, signal);
                            match signal {
                                SIGINT | SIGQUIT | SIGTERM => {
                                    println!("{}.run Received signal {:?}", self_id, signal);
                                    println!("{}.run Application exit...", self_id);
                                    let services_iter = services.rlock(&self_id).all();
                                    for (_id, service) in services_iter {
                                        println!("{}.run Stopping service '{}'...", self_id, _id);
                                        service.slock(&self_id).exit();
                                        println!("{}.run Stopping service '{}' - Ok", self_id, _id);
                                    }
                                    services.rlock(&self_id).exit();
                                    break;
                                }
                                SIGKILL => {
                                    println!("{}.run Received signal {:?}", self_id, signal);
                                    println!("{}.run Application halt...", self_id);
                                    exit(0);
                                }
                                _ => {
                                    println!("{}.run Received unknown signal {:?}", self_id, signal);
                                }
                            }
                        }
                    }).unwrap();
                    handle.wait().unwrap();
                    signals_handle.close();
                });
            }
            Err(err) => {
                panic!("{}.run | Application hook system signals error; {:#?}", self_id, err);
            }
        }
    }
}