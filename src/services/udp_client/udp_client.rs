//!
//! communication with Vibro-analitics microcontroller (Sub MC) over udp simple protocol
//! 
//! Basic configuration parameters:
//! ```yaml
//! service UdpClient Id:
//!     parameter: value    # meaning
//!     parameter: value    # meaning
//! ```
use std::{sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}, mpsc::Sender}, thread};
use log::{info, warn};
use crate::{
    conf::{point_config::name::Name, udp_client_config::udp_client_config::UdpClientConfig}, 
    core_::{object::object::Object, point::point_type::PointType},
    services::{
        service::{service::Service, service_handles::ServiceHandles},
        services::Services,
    },
};
///
/// Do something ...
pub struct UdpClient {
    id: String,
    name: Name,
    conf: UdpClientConfig,
    services: Arc<Mutex<Services>>,
    exit: Arc<AtomicBool>,
}
//
//
impl UdpClient {
    //
    /// Crteates new instance of the UdpClient 
    pub fn new(parent: impl Into<String>, conf: UdpClientConfig, services: Arc<Mutex<Services>>) -> Self {
        Self {
            id: conf.name.join(),
            name: conf.name.clone(),
            conf: conf.clone(),
            services,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
}
//
//
impl Object for UdpClient {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> crate::conf::point_config::name::Name {
        todo!()
    }
}
//
// 
impl std::fmt::Debug for UdpClient {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UdpClient")
            .field("id", &self.id)
            .finish()
    }
}
//
// 
impl Service for UdpClient {
    //
    // 
    fn get_link(&mut self, name: &str) -> Sender<PointType> {
        panic!("{}.get_link | Does not support get_link", self.id())
        // match self.rxSend.get(name) {
        //     Some(send) => send.clone(),
        //     None => panic!("{}.run | link '{:?}' - not found", self.id, name),
        // }
    }
    //
    //
    fn run(&mut self) -> Result<ServiceHandles, String> {
        info!("{}.run | Starting...", self.id);
        let self_id = self.id.clone();
        let exit = self.exit.clone();
        info!("{}.run | Preparing thread...", self_id);
        let handle = thread::Builder::new().name(format!("{}.run", self_id.clone())).spawn(move || {
            loop {
                if exit.load(Ordering::SeqCst) {
                    break;
                }
            }
        });
        match handle {
            Ok(handle) => {
                info!("{}.run | Starting - ok", self.id);
                Ok(ServiceHandles::new(vec![(self.id.clone(), handle)]))
            }
            Err(err) => {
                let message = format!("{}.run | Start failed: {:#?}", self.id, err);
                warn!("{}", message);
                Err(message)
            }
        }
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }    
}