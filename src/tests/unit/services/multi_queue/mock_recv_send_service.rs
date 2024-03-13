#![allow(non_snake_case)]

use log::{info, warn, debug, trace};
use std::{collections::HashMap, sync::{mpsc::{Sender, self, Receiver}, Arc, Mutex, atomic::{AtomicBool, Ordering}}, thread::{self, JoinHandle}};
use testing::entities::test_value::Value;
use crate::{
    core_::{constants::constants::RECV_TIMEOUT, object::object::Object, point::{point_tx_id::PointTxId, point_type::{PointType, ToPoint}}}, 
    services::{service::service::Service, services::Services},
};


pub struct MockRecvSendService {
    id: String,
    rxSend: HashMap<String, Sender<PointType>>,
    rxRecv: Vec<Receiver<PointType>>,
    txQueue: String,
    services: Arc<Mutex<Services>>,
    test_data: Vec<Value>,
    sent: Arc<Mutex<Vec<PointType>>>,
    received: Arc<Mutex<Vec<PointType>>>,
    recvLimit: Option<usize>,
    exit: Arc<AtomicBool>,
}
///
/// 
impl MockRecvSendService {
    pub fn new(parent: impl Into<String>, rxQueue: &str, txQueue: &str, services: Arc<Mutex<Services>>, test_data: Vec<Value>, recvLimit: Option<usize>) -> Self {
        let self_id = format!("{}/MockRecvSendService", parent.into());
        let (send, recv) = mpsc::channel::<PointType>();
        Self {
            id: self_id.clone(),
            rxSend: HashMap::from([(rxQueue.to_string(), send)]),
            rxRecv: vec![recv],
            txQueue: txQueue.to_string(),
            services,
            test_data,
            sent: Arc::new(Mutex::new(vec![])),
            received: Arc::new(Mutex::new(vec![])),
            recvLimit,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// 
    pub fn id(&self) -> String {
        self.id.clone()
    }
    ///
    /// 
    pub fn sent(&self) -> Arc<Mutex<Vec<PointType>>> {
        self.sent.clone()
    }
    ///
    /// 
    pub fn received(&self) -> Arc<Mutex<Vec<PointType>>> {
        self.received.clone()
    }
}
///
/// 
impl Object for MockRecvSendService {
    fn id(&self) -> &str {
        &self.id
    }
}
///
/// 
impl Service for MockRecvSendService {
    //
    //
    fn get_link(&mut self, name: &str) -> std::sync::mpsc::Sender<crate::core_::point::point_type::PointType> {
        match self.rxSend.get(name) {
            Some(send) => send.clone(),
            None => panic!("{}.run | link '{:?}' - not found", self.id, name),
        }
    }
    //
    //
    fn run(&mut self) -> Result<JoinHandle<()>, std::io::Error> {
        info!("{}.run | starting...", self.id);
        let self_id = self.id.clone();
        let exit = self.exit.clone();
        let rxRecv = self.rxRecv.pop().unwrap();
        let received = self.received.clone();
        let recvLimit = self.recvLimit.clone();
        let handle = thread::Builder::new().name(format!("{}.run | Recv", self_id)).spawn(move || {
            info!("{}.run | Preparing thread Recv - ok", self_id);
            match recvLimit {
                Some(recvLimit) => {
                    let mut receivedCount = 0;
                    loop {
                        match rxRecv.recv_timeout(RECV_TIMEOUT) {
                            Ok(point) => {
                                trace!("{}.run | received: {:?}", self_id, point);
                                received.lock().unwrap().push(point);
                                receivedCount += 1;
                            },
                            Err(_) => {},
                        };
                        if receivedCount >= recvLimit {
                            break;
                        }
                        if exit.load(Ordering::SeqCst) {
                            break;
                        }
                    }
                },
                None => {
                    loop {
                        match rxRecv.recv_timeout(RECV_TIMEOUT) {
                            Ok(point) => {
                                trace!("{}.run | received: {:?}", self_id, point);
                                received.lock().unwrap().push(point);
                            },
                            Err(_) => {},
                        };
                        if exit.load(Ordering::SeqCst) {
                            break;
                        }
                    }
                },
            }
        });        
        let self_id = self.id.clone();
        let exit = self.exit.clone();
        debug!("{}.run | Lock services...", self_id);
        let txSend = self.services.lock().unwrap().get_link(&self.txQueue);
        debug!("{}.run | Lock services - ok", self_id);
        let test_data = self.test_data.clone();
        let sent = self.sent.clone();
        let _handle = thread::Builder::new().name(format!("{}.run | Send", self_id)).spawn(move || {
            info!("{}.run | Preparing thread Send - ok", self_id);
            let txId = PointTxId::fromStr(&self_id);
            for value in test_data.iter() {
                let point = value.to_point(txId,&format!("{}/test", self_id));
                match txSend.send(point.clone()) {
                    Ok(_) => {
                        trace!("{}.run | send: {:?}", self_id, point);
                        sent.lock().unwrap().push(point);
                    },
                    Err(err) => {
                        warn!("{}.run | send error: {:?}", self_id, err);
                    },
                }
                if exit.load(Ordering::SeqCst) {
                    break;
                }
            }
        });
        info!("{}.run | starting - ok", self.id);
        handle
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }
}