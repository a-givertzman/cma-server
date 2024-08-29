#[cfg(test)]
mod api_client {
    use log::{info, debug, error};
    use sal_sync::services::{entity::point::point::ToPoint, service::service::Service};
    use std::{sync::{Once, Arc, Mutex}, thread, time::{Duration, Instant}, net::TcpListener, io::{Read, Write}};
    use testing::{entities::test_value::Value, session::test_session::TestSession, stuff::{max_test_duration::TestDuration, random_test_values::RandomTestValues}};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use api_tools::api::reply::api_reply::ApiReply;
    use crate::{
        conf::api_client_config::ApiClientConfig,
        services::api_cient::api_client::ApiClient,
    };
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
    fn basic() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test ApiClient";
        println!("\n{}", self_id);
        let path = "./src/tests/unit/services/api_client/api_client.yaml";
        let test_duration = TestDuration::new(self_id, Duration::from_secs(20));
        test_duration.run().unwrap();
        let mut conf = ApiClientConfig::read(self_id, path);
        // let addr = conf.address.clone();
        let addr = "127.0.0.1:".to_owned() + &TestSession::free_tcp_port_str();
        conf.address = addr.parse().unwrap();
        let mut api_client = ApiClient::new(conf);
        // let test_duration = Duration::from_secs(10);
        let count = 10;
        let mut state = 0;
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
            count,
        );
        let test_data: Vec<Value> = test_data.collect();
        let mut sent = vec![];
        let received = Arc::new(Mutex::new(vec![]));
        let received_ref = received.clone();
        let mut buf = [0; 1024 * 4];
        let receiver_handle = thread::spawn(move || {
            let mut received = received_ref.lock().unwrap();
            info!("TCP server | Preparing test server...");
            match TcpListener::bind(addr) {
                Ok(listener) => {
                    info!("TCP server | Preparing test server - ok");
                    let mut accept_count = 2;
                    let mut max_read_errors = 3;
                    while accept_count > 0 {
                        accept_count -= 1;
                        match listener.accept() {
                            Ok((mut _socket, addr)) => {
                                info!("TCP server | accept connection - ok\n\t{:?}", addr);
                                _socket.set_read_timeout(Some(Duration::from_millis(100))).unwrap();
                                while received.len() < count {
                                    for e in buf.iter_mut() {*e = 0;}
                                    match _socket.read(&mut buf) {
                                        Ok(bytes) => {
                                            debug!("TCP server | received bytes: {:?}", bytes);
                                            let raw = String::from_utf8(buf.to_vec()).unwrap();
                                            let raw = raw.trim_matches(char::from(0));
                                            debug!("TCP server | received raw: {:?}", raw);
                                            match serde_json::from_str(&raw) {
                                                Ok(value) => {
                                                    let value: serde_json::Value = value;
                                                    debug!("TCP server | received: {:?}", value);
                                                    received.push(value.clone());
                                                    let obj = value.as_object().unwrap();
                                                    let reply = ApiReply::new(
                                                        obj.get("authToken").unwrap().as_str().unwrap().to_string(),
                                                        obj.get("id").unwrap().as_str().unwrap().to_string(),
                                                        obj.get("keepAlive").unwrap().as_bool().unwrap(),
                                                        "".into(),
                                                        vec![],
                                                    );
                                                    match _socket.write(&reply.as_bytes()) {
                                                        Ok(bytes) => {
                                                            debug!("TCP server | sent bytes: {:?}", bytes);
                                                        }
                                                        Err(err) => {
                                                            debug!("TCP server | socket write - error: {:?}", err);
                                                        }
                                                    };
                                                    // debug!("TCP server | received / count: {:?}", received.len() / count);
                                                    if (state == 0) && received.len() as f64 / count as f64 > 0.333 {
                                                        state = 1;
                                                        let duration = Duration::from_millis(500);
                                                        debug!("TCP server | beaking socket connection for {:?}", duration);
                                                        _socket.flush().unwrap();
                                                        _socket.shutdown(std::net::Shutdown::Both).unwrap();
                                                        thread::sleep(duration);
                                                        debug!("TCP server | beaking socket connection for {:?} - elapsed, restoring...", duration);
                                                        break;
                                                    }
                                                }
                                                Err(err) => {
                                                    debug!("TCP server | parse read data error: {:?}", err);
                                                }
                                            };
                                        }
                                        Err(err) => {
                                            debug!("socket read - error: {:?}", err);
                                            max_read_errors -= 1;
                                            if max_read_errors <= 0 {
                                                error!("TCP server | socket read error: {:?}", err);
                                                break;
                                            }
                                        }
                                    };
                                    thread::sleep(Duration::from_micros(100));
                                }
                            }
                            Err(err) => {
                                info!("incoming connection - error: {:?}", err);
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
        });
        api_client.run().unwrap();
        let send = api_client.get_link("api-link");
        let timer = Instant::now();
        for value in test_data {
            let point = format!("select from table where id = {}", value.to_string()).to_point(0, "teset");
            send.send(point.clone()).unwrap();
            sent.push(point.as_string().value);
            println!("sent: {:?}", point);
        }
        receiver_handle.join().unwrap();
        println!("elapsed: {:?}", timer.elapsed());
        println!("total test events: {:?}", count);
        println!("sent events: {:?}", sent.len());
        let mut received = received.lock().unwrap();
        println!("recv events: {:?}", received.len());
        assert!(sent.len() == count, "sent: {:?}\ntarget: {:?}", sent.len(), count);
        assert!(received.len() == count, "received: {:?}\ntarget: {:?}", received.len(), count);
        while &sent.len() > &0 {
            let target = sent.pop().unwrap();
            let result = received.pop().unwrap();
            let result = result.as_object().unwrap().get("sql").unwrap().as_object().unwrap().get("sql").unwrap().as_str().unwrap();
            debug!("\nresult({}): {:?}\ntarget({}): {:?}", received.len(), result, sent.len(), target);
            assert!(result == &target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        test_duration.exit();
    }
}