//!
//! Construct the Service by it's name in the runtime
//! 
use std::sync::Arc;
use sal_core::dbg::Dbg;
use sal_sync::{services::{conf::ConfTree, entity::Name, MultiQueue, MultiQueueConf, Service, Services}, thread_pool::Scheduler};
use crate::{
    conf::{profinet_client_conf::profinet_client_conf::ProfinetClientConf, slmp_client_conf::slmp_client_conf::SlmpClientConf, tcp_client_conf::TcpClientConf},
    services::{
        ApiClient, ApiClientConf, CacheService, CacheServiceConf, VirtualDevice, VirtualDeviceConf, history::{producer_service::ProducerService, producer_service_conf::ProducerServiceConf}, profinet_client::profinet_client::ProfinetClient, server::{TcpServer, TcpServerConf}, slmp_client::slmp_client::SlmpClient, task::{Task, TaskConf}, tcp_client::tcp_client::TcpClient
    },
};

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
    const API_CLIENT: &'static str = "ApiClient";
    const MULTI_QUEUE: &'static str = "MultiQueue";
    const PROFINET_CLIENT: &'static str = "ProfinetClient";
    const TASK: &'static str = "Task";
    const TCP_CLIENT: &'static str = "TcpClient";
    const TCP_SERVER: &'static str = "TcpServer";
    const PRODUCER_SERVICE: &'static str = "ProducerService";
    const CACHE_SERVICE: &'static str = "CacheService";
    const SLMP_CLIENT: &'static str = "SlmpClient";
    const VIRTUAL_DEVICE: &'static str = "VirtualDevice";
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
            Self::API_CLIENT => {
                let conf = ApiClientConf::new(&self.parent, conf);
                log::trace!("{}.run | Conf: {:#?}", self.dbg, conf);
                Arc::new(ApiClient::new(conf, services, scheduler.clone()))
            }
            Self::MULTI_QUEUE => {
                let conf = MultiQueueConf::new(&self.parent, conf);
                log::trace!("{}.run | Conf: {:#?}", self.dbg, conf);
                Arc::new(MultiQueue::new(conf, services, Some(scheduler.clone())))
            }
            Self::PROFINET_CLIENT => {
                let conf = ProfinetClientConf::new(&self.parent, conf);
                log::trace!("{}.run | Conf: {:#?}", self.dbg, conf);
                Arc::new(ProfinetClient::new(conf, services, scheduler.clone()))
            }
            Self::TASK => {
                let conf = TaskConf::new(&self.parent, conf);
                log::trace!("{}.run | Conf: {:#?}", self.dbg, conf);
                Arc::new(Task::new(conf, services.clone(), scheduler.clone()))
            }
            Self::TCP_CLIENT => {
                let conf = TcpClientConf::new(&self.parent, conf);
                log::trace!("{}.run | Conf: {:#?}", self.dbg, conf);
                Arc::new(TcpClient::new(conf, services.clone(), scheduler.clone()))
            }
            Self::TCP_SERVER => {
                let conf = TcpServerConf::new(&self.parent, conf);
                log::trace!("{}.run | Conf: {:#?}", self.dbg, conf);
                Arc::new(TcpServer::new(conf, services.clone(), scheduler.clone()))
            }
            Self::PRODUCER_SERVICE => {
                let conf = ProducerServiceConf::new(&self.parent, conf);
                log::trace!("{}.run | Conf: {:#?}", self.dbg, conf);
                Arc::new(ProducerService::new(conf, services.clone(), scheduler.clone()))
            }
            Self::CACHE_SERVICE => {
                let conf = CacheServiceConf::new(&self.parent, conf);
                log::trace!("{}.run | Conf: {:#?}", self.dbg, conf);
                Arc::new(CacheService::new(conf, services.clone(), scheduler.clone()))
            }
            Self::SLMP_CLIENT => {
                let conf = SlmpClientConf::new(&self.parent, conf);
                log::trace!("{}.run | Conf: {:#?}", self.dbg, conf);
                Arc::new(SlmpClient::new(conf, services, scheduler.clone()))
            }
            Self::VIRTUAL_DEVICE => {
                let conf = VirtualDeviceConf::new(&self.parent, conf);
                log::trace!("{}.run | Conf: {:#?}", self.dbg, conf);
                Arc::new(VirtualDevice::new(conf, services, scheduler.clone()))
            }
            _ => {
                panic!("{}.service | Unknown service: {}({})", self.dbg, kind, name);
            }
        }
    }
}
