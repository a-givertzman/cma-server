//!
//! # Communication with Vibro-analytics microcontroller (Sub MC) over udp simple protocol
//! 
//! ## Basic configuration parameters:
//! 
//! ```yaml
//! service UdpClient Id:
//!     parameter: value    # meaning
//!     parameter: value    # meaning
//! ```
//! 
//! ## UDP protocol description
//! 
//! Message in the UDP has fallowing fiels
//! 
//! |Field name:   | SYN | ADDR | TYPE | COUNT | DATA        |
//! |---           | --- | ---- | ---- | ----- | ----        |
//! |Data type:    | u8  | u8   | u8   | u8    | u8[1024]    | 
//! |Example value:| 22  | 0    | 16   | 1024  | [u16; 1024] |
//! - `SYN` = 22 - message starts with
//! - `ADDR` = 0...255 - an address of the input channel (0 - first input channel)
//! - `TYPE` - type of values in the array in `DATA` field
//!     - 8 - 1 byte integer value
//!     - 16 - 2 byte float value
//!     - 32 - u16[1024] an array of 2 byte values of length 512
//! - `COUNT` - length of the array in the `DATA` field
//! - `DATA` - array of values of type specified in the `TYPE` field
//! 
use std::{sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}, mpsc::Sender}, thread};
use log::{info, warn};
use sal_sync::services::{entity::{name::Name, object::Object, point::point::Point}, service::{service::Service, service_handles::ServiceHandles}};
use crate::{
    conf::udp_client_config::udp_client_config::UdpClientConfig, 
    services::services::Services,
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
    fn name(&self) -> Name {
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
    fn get_link(&mut self, name: &str) -> Sender<Point> {
        panic!("{}.get_link | Does not support get_link", self.id())
        // match self.rxSend.get(name) {
        //     Some(send) => send.clone(),
        //     None => panic!("{}.run | link '{:?}' - not found", self.id, name),
        // }
    }
    //
    //
    fn run(&mut self) -> Result<ServiceHandles<()>, String> {
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
