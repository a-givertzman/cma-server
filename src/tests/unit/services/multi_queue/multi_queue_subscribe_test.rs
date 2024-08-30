use std::{fmt::Debug, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, Arc, RwLock}, thread, time::Duration};
use log::{info, trace, warn};
use sal_sync::services::{entity::{name::Name, object::Object, point::point::Point}, service::{service::Service, service_handles::ServiceHandles}};
use crate::services::{safe_lock::SafeLock, services::Services};
#[cfg(test)]

mod multi_queue {
    use log::debug;
    use sal_sync::services::{retain::retain_conf::RetainConf, service::service::Service};
    use std::{sync::{Arc, Once, RwLock}, thread, time::{Duration, Instant}};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use testing::{
        entities::test_value::Value,
        stuff::{random_test_values::RandomTestValues, max_test_duration::TestDuration, wait::WaitTread},
    };
    use crate::{
        conf::multi_queue_config::MultiQueueConfig,
        services::{multi_queue::multi_queue::MultiQueue, safe_lock::SafeLock, services::Services},
        tests::unit::services::multi_queue::{mock_send_service::MockSendService, multi_queue_subscribe_test::MockReceiver},
    };
    ///
    ///
    static INIT: Once = Once::new();
    ///
    /// once called initialisation
    fn init_once() {
        INIT.call_once(|| {
            // implement your initialisation code to be called only once for current test file
        })
    }
    ///
    /// returns:
    ///  - ...
    fn init_each() -> () {}
    ///
    /// Test MultiQueue for broadcast subscription
    /// - events sent by multiple senders
    ///     - number of events = iterations
    /// - events received by multiple receivers
    ///     - each receiver must receive events: iterations * sender_count
    #[test]
    fn subscribe_broadcast() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "multi_queue_subscribe_test";
        println!("\n{}", self_id);
        let sender_count = 10;         // count of MockSendService's
        let receiver_count = 10;         // count of MockReceiver's
        let iterations = 1000;          // test data length of the single sender
        let total_test_events = sender_count * iterations;
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let conf = r#"
            service MultiQueue:
                in queue in-queue:
                    max-length: 10000
                send-to:  # direct send links - are empty, because only client subscribtions will be used
        "#.to_string();
        let conf = serde_yaml::from_str(&conf).unwrap();
        let mq_conf = MultiQueueConfig::from_yaml(self_id, &conf);
        debug!("mqConf: {:?}", mq_conf);
        let services = Arc::new(RwLock::new(Services::new(self_id, RetainConf::new(None::<&str>, None))));
        let mq_service = Arc::new(RwLock::new(MultiQueue::new(mq_conf, services.clone())));
        services.wlock(self_id).insert(mq_service.clone());
        let mut receiver_handles = vec![];
        let mut receivers = vec![];
        for _ in 0..receiver_count {
            let receiver = Arc::new(RwLock::new(MockReceiver::new(
                self_id,
                &format!("/{}/MultiQueue", self_id),
                services.clone(),
                Some(total_test_events),
            )));
            services.wlock(self_id).insert(receiver.clone());
            receivers.push(receiver);
        }
        let mq_handle = mq_service.write().unwrap().run().unwrap();
        for receiver in &receivers {
            let h = receiver.write().unwrap().run().unwrap();
            receiver_handles.push(h);
        }
        println!("All MockReceiver's threads - started");
        thread::sleep(Duration::from_millis(100));
        let mut senders = vec![];
        let mut sender_handles = vec![];
        let time = Instant::now();
        for i in 0..sender_count {
            let dynamic_test_data = RandomTestValues::new(
                self_id,
                vec![
                    Value::String(format!("dynamic01{}", i)),
                    Value::String(format!("dynamic02{}", i)),
                    Value::String(format!("dynamic03{}", i)),
                    Value::String(format!("dynamic04{}", i)),
                    Value::String(format!("dynamic05{}", i)),
                    Value::String(format!("dynamic06{}", i)),
                    Value::String(format!("dynamic07{}", i)),
                ],
                iterations,
            );
            let dynamic_test_data: Vec<Value> = dynamic_test_data.collect();
            let sender = Arc::new(RwLock::new(MockSendService::new(
                self_id,
                &format!("/{}/MultiQueue.in-queue", self_id),
                services.clone(),
                dynamic_test_data.clone(),
                None,
            )));
            services.wlock(self_id).insert(sender.clone());
            senders.push(sender.clone());
        }
        let services_handle = services.wlock(self_id).run().unwrap();
        for sender in &senders {
            let sender_handle = sender.write().unwrap().run().unwrap();
            sender_handles.push(sender_handle);
        }
        for h in sender_handles {
            h.wait().unwrap()
        }
        for h in receiver_handles {
            h.wait().unwrap();
        }
        for receiver in &receivers {
            receiver.read().unwrap().exit();
        }
        let elapsed = time.elapsed();
        println!("Total elapsed: {:?}", elapsed);
        println!("Total test events: {:?}", total_test_events);
        println!("Elapsed per event: {:?}", elapsed.div_f64(total_test_events as f64));
        let target = iterations;
        for sender in senders {
            let sent = sender.read().unwrap().sent();
            let result = sent.read().unwrap().len();
            println!("\t {} sent: {:?}", sender.read().unwrap().id(), result);
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        let target = total_test_events;
        for receiver in receivers {
            let result = receiver.read().unwrap().received.read().unwrap().len();
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        mq_service.read().unwrap().exit();
        services.rlock(self_id).exit();
        mq_handle.wait().unwrap();
        services_handle.wait().unwrap();
        test_duration.exit();
    }
}

///
/// Receiver with subscribtion to Multiqueue
struct MockReceiver {
    id: String,
    name: Name,
    subscribe: String,
    services: Arc<RwLock<Services>>,
    received: Arc<RwLock<Vec<Point>>>,
    recv_limit: Option<usize>,
    exit: Arc<AtomicBool>,
}
//
//
impl MockReceiver {
    pub fn new(parent: impl Into<String>, subscribe: &str, services: Arc<RwLock<Services>>, recv_limit: Option<usize>) -> Self {
        let name = Name::new(parent, format!("MockReceiver{}", COUNT.fetch_add(1, Ordering::Relaxed)));
        Self {
            id: name.join(),
            name,
            subscribe: subscribe.to_owned(),
            services,
            received: Arc::new(RwLock::new(vec![])),
            recv_limit,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
}
//
//
impl Object for MockReceiver {
    fn id(&self) -> &str {
        self.id.as_str()
    }
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
//
impl Debug for MockReceiver {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MockReceiver")
            .field("id", &self.id)
            .finish()
    }
}
//
//
impl Service for MockReceiver {
    //
    //
    fn run(&mut self) -> Result<ServiceHandles<()>, String> {
        let self_id = self.id.clone();
        let exit = self.exit.clone();
        let recv_limit = self.recv_limit;
        let subscribe = self.subscribe.clone();
        let received = self.received.clone();
        let services = self.services.clone();
        let handle = thread::Builder::new().name(format!("{}.run", self_id)).spawn(move || {
            let self_id = self_id.as_str();
            let points = vec![];
            let (_, recv) = services.wlock(self_id).subscribe(&subscribe, self_id, &points);
            match recv_limit {
                Some(recv_limit) => {
                    let mut received_len = 0;
                    while received_len < recv_limit {
                        match recv.recv_timeout(Duration::from_secs(3)) {
                            Ok(point) => {
                                received_len += 1;
                                trace!("{}.run | Received point: {:#?}", self_id, point);
                                received.write().unwrap().push(point);
                            }
                            Err(err) => match err {
                                std::sync::mpsc::RecvTimeoutError::Timeout      => warn!("{}.run | Receive error: {:#?}", self_id, err),
                                std::sync::mpsc::RecvTimeoutError::Disconnected => {}
                            }
                        }
                        if exit.load(Ordering::SeqCst) {
                            break;
                        }
                    }
                }
                None => {
                    loop {
                        match recv.recv_timeout(Duration::from_secs(3)) {
                            Ok(point) => {
                                received.write().unwrap().push(point)
                            }
                            Err(err) => match err {
                                std::sync::mpsc::RecvTimeoutError::Timeout      => warn!("{}.run | Receive error: {:#?}", self_id, err),
                                std::sync::mpsc::RecvTimeoutError::Disconnected => {}
                            }
                        }
                        if exit.load(Ordering::SeqCst) {
                            break;
                        }
                    }
                }
            }
        });
        match handle {
            Ok(handle) => {
                info!("{}.run | Starting - ok", self.id);
                Ok(ServiceHandles::new(vec![(self.id.to_owned(), handle)]))
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
///
/// Global static counter of FnOut instances
static COUNT: AtomicUsize = AtomicUsize::new(0);
