use sal_core::dbg::Dbg;
use sal_sync::{services::{
    conf::ConfTree, entity::Name, MultiQueue, MultiQueueConf,
    Service, Services,
}, thread_pool::{Scheduler, ThreadPool}};
use std::{path::Path, process::exit, sync::Arc, thread, time::Duration};
use libc::{
    SIGABRT, SIGHUP, SIGINT, SIGKILL, SIGQUIT, SIGTERM, SIGUSR1, SIGUSR2,
    // SIGFPE, SIGILL, SIGSEGV, 
};
use signal_hook::iterator::Signals;
use crate::{
    conf::{
        api_client_conf::ApiClientConf, app::app_config::AppConfig, cache_service_config::CacheServiceConfig,
        profinet_client_config::profinet_client_config::ProfinetClientConfig,
        slmp_client_config::slmp_client_config::SlmpClientConfig, task_config::TaskConfig,
        tcp_client_config::TcpClientConfig, tcp_server_config::TcpServerConfig
    }, services::{
        api_cient::api_client::ApiClient, cache::cache_service::CacheService,
        history::{producer_service::ProducerService, producer_service_config::ProducerServiceConfig},
        profinet_client::profinet_client::ProfinetClient,
        server::tcp_server::TcpServer,
        slmp_client::slmp_client::SlmpClient, task::task::Task, tcp_client::tcp_client::TcpClient,
    }
};

pub struct App {
    dbg: Dbg,
    name: Name,
    conf: AppConfig,
}
//
// 
impl App {
    ///
    /// Creates new instance of the ReatinBuffer
    ///     - path - path to the application configuration
    pub fn new(path: Vec<impl AsRef<Path>>) -> Self {
        path.iter().for_each(|p| {
            log::info!("App.run | Configuration path: '{}'", p.as_ref().display());
        });
        let conf: AppConfig = AppConfig::read(path);
        let dbg = Dbg::new(conf.name.parent(), conf.name.me());
        Self {
            dbg,
            name: conf.name.clone(),
            conf,
        }
    }
    ///
    /// Executes all services
    pub fn run(self) -> Result<(), String>  {
        let dbg = self.dbg.clone();
        log::info!("{}.run | Starting application...", dbg);
        let conf = self.conf.clone();
        let self_name = conf.name.clone();
        let thread_pool = ThreadPool::new(&dbg, conf.tread_pool);
        let services = Arc::new(Services::new(&dbg, conf.services.clone(), Some(thread_pool.scheduler())));
        log::info!("{}.run |     Configuring services...", dbg);
        for (node_keywd, node_conf) in conf.nodes {
            let node_name = node_keywd.name();
            let node_sufix = node_keywd.sufix();
            log::info!("{}.run |         Configuring service: {}({})...", dbg, node_name, node_sufix);
            log::trace!("{}.run |         Config: {:#?}", dbg, node_conf);
            services.insert(
                Self::build_service(&dbg, &self_name, &node_name, &node_sufix, node_conf, services.clone(), thread_pool.scheduler()),
            );
            log::info!("{}.run |         Configuring service: {}({}) - ok\n", dbg, node_name, node_sufix);
        }
        log::info!("{}.run |     All services configured\n", dbg);
        thread::sleep(Duration::from_millis(100));
        services.run().unwrap();
        // let name = services.name().join();
        // app.write().unwrap().insert_handles(&name, handles);
        thread::sleep(Duration::from_millis(100));
        log::info!("{}.run |     Starting services...", dbg);
        let services_iter = services.all();
        for (name, service) in services_iter {
            log::info!("{}.run |         Starting service: {}...", dbg, name);
            match service.run() {
                Ok(_) => {
                    // app.write().unwrap().insert_handles(&name, handles);
                    log::info!("{}.run |         Starting service: {} - ok", dbg, name);
                }
                Err(err) => {
                    log::error!("{}.run |         Error starting service '{}': {:#?}", dbg, name, err);
                }
            };
            thread::sleep(Duration::from_millis(100));
        }
        log::info!("{}.run |     All services started\n", dbg);
        log::info!("{}.run | Application started\n", dbg);
        Self::listen_sys_signals(dbg.clone(), services.clone(), thread_pool.scheduler());
        for (service_name, service) in services.all() {
            log::info!("{}.run | Waiting for service '{}' being finished...", dbg, service_name);
            match service.wait() {
                Ok(_) => log::info!("{}.run | Waiting for service '{}' being finished - Ok", dbg, service_name),
                Err(err) => log::info!("{}.run | Waiting for service '{}' being finished - Error: \n\t{:?}", dbg, service_name, err),
            }
        }
        log::info!("{}.run | Application exit - Ok\n", dbg);
        Ok(())
    }    
    ///
    /// Returns service by it's name
    fn build_service(dbg: &Dbg, parent: &Name, node_name: &str, node_sufix: &str, node_conf: ConfTree, services: Arc<Services>, scheduler: Scheduler) -> Arc<dyn Service> {
        match node_name {
            Services::API_CLIENT => Arc::new(
                ApiClient::new(ApiClientConf::new(parent, node_conf), scheduler.clone())
            ),
            Services::MULTI_QUEUE => Arc::new(
                MultiQueue::new(MultiQueueConf::new(parent, node_conf), services, Some(scheduler.clone()))
            ),
            Services::PROFINET_CLIENT => Arc::new(
                ProfinetClient::new(ProfinetClientConfig::new(parent, node_conf), services, scheduler.clone())
            ),
            Services::TASK => Arc::new(
                Task::new(TaskConfig::new(parent, node_conf), services.clone(), scheduler.clone())
            ),
            Services::TCP_CLIENT => Arc::new(
                TcpClient::new(TcpClientConfig::new(parent, node_conf), services.clone(), scheduler.clone())
            ),
            Services::TCP_SERVER => Arc::new(
                TcpServer::new(TcpServerConfig::new(parent, node_conf), services.clone(), scheduler.clone())
            ),
            Services::PRODUCER_SERVICE => Arc::new(
                ProducerService::new(ProducerServiceConfig::new(parent, node_conf), services.clone(), scheduler.clone())
            ),
            Services::CACHE_SERVICE => Arc::new(
                CacheService::new(CacheServiceConfig::new(parent, node_conf), services.clone(), scheduler.clone())
            ),
            Services::SLMP_CLIENT => Arc::new(
                SlmpClient::new(SlmpClientConfig::new(parent, node_conf), services, scheduler.clone())
            ),
            _ => {
                panic!("{}.build_service | Unknown service: {}({})", dbg, node_name, node_sufix);
            }
        }
    }
    ///
    /// Listening for signals from the operating system
    fn listen_sys_signals(dbg: Dbg, services: Arc<Services>, scheduler: Scheduler) {
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
                scheduler.clone().spawn(move || {
                    let signals_handle = signals.handle();
                    let dbg_ = dbg.clone();
                    let handle = scheduler.spawn(move || {
                        let dbg = dbg_;
                        for signal in signals.forever() {
                            println!("{}.run Received signal {:?}", dbg, signal);
                            match signal {
                                SIGINT | SIGQUIT | SIGTERM => {
                                    println!("{}.run Received signal {:?}", dbg, signal);
                                    println!("{}.run Application exit...", dbg);
                                    let services_iter = services.all();
                                    for (id, service) in services_iter {
                                        println!("{}.run Stopping service '{}'...", dbg, id);
                                        service.exit();
                                        println!("{}.run Stopping service '{}' - Ok", dbg, id);
                                    }
                                    services.exit();
                                    break;
                                }
                                SIGKILL => {
                                    println!("{}.run Received signal {:?}", dbg, signal);
                                    println!("{}.run Application halt...", dbg);
                                    exit(0);
                                }
                                _ => println!("{}.run Received unknown signal {:?}", dbg, signal)
                            }
                        }
                        Ok(())
                    }).unwrap();
                    handle.join().unwrap();
                    signals_handle.close();
                    Ok(())
                }).unwrap();
            }
            Err(err) => {
                panic!("{}.run | Application hook system signals error; {:#?}", dbg, err);
            }
        }
    }
}