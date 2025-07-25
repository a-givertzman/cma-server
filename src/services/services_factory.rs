use std::sync::Arc;
use sal_core::dbg::Dbg;
use sal_sync::{services::{conf::ConfTree, entity::Name, MultiQueue, MultiQueueConf, Service, Services}, thread_pool::Scheduler};
use crate::{conf::{api_client_conf::ApiClientConf, cache_service_conf::CacheServiceConf, profinet_client_conf::profinet_client_conf::ProfinetClientConf, slmp_client_conf::slmp_client_conf::SlmpClientConf, tcp_client_conf::TcpClientConf}, services::{api_cient::api_client::ApiClient, cache::cache_service::CacheService, history::{producer_service::ProducerService, producer_service_conf::ProducerServiceConf}, profinet_client::profinet_client::ProfinetClient, server::{TcpServer, TcpServerConf}, slmp_client::slmp_client::SlmpClient, task::{Task, TaskConf}, tcp_client::tcp_client::TcpClient, FrdmService, FrdmServiceConf}};

///
/// Creates the service's by the  name
/// - Supports all implemented servicesa in the application
/// - to add new service to the application specifi it conctructor in the service name matcher
pub struct ServicesFactory {
    parent: Name,
    dbg: Dbg,
}
//
//
impl ServicesFactory {
    pub const API_CLIENT: &'static str = "ApiClient";
    pub const MULTI_QUEUE: &'static str = "MultiQueue";
    pub const PROFINET_CLIENT: &'static str = "ProfinetClient";
    pub const TASK: &'static str = "Task";
    pub const TCP_CLIENT: &'static str = "TcpClient";
    pub const TCP_SERVER: &'static str = "TcpServer";
    pub const PRODUCER_SERVICE: &'static str = "ProducerService";
    pub const CACHE_SERVICE: &'static str = "CacheService";
    pub const SLMP_CLIENT: &'static str = "SlmpClient";
    pub const FRDM_SERVICE: &'static str = "FrdmService";
    ///
    /// Crteates [ServicesFactory] new instance
    pub fn new(parent: &Name) -> Self {
        let dbg = Dbg::new(parent, "ServicesFactory");
        Self {
            parent: parent.to_owned(),
            dbg,
        }
    }
    ///
    /// ## Returns service new instance by it's name
    /// - `kind` - The kind of the service
    /// - `name` - The name of the service
    /// - `conf` - The conf of the service
    /// - `services` - [Services] to the service if required
    /// - `scheduler` - [Scheduler] of the external `ThreadPool`
    /// 
    /// ```yaml
    /// # keywd |     kind      |     name
    /// service     ApiClient       ApiClient-1
    /// ```
    /// 
    /// ### Panics
    /// - if specified service `name` is not supported
    pub fn service(&self, kind: impl Into<String>, name: &str, conf: ConfTree, services: Arc<Services>, scheduler: Scheduler) -> Arc<dyn Service> {
        let kind = &kind.into();
        match kind.as_ref() {
            Self::API_CLIENT => Arc::new(
                ApiClient::new(ApiClientConf::new(&self.parent, conf), services, scheduler.clone())
            ),
            Self::MULTI_QUEUE => Arc::new(
                MultiQueue::new(MultiQueueConf::new(&self.parent, conf), services, Some(scheduler.clone()))
            ),
            Self::PROFINET_CLIENT => Arc::new(
                ProfinetClient::new(ProfinetClientConf::new(&self.parent, conf), services, scheduler.clone())
            ),
            Self::TASK => Arc::new(
                Task::new(TaskConf::new(&self.parent, conf), services.clone(), scheduler.clone())
            ),
            Self::TCP_CLIENT => Arc::new(
                TcpClient::new(TcpClientConf::new(&self.parent, conf), services.clone(), scheduler.clone())
            ),
            Self::TCP_SERVER => Arc::new(
                TcpServer::new(TcpServerConf::new(&self.parent, conf), services.clone(), scheduler.clone())
            ),
            Self::PRODUCER_SERVICE => Arc::new(
                ProducerService::new(ProducerServiceConf::new(&self.parent, conf), services.clone(), scheduler.clone())
            ),
            Self::CACHE_SERVICE => Arc::new(
                CacheService::new(CacheServiceConf::new(&self.parent, conf), services.clone(), scheduler.clone())
            ),
            Self::SLMP_CLIENT => Arc::new(
                SlmpClient::new(SlmpClientConf::new(&self.parent, conf), services, scheduler.clone())
            ),
            Self::FRDM_SERVICE => Arc::new(
                FrdmService::new(FrdmServiceConf::new(&self.parent, conf), services, scheduler.clone())
            ),
            _ => {
                panic!("{}.service | Unknown service: {}({})", self.dbg, kind, name);
            }
        }
    }
}
