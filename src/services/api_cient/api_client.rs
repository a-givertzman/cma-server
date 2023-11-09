#![allow(non_snake_case)]

use std::{sync::{mpsc::{Receiver, Sender, self}, Arc, atomic::{AtomicBool, Ordering}}, time::Duration, thread, collections::VecDeque};

use log::{info, debug, trace, warn};

use crate::{core_::{point::point_type::PointType, conf::api_client_config::ApiClientConfig}, services::task::task_cycle::ServiceCycle};

///
/// - Holding single input queue
/// - Received string messages pops from the queue into the end of local buffer
/// - Sending messages (wrapped into ApiQuery) from the beginning of the buffer
/// - Sent messages immediately removed from the buffer
pub struct ApiClient {
    id: String,
    recv: Vec<Receiver<PointType>>,
    send: Sender<PointType>,
    conf: ApiClientConfig,
    cycle: Option<Duration>,
    exit: Arc<AtomicBool>,
}
///
/// 
impl ApiClient {
    ///
    /// 
    pub fn new(id: String, conf: ApiClientConfig) -> Self {
        let (send, recv) = mpsc::channel();
        Self {
            id,
            recv: vec![recv],
            send: send,
            conf: conf.clone(),
            cycle: conf.cycle.clone(),
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// 
    pub fn getLink(&self, _name: &str) -> Sender<PointType> {
        self.send.clone()
    }
    ///
    /// 
    fn readQueue(selfName: &str, recv: &Receiver<PointType>, buffer: &mut VecDeque<PointType>) {
        for _ in 0..1000 {                    
            match recv.recv() {
                Ok(point) => {
                    debug!("ApiClient({}).run | point: {:?}", selfName, &point);
                    buffer.push_front(point);
                },
                Err(_err) => {
                    break;
                    // warn!("ApiClient({}).run | Error receiving from queue: {:?}", selfName, err);
                },
            };
        }
    }
    ///
    /// 
    pub fn run(&mut self) {
        info!("ApiClient({}).run | starting...", self.id);
        let selfName = self.id.clone();
        let exit = self.exit.clone();
        let cycleInterval = self.cycle;
        let (cyclic, cycleInterval) = match cycleInterval {
            Some(interval) => (interval > Duration::ZERO, interval),
            None => (false, Duration::ZERO),
        };
        let conf = self.conf.clone();
        let recv = self.recv.pop().unwrap();
        let _h = thread::Builder::new().name("name".to_owned()).spawn(move || {
            let mut buffer = VecDeque::new();
            let mut cycle = ServiceCycle::new(cycleInterval);
            'main: loop {
                cycle.start();
                trace!("ApiClient({}).run | step...", selfName);
                Self::readQueue(&selfName, &recv, &mut buffer);
                if exit.load(Ordering::SeqCst) {
                    break 'main;
                }
                trace!("ApiClient({}).run | step - done ({:?})", selfName, cycle.elapsed());
                if cyclic {
                    cycle.wait();
                }
            };
            info!("ApiClient({}).run | stopped", selfName);
        }).unwrap();
        info!("ApiClient({}).run | started", self.id);
        // h.join().unwrap();
    }
    ///
    /// 
    pub fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }

}