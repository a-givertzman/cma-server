#[cfg(test)]
use sal_core::{dbg::Dbg, error::ErrorLimit};
use sal_sync::services::{conf::{ConfTree, ServicesConf}, Services};
use sal_sync::{services::{entity::ToPoint, Service}, thread_pool::ThreadPool};
use std::{sync::{Once, Arc}, thread, time::{Duration, Instant}, net::TcpListener, io::{Read, Write}};
use testing::{entities::test_value::Value, stuff::{max_test_duration::TestDuration, random_test_values::RandomTestValues}};
use debugging::session::{DebugSession, LogLevel};
use api_tools::api::{message::{fields::{FieldData, FieldId, FieldKind, FieldSize, FieldSyn}, message::{MessageField, MessageParse}, message_kind::MessageKind, parse_data::ParseData, parse_id::ParseId, parse_kind::ParseKind, parse_size::ParseSize, parse_syn::ParseSyn}, reply::api_reply::ApiReply, socket::tcp_socket::TcpMessage};
use crate::{domain::Mutex, services::{ApiClient, ApiClientConf}};
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
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
    init_once();
    init_each();
    let dbg = Dbg::own("api-client-test");
    println!("\n{}", dbg);
    let path = "./src/tests/unit/services/api_client/api_client.yaml";
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(10));
    test_duration.run().unwrap();
    let mut conf = ApiClientConf::read(&dbg, path);
    // let addr = conf.address.clone();
    // let addr = "127.0.0.1:".to_owned() + &TestSession::free_tcp_port_str();
    let addr = "127.0.0.1:3131".to_owned();
    conf.address = addr.parse().unwrap();
    let tp = ThreadPool::new(&dbg, Some(4));
    let services = Arc::new(Services::new(&dbg, ServicesConf::new(
        &dbg, 
        ConfTree::empty()//new_root(serde_yaml::from_str(r#"
        // retain:
        // "#).unwrap()),
    ), Some(tp.scheduler())).unwrap());

    let api_client = ApiClient::new(conf, services, tp.scheduler());
    // let test_duration = Duration::from_secs(10);
    let count = 10;
    let mut state = 0;
    let test_data = RandomTestValues::new(
        &dbg,
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
    let dbg_clone = dbg.clone();
    let receiver_handle = tp.spawn(move || {
        let dbg = Dbg::new(dbg_clone, "MockTcpServer");
        let mut received = received_ref.lock();
        let mut message = TcpMessage::new(
            &dbg,
            vec![
                MessageField::Syn(FieldSyn::default()),
                MessageField::Id(FieldId(4)),
                MessageField::Kind(FieldKind(MessageKind::Bytes)),
                MessageField::Size(FieldSize(4)),
                MessageField::Data(FieldData(vec![]))
            ],
            ParseData::new(
                &dbg,
                ParseSize::new(
                    &dbg,
                    FieldSize(4),
                    ParseKind::new(
                        &dbg,
                        FieldKind(MessageKind::Bytes),
                        ParseId::new(
                            &dbg,
                            FieldId(4),
                            ParseSyn::new(
                                &dbg,
                                FieldSyn::default(),
                            ),
                        ),
                    ),
                ),
            ),
        );
        log::info!("{dbg} | Preparing test server...");
        match TcpListener::bind(&addr) {
            Ok(listener) => {
                log::info!("{dbg} | Preparing test server - ok");
                let mut max_read_errors = ErrorLimit::new(100);
                'main: while received.len() < count {
                    log::info!("{dbg} | accept connections on {addr}...");
                    match listener.accept() {
                        Ok((mut _socket, _)) => {
                            log::info!("{dbg} | accept connection on - ok\n\t{:?} -> {:?}", _socket.local_addr(), _socket.peer_addr());
                            _socket.set_read_timeout(Some(Duration::from_millis(100))).unwrap();
                            while received.len() < count {
                                // for e in buf.iter_mut() {*e = 0;}
                                log::debug!("{dbg} | Receiving bytes...", );
                                match _socket.read(&mut buf) {
                                    Ok(len) => {
                                        log::debug!("{dbg} | received bytes: {:?}", len);
                                        // let raw = String::from_utf8(buf[..bytes].to_vec()).unwrap();
                                        // let raw = raw.trim_matches(char::from(0));
                                        // log::debug!("{dbg} | received raw: {:?}", raw);
                                        match message.parse(buf[..len].to_owned()) {
                                            Ok((id, _, _, bytes)) => {
                                                match serde_json::from_slice(&bytes) {
                                                    Ok(value) => {
                                                        let value: serde_json::Value = value;
                                                        log::debug!("{dbg} | received value: {:?}", value);
                                                        received.push(value.clone());
                                                        log::debug!("{dbg} | received count: {} of {}", received.len(), count);
                                                        let obj = value.as_object().unwrap();
                                                        let reply = ApiReply::new(
                                                            obj.get("authToken").unwrap().as_str().unwrap().to_string(),
                                                            obj.get("id").unwrap().as_str().unwrap().to_string(),
                                                            obj.get("keepAlive").unwrap().as_bool().unwrap(),
                                                            "",
                                                            vec![],
                                                        );
                                                        let bytes = message.build(&reply.as_bytes(), id.0);
                                                        match _socket.write(&bytes) {
                                                            Ok(bytes) => {
                                                                log::debug!("{dbg} | sent bytes: {:?}", bytes);
                                                            }
                                                            Err(err) => {
                                                                log::debug!("{dbg} | socket write - error: {:?}", err);
                                                            }
                                                        };
                                                        // debug!("{dbg} | received / count: {:?}", received.len() / count);
                                                        if (state == 0) && received.len() as f64 / count as f64 > 0.333 {
                                                            state = 1;
                                                            let break_socket_duration = Duration::from_millis(100);
                                                            log::debug!("{dbg} | breaking socket connection for {:?}", break_socket_duration);
                                                            _socket.flush().unwrap();
                                                            _socket.shutdown(std::net::Shutdown::Both).unwrap();
                                                            thread::sleep(break_socket_duration);
                                                            log::debug!("{dbg} | beaking socket connection for {:?} - elapsed, restoring...", break_socket_duration);
                                                            break;
                                                        }
                                                        if (state == 1) & (received.len() >= count) {
                                                            _socket.flush().unwrap();
                                                            thread::sleep(Duration::from_millis(100));
                                                            _socket.shutdown(std::net::Shutdown::Both).unwrap();
                                                            log::debug!("{dbg} | All received, count: {} of {}", received.len(), count);
                                                            break 'main;
                                                        }
                                                    }
                                                    Err(err) => {
                                                        log::error!("{dbg} | Deserialise bytes error: {:?}", err);
                                                    }
                                                };
                                            }
                                            Err(err) => {
                                                log::error!("{dbg} | Parse message error: {:?}", err);
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        log::debug!("{dbg} | socket read - error: {:?}", err);
                                        if let Err(_) = max_read_errors.add() {
                                            log::error!("{dbg} | Max socket read errors ({}), current: {:?}", max_read_errors.limit(), err);
                                            break;
                                        }
                                    }
                                };
                                thread::sleep(Duration::from_micros(100));
                            }
                        }
                        Err(err) => {
                            log::info!("{dbg} | incoming connection - error: {:?}", err);
                        }
                    }
                }
                log::info!("{dbg} | Exit");
            }
            Err(err) => {
                // connectExit.send(true).unwrap();
                // okRef.store(false, Ordering::SeqCst);
                panic!("{dbg} | Preparing test TCP server - error: {:?}", err);
            }
        };
    }).unwrap();
    api_client.run().unwrap();
    let send = api_client.get_link("api-link");
    let timer = Instant::now();
    for value in test_data {
        let point = format!("select from table where id = {}", value.to_string()).to_point(0, "teset");
        send.send(point.clone()).unwrap();
        sent.push(point.as_string().value);
        println!("sent: {:?}", point);
        std::thread::sleep(Duration::from_millis(100));
    }
    receiver_handle.join().unwrap();
    api_client.exit();
    println!("elapsed: {:?}", timer.elapsed());
    println!("total test events: {:?}", count);
    println!("sent events: {:?}", sent.len());
    let mut received = received.lock();
    println!("recv events: {:?}", received.len());
    assert!(sent.len() == count, "sent: {:?}\ntarget: {:?}", sent.len(), count);
    assert!(received.len() == count, "received: {:?}\ntarget: {:?}", received.len(), count);
    while &sent.len() > &0 {
        let target = sent.pop().unwrap();
        let result = received.pop().unwrap();
        let result = result.as_object().unwrap().get("sql").unwrap().as_object().unwrap().get("sql").unwrap().as_str().unwrap();
        log::debug!("\nresult({}): {:?}\ntarget({}): {:?}", received.len(), result, sent.len(), target);
        assert!(result == &target, "\nresult: {:?}\ntarget: {:?}", result, target);
    }
    api_client.wait().unwrap();
    test_duration.exit();
}
