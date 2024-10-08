#[cfg(test)]

mod jds_deserialize {
    use chrono::{DateTime, Utc};
    use log::{info, debug, trace, error};
    use rand::Rng;
    use sal_sync::services::{entity::{cot::Cot, point::{point::Point, point_hlr::PointHlr}, status::status::Status}, types::bool::Bool};
    use std::{sync::{Once, atomic::{AtomicUsize, Ordering}, Arc}, time::{Duration, Instant}, net::{TcpStream, TcpListener}, thread, io::{Write, BufReader}};
    use testing::session::test_session::TestSession;
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use crate::{core_::net::{connection_status::ConnectionStatus, protocols::jds::{jds_decode_message::JdsDecodeMessage, jds_deserialize::JdsDeserialize}}, tcp::tcp_stream_write::OpResult};
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
    fn ts() -> DateTime<Utc> {
        chrono::offset::Utc::now()
    }
    fn ts_str(ts: DateTime<Utc>) -> String {
        ts.to_rfc3339()
    }
    ///
    ///
    #[test]
    fn basic() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test JdsDeserialize";
        println!("{}", self_id);
        let name = "/server/line1/ied1/test1";
        let ts = ts();
        let tx_id = 0;
        // debug!("timestamp: {:?}", ts);j
        let test_data = [
            (
                format!(r#"{{"id": "1", "type": "Bool",  "name": "{}", "value": 0,   "status": 0, "timestamp":"{}"}}"#, name, ts_str(ts)),
                Point::Bool(PointHlr::new(tx_id, name, Bool(false), Status::Ok, Cot::default(), ts))
            ),
            (
                format!(r#"{{"id": "1", "type": "Bool",  "name": "{}", "value": 1,    "status": 0, "cot": "Inf", "timestamp":"{}"}}"#, name, ts_str(ts)),
                Point::Bool(PointHlr::new(tx_id, name, Bool(true), Status::Ok, Cot::default(), ts))
            ),
            (
                format!(r#"{{"id": "1", "type": "Int",   "name": "{}", "value": 1,   "status": 0, "cot": "Inf", "timestamp":"{}"}}"#, name, ts_str(ts)),
                Point::Int(PointHlr::new(tx_id, name, 1, Status::Ok, Cot::Inf, ts))
            ),
            (
                format!(r#"{{"id": "1", "type": "Int",   "name": "{}", "value": -9223372036854775808,   "status": 0, "cot": "Act", "timestamp":"{}"}}"#, name, ts_str(ts)),
                Point::Int(PointHlr::new(tx_id, name, -9223372036854775808, Status::Ok, Cot::Act, ts))
            ),
            (
                format!(r#"{{"id": "1", "type": "Int",   "name": "{}", "value":  9223372036854775807,   "status": 0, "cot": "act", "timestamp":"{}"}}"#, name, ts_str(ts)),
                Point::Int(PointHlr::new(tx_id, name,  9223372036854775807, Status::Ok, Cot::Act, ts))
            ),


            (
                format!(r#"{{"id": "1", "type": "Real", "name": "{}", "value":  0.0, "status": 0, "timestamp":"{}"}}"#, name, ts_str(ts)),
                Point::Real(PointHlr::new(tx_id, name,  0.0, Status::Ok, Cot::default(), ts))
            ),
            (
                format!(r#"{{"id": "1", "type": "Real", "name": "{}", "value": -1.1, "status": 0, "timestamp":"{}"}}"#, name, ts_str(ts)),
                Point::Real(PointHlr::new(tx_id, name, -1.1, Status::Ok, Cot::default(), ts))
            ),
            (
                format!(r#"{{"id": "1", "type": "Real", "name": "{}", "value":  1.1, "status": 0, "timestamp":"{}"}}"#, name, ts_str(ts)),
                Point::Real(PointHlr::new(tx_id, name,  1.1, Status::Ok, Cot::default(), ts))
            ),
            (
                format!(r#"{{"id": "1", "type": "Real", "name": "{}", "value": -3.4028235e38, "status": 0, "timestamp":"{}"}}"#, name, ts_str(ts)),
                Point::Real(PointHlr::new(tx_id, name, -f32::MAX, Status::Ok, Cot::default(), ts))
            ),
            (
                format!(r#"{{"id": "1", "type": "Real", "name": "{}", "value":  3.4028235e38, "status": 0, "timestamp":"{}"}}"#, name, ts_str(ts)),
                Point::Real(PointHlr::new(tx_id, name,  f32::MAX, Status::Ok, Cot::default(), ts))
            ),



            (
                format!(r#"{{"id": "1", "type": "Double", "name": "{}", "value":  0.0, "status": 0, "timestamp":"{}"}}"#, name, ts_str(ts)),
                Point::Double(PointHlr::new(tx_id, name,  0.0, Status::Ok, Cot::default(), ts))
            ),
            (
                format!(r#"{{"id": "1", "type": "Double", "name": "{}", "value": -1.1, "status": 0, "timestamp":"{}"}}"#, name, ts_str(ts)),
                Point::Double(PointHlr::new(tx_id, name, -1.1, Status::Ok, Cot::default(), ts))
            ),
            (
                format!(r#"{{"id": "1", "type": "Double", "name": "{}", "value":  1.1, "status": 0, "timestamp":"{}"}}"#, name, ts_str(ts)),
                Point::Double(PointHlr::new(tx_id, name,  1.1, Status::Ok, Cot::default(), ts))
            ),
            (
                format!(r#"{{"id": "1", "type": "Double", "name": "{}", "value": -1.7976931348623157e308, "status": 0, "timestamp":"{}"}}"#, name, ts_str(ts)),
                Point::Double(PointHlr::new(tx_id, name, -1.7976931348623157e308, Status::Ok, Cot::default(), ts))
            ),
            (
                format!(r#"{{"id": "1", "type": "Double", "name": "{}", "value":  1.7976931348623157e308, "status": 0, "timestamp":"{}"}}"#, name, ts_str(ts)),
                Point::Double(PointHlr::new(tx_id, name,  1.7976931348623157e308, Status::Ok, Cot::default(), ts))
            ),
            (
                format!(r#"{{"id": "1", "type": "String","name": "{}", "value": "~!@#$%^&*()_+`1234567890-=","status": 0, "timestamp":"{}"}}"#, name, ts_str(ts)),
                Point::String(PointHlr::new(tx_id, name, "~!@#$%^&*()_+`1234567890-=".to_string(), Status::Ok, Cot::default(), ts))
            ),
        ];
        //
        //
        let addr = "127.0.0.1:".to_owned() + &TestSession::free_tcp_port_str();
        let received = Arc::new(AtomicUsize::new(0));
        let count = 10;
        let test_data_len = test_data.len();
        let total = count * test_data_len;
        mock_tcp_server(addr.to_string(), count, &test_data, received.clone());
        thread::sleep(Duration::from_micros(100));
        {
            println!("\nReading from stream.read(byte)...");
            let time = Instant::now();
            'main: loop {
                match TcpStream::connect(&addr) {
                    Ok(tcp_stream) => {
                        let mut tcp_stream = BufReader::new(tcp_stream);
                        let mut stream = JdsDeserialize::new(
                            self_id,
                            JdsDecodeMessage::new("test")
                        );
                        'read: loop {
                            match stream.read(&mut tcp_stream) {
                                ConnectionStatus::Active(point) => {
                                    match point {
                                        OpResult::Ok(point) => {
                                            received.fetch_add(1, Ordering::SeqCst);
                                            let recv_index = (received.load(Ordering::SeqCst) - 1) % test_data_len;
                                            trace!("socket read - received[{}]: {:?}", recv_index, point);
                                            assert!(point.name() == test_data[recv_index].1.name(), "\nreceived: {:?}\nexpected: {:?}", point.name(), test_data[recv_index].1.name());
                                            assert!(point.status() == test_data[recv_index].1.status(), "\nreceived: {:?}\nexpected: {:?}", point.status(), test_data[recv_index].1.status());
                                            assert!(point.cot() == test_data[recv_index].1.cot(), "\nreceived: {:?}\nexpected: {:?}", point.cot(), test_data[recv_index].1.cot());
                                            assert!(point.timestamp() == test_data[recv_index].1.timestamp(), "\nreceived: {:?}\nexpected: {:?}", point.timestamp(), test_data[recv_index].1.timestamp());
                                            match point {
                                                Point::Bool(point) => assert!(point.value == test_data[recv_index].1.as_bool().value, "\nreceived: {:?}\nexpected: {:?}", point.value, test_data[recv_index].1.as_bool().value),
                                                Point::Int(point) => assert!(point.value == test_data[recv_index].1.as_int().value, "\nreceived: {:?}\nexpected: {:?}", point.value, test_data[recv_index].1.as_int().value),
                                                Point::Real(point) => assert!(point.value == test_data[recv_index].1.as_real().value, "\nreceived: {:?}\nexpected: {:?}", point.value, test_data[recv_index].1.as_real().value),
                                                Point::Double(point) => assert!(point.value == test_data[recv_index].1.as_double().value, "\nreceived: {:?}\nexpected: {:?}", point.value, test_data[recv_index].1.as_double().value),
                                                Point::String(point) => assert!(point.value == test_data[recv_index].1.as_string().value, "\nreceived: {:?}\nexpected: {:?}", point.value, test_data[recv_index].1.as_string().value),
                                            }
                                            // debug!("socket read - received: {:?}", received.load(Ordering::SeqCst));
                                            if received.load(Ordering::SeqCst) >= total {
                                                break 'read;
                                            }
                                        }
                                        OpResult::Err(err) => {
                                            panic!("socket read - parsing error: {:?}", err);
                                        }
                                        OpResult::Timeout() => {}
                                    }
                                }
                                ConnectionStatus::Closed(_err) => {
                                    break 'read;
                                }
                            }
                        }
                        println!("elapsed: {:?}", time.elapsed());
                        println!("received: {:?}", received.load(Ordering::SeqCst));
                        // println!("buffer: {:?}", buffer);
                        break 'main;
                    }
                    Err(_) => {}
                };
            }
        }
    }
    ///
    /// TcpServer setup
    fn mock_tcp_server(addr: String, count: usize, test_data: &[(String, Point)], received: Arc<AtomicUsize>) {
        let mut sent = 0;
        let test_data = test_data.to_owned().clone();
        thread::spawn(move || {
            info!("TCP server | Preparing test server...");
            let mut rng = rand::thread_rng();
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
                                let eot = [4];
                                for _ in 0..count {
                                    for (msg, _) in &test_data {
                                        let pos: usize = rng.gen_range(5..(msg.len() - 5));
                                        let (msg1, msg2) = msg.split_at(pos);
                                        let bytes1 = msg1.as_bytes();
                                        let bytes2 = msg2.as_bytes();
                                        match _socket.write(bytes1) {
                                            Ok(_bytes) => {
                                                sent += 1;
                                                debug!("socket sent: {:?}", msg);
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
                                        _socket.flush().unwrap();
                                        match _socket.write(&[bytes2, &eot].concat()) {
                                            Ok(_bytes) => {
                                                sent += 1;
                                                trace!("socket sent: {:?}", msg);
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
                                    }
                                }
                                info!("TCP server | all sent: {:?}", sent);
                                while received.load(Ordering::SeqCst) < count {
                                    thread::sleep(Duration::from_micros(10));
                                }
                                // while received.len() < count {}
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
    }
}

