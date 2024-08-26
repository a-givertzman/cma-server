#[cfg(test)]
mod tcp_client {
    use log::{info, debug, warn};
    use sal_sync::services::entity::{object::Object, point::point::{Point, ToPoint}};
    use std::{io::BufReader, net::TcpListener, sync::{Arc, Mutex, Once, RwLock}, thread::{self, JoinHandle}, time::{Duration, Instant}};
    use testing::{entities::test_value::Value, session::test_session::TestSession, stuff::{max_test_duration::TestDuration, random_test_values::RandomTestValues, wait::WaitTread}};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::tcp_client_config::TcpClientConfig,
        core_::net::{connection_status::ConnectionStatus, protocols::jds::{jds_decode_message::JdsDecodeMessage, jds_deserialize::JdsDeserialize}},
        services::{safe_lock::SafeLock, services::Services, tcp_client::tcp_client::TcpClient},
        tcp::tcp_stream_write::OpResult, tests::unit::services::tcp_client::mock_multiqueue::MockMultiQueue,
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
    ///
    #[test]
    fn write() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "TcpClient-WRITE";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let port = TestSession::free_tcp_port_str();
        let conf = serde_yaml::from_str(&format!(r#"
            service TcpClient:
                cycle: 1 ms
                reconnect: 1 s  # default 3 s
                address: 127.0.0.1:{}
                in queue link:
                    max-length: 10000
                send-to: /{}/MockMultiQueue.queue
        "#, port, self_id)).unwrap();
        let conf = TcpClientConfig::from_yaml(self_id, &conf);
        let addr = format!("127.0.0.1:{}", port);
        // conf.address = addr.parse().unwrap();
        let iterations = 100;
        let test_data = RandomTestValues::new(
            self_id,
            vec![
                Value::Int(i64::MIN),
                Value::Int(i64::MAX),
                Value::Int(-7),
                Value::Int(0),
                Value::Int(12),
                Value::Real(f32::MAX),
                Value::Real(f32::MIN),
                Value::Real(f32::MIN_POSITIVE),
                Value::Real(-f32::MIN_POSITIVE),
                Value::Real(0.0),
                Value::Real(1.33),
                Value::Double(f64::MAX),
                Value::Double(f64::MIN),
                Value::Double(f64::MIN_POSITIVE),
                Value::Double(-f64::MIN_POSITIVE),
                Value::Double(0.0),
                Value::Double(1.33),
                Value::Bool(true),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(true),
                Value::String("test1".to_string()),
                Value::String("test1test1test1test1test1test1test1test1test1test1test1test1test1test1test1".to_string()),
                Value::String("test2".to_string()),
                Value::String("test2test2test2test2test2test2test2test2test2test2test2test2test2test2test2test2test2test2test2test2test2test2test2test2".to_string()),
            ],
            iterations,
        );
        let test_data: Vec<Value> = test_data.collect();

        let services = Arc::new(RwLock::new(Services::new(self_id, None)));
        let multi_queue = Arc::new(Mutex::new(MockMultiQueue::new(self_id, "", None)));
        let tcp_client = Arc::new(Mutex::new(TcpClient::new(conf, services.clone())));
        let tcp_client_service_id = tcp_client.lock().unwrap().id().to_owned();
        services.wlock(self_id).insert(tcp_client.clone());     // tcpClientServiceId,
        services.wlock(self_id).insert(multi_queue.clone());            // multiQueueServiceId,
        let services_handle = services.wlock(self_id).run().unwrap();
        let mut sent = vec![];
        let received = Arc::new(Mutex::new(vec![]));
        let handle = mock_tcp_server(addr.to_string(), iterations, received.clone());
        thread::sleep(Duration::from_micros(100));
        let tcp_client = services.rlock(self_id).get(&tcp_client_service_id).unwrap();
        debug!("Running service {}...", tcp_client_service_id);
        tcp_client.slock(self_id).run().unwrap();
        debug!("Running service {} - ok", tcp_client_service_id);
        let timer = Instant::now();
        let send = tcp_client.slock(self_id).get_link("link");
        debug!("Test - setup - ok");
        debug!("Sending points...");
        for value in test_data {
            let point = value.to_point(0, "teset");
            send.send(point.clone()).unwrap();
            sent.push(point);
        }
        services.rlock(self_id).exit();
        handle.wait().unwrap();
        services_handle.wait().unwrap();
        // let waitDuration = Duration::from_millis(10);
        // let mut waitAttempts = test_duration.as_micros() / waitDuration.as_micros();
        // while received.lock().unwrap().len() < count {
        //     debug!("waiting while all data beeng received {}/{}...", received.lock().unwrap().len(), count);
        //     thread::sleep(waitDuration);
        //     waitAttempts -= 1;
        //     assert!(waitAttempts > 0, "Transfering {}/{} points taks too mach time {:?} of {:?}", received.lock().unwrap().len(), count, timer.elapsed(), test_duration);
        // }
        println!("elapsed: {:?}", timer.elapsed());
        println!("total test events: {:?}", iterations);
        println!("sent events: {:?}", sent.len());
        let received = received.lock().unwrap();
        println!("recv events: {:?}", received.len());
        assert!(sent.len() == iterations, "sent: {:?}\ntarget: {:?}", sent.len(), iterations);
        assert!(received.len() == iterations, "received: {:?}\ntarget: {:?}", received.len(), iterations);
        test_duration.exit();
    }
    ///
    /// TcpServer setup
    fn mock_tcp_server(addr: String, count: usize, received: Arc<Mutex<Vec<Point>>>) -> JoinHandle<()> {
        let sent = 0;
        thread::spawn(move || {
            info!("TCP server | Preparing test server...");
            match TcpListener::bind(addr) {
                Ok(listener) => {
                    info!("TCP server | Preparing test server - ok");
                    let mut accept_count = 2;
                    while accept_count > 0 {
                        accept_count -= 1;
                        match listener.accept() {
                            Ok((mut _socket, addr)) => {
                                let mut tcp_stream = BufReader::new(_socket);
                                info!("TCP server | accept connection - ok\n\t{:?}", addr);
                                let mut jds = JdsDeserialize::new(
                                    "test",
                                    JdsDecodeMessage::new("test"),
                                );
                                let mut received_count = 0;
                                loop {
                                    match jds.read(&mut tcp_stream) {
                                        ConnectionStatus::Active(point) => {
                                            match point {
                                                OpResult::Ok(point) => {
                                                    received.lock().unwrap().push(point);
                                                    received_count += 1;
                                                    if received_count >= count {
                                                        accept_count = -1;
                                                        break;
                                                    }
                                                }
                                                OpResult::Err(err) => {
                                                    warn!("{:?}", err);
                                                }
                                                OpResult::Timeout() => {}
                                            }
                                        }
                                        ConnectionStatus::Closed(_err) => {
                                            warn!("TCP server | connection - closed");
                                        }
                                    }

                                }
                                info!("TCP server | all received: {:?}", sent);
                                // while received.lock().unwrap().len() < count {
                                //     thread::sleep(Duration::from_micros(100));
                                // }
                            }
                            Err(err) => {
                                warn!("incoming connection - error: {:?}", err);
                            }
                        }
                    }
                }
                Err(err) => {
                    // connectExit.send(true).unwrap();
                    // okRef.store(false, Ordering::SeqCst);
                    panic!("Preparing test TCP server - error: {:?}", err);
                }
            };
        })
    }
}
