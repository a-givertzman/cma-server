#[cfg(test)]
mod socket_read_performance {
    use chrono::{DateTime, Utc};
    use log::{info, debug, error, trace};
    use std::{sync::{Once, atomic::{AtomicUsize, Ordering}, Arc}, time::{Duration, Instant}, net::{TcpStream, TcpListener}, thread, io::{Read, BufReader, Write}};
    use testing::session::test_session::TestSession;
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use crate::core_::{
        cot::cot::Cot, net::protocols::jds::jds_define::JDS_END_OF_TRANSMISSION, point::{point_hlr::PointHlr, point_type::PointType}, status::status::Status, types::bool::Bool
    };
    //
    //
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
    fn init_each() -> () {

    }
    ///
    /// Returns current timestamp
    fn ts() -> DateTime<Utc> {
        chrono::offset::Utc::now()
    }
    ///
    /// Returns timestamp in string
    fn ts_str(ts: DateTime<Utc>) -> String {
        ts.to_rfc3339()
    }
    ///
    /// Performance comparation of different methods socket reading
    /// - using stream.bytes() iterator
    /// - using stream.read(byte) method
    /// - using BufReader<stream>.bytes() iterator
    #[ignore = "Performance test"]
    #[test]
    fn read_bytes() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        println!("test read bytes from socket performance");
        let name = "/server/line1/ied1/test1";
        let ts = ts();
        // debug!("timestamp: {:?}", ts);j
        let test_data = [
            (
                format!(
                    r#"{{"id": "1", "type": "Bool",  "name": "{}", "value": false,   "status": 0, "timestamp":"{}"}}"#,
                    name, ts_str(ts),
                ),
                PointType::Bool(PointHlr::new(
                    0,
                    name,
                    Bool(false),
                    Status::Ok,
                    Cot::default(),
                    ts,
                )),
            ),
            (
                format!(
                    r#"{{"id": "1", "type": "Bool",  "name": "{}", "value": true,    "status": 0, "timestamp":"{}"}}"#,
                    name, ts_str(ts),
                ),
                PointType::Bool(PointHlr::new(
                    0,
                    name,
                    Bool(true),
                    Status::Ok,
                    Cot::default(),
                    ts,
                )),
            ),
            (
                format!(
                    r#"{{"id": "1", "type": "Int",   "name": "{}", "value": 1,   "status": 0, "timestamp":"{}"}}"#,
                    name, ts_str(ts),
                ),
                PointType::Int(PointHlr::new(
                    0,
                    name,
                    1,
                    Status::Ok,
                    Cot::default(),
                    ts,
                )),
            ),
            (
                format!(
                    r#"{{"id": "1", "type": "Int",   "name": "{}", "value": -9223372036854775808,   "status": 0, "timestamp":"{}"}}"#,
                    name, ts_str(ts),
                ),
                PointType::Int(PointHlr::new(
                    0,
                    name,
                    -9223372036854775808,
                    Status::Ok,
                    Cot::default(),
                    ts,
                )),
            ),
            (
                format!(
                    r#"{{"id": "1", "type": "Int",   "name": "{}", "value":  9223372036854775807,   "status": 0, "timestamp":"{}"}}"#,
                    name, ts_str(ts),
                ),
                PointType::Int(PointHlr::new(0, name,  9223372036854775807, Status::Ok, Cot::default(), ts))
            ),
            (
                format!(
                    r#"{{"id": "1", "type": "Real", "name": "{}", "value":  0.0, "status": 0, "timestamp":"{}"}}"#,
                    name, ts_str(ts),
                ),
                PointType::Real(PointHlr::new(0, name,  0.0, Status::Ok, Cot::default(), ts))
            ),
            (
                format!(
                    r#"{{"id": "1", "type": "Real", "name": "{}", "value": -1.1, "status": 0, "timestamp":"{}"}}"#,
                    name, ts_str(ts),
                ),
                PointType::Real(PointHlr::new(0, name, -1.1, Status::Ok, Cot::default(), ts))
            ),
            (
                format!(
                    r#"{{"id": "1", "type": "Real", "name": "{}", "value":  1.1, "status": 0, "timestamp":"{}"}}"#,
                    name, ts_str(ts),
                ),
                PointType::Real(PointHlr::new(0, name,  1.1, Status::Ok, Cot::default(), ts))
            ),
            (
                format!(
                    r#"{{"id": "1", "type": "Real", "name": "{}", "value": -3.4028235e38, "status": 0, "timestamp":"{}"}}"#,
                    name, ts_str(ts),
                ),
                PointType::Real(PointHlr::new(0, name, -f32::MAX, Status::Ok, Cot::default(), ts))
            ),
            (
                format!(
                    r#"{{"id": "1", "type": "Real", "name": "{}", "value":  3.4028235e38, "status": 0, "timestamp":"{}"}}"#,
                    name, ts_str(ts),
                ),
                PointType::Real(PointHlr::new(0, name,  f32::MAX, Status::Ok, Cot::default(), ts))
            ),
            (
                format!(
                    r#"{{"id": "1", "type": "Double", "name": "{}", "value":  0.0, "status": 0, "timestamp":"{}"}}"#,
                    name, ts_str(ts),
                ),
                PointType::Double(PointHlr::new(0, name,  0.0, Status::Ok, Cot::default(), ts))
            ),
            (
                format!(
                    r#"{{"id": "1", "type": "Double", "name": "{}", "value": -1.1, "status": 0, "timestamp":"{}"}}"#,
                    name, ts_str(ts),
                ),
                PointType::Double(PointHlr::new(0, name, -1.1, Status::Ok, Cot::default(), ts))
            ),
            (
                format!(
                    r#"{{"id": "1", "type": "Double", "name": "{}", "value":  1.1, "status": 0, "timestamp":"{}"}}"#,
                    name, ts_str(ts),
                ),
                PointType::Double(PointHlr::new(0, name,  1.1, Status::Ok, Cot::default(), ts))
            ),
            (
                format!(
                    r#"{{"id": "1", "type": "Double", "name": "{}", "value": -1.7976931348623157e308, "status": 0, "timestamp":"{}"}}"#,
                    name, ts_str(ts),
                ),
                PointType::Double(PointHlr::new(0, name, -1.7976931348623157e308, Status::Ok, Cot::default(), ts))
            ),
            (
                format!(
                    r#"{{"id": "1", "type": "Double", "name": "{}", "value":  1.7976931348623157e308, "status": 0, "timestamp":"{}"}}"#,
                    name, ts_str(ts),
                ),
                PointType::Double(PointHlr::new(0, name,  1.7976931348623157e308, Status::Ok, Cot::default(), ts))
            ),
            (
                format!(
                    r#"{{"id": "1", "type": "String","name": "{}", "value": "~!@#$%^&*()_+`1234567890-=","status": 0, "timestamp":"{}"}}"#,
                    name, ts_str(ts),
                ),
                PointType::String(PointHlr::new(0, name, "~!@#$%^&*()_+`1234567890-=".to_string(), Status::Ok, Cot::default(), ts))
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
                    Ok(mut stream) => {
                        let mut buffer = vec![];
                        let mut byte = [0u8];
                        'read: loop {
                            match stream.read(&mut byte) {
                                Ok(_) => match byte[0] {
                                    JDS_END_OF_TRANSMISSION => {
                                        received.fetch_add(1, Ordering::SeqCst);
                                        // debug!("socket read - received: {:?}", received.load(Ordering::SeqCst));
                                        if received.load(Ordering::SeqCst) >= total {
                                            break 'read;
                                        }
                                        let msg = String::from_utf8(buffer).unwrap();
                                        let recv_index = (received.load(Ordering::SeqCst) - 1) % test_data_len;
                                        trace!("socket read - received[{}]: {:?}", recv_index, msg);
                                        assert!(msg == test_data[recv_index].0);
                                        buffer = vec![];
                                    }
                                    _ => {
                                        buffer.push(byte[0]);
                                        // println!("byte[0]: {:?} => {}", byte[0], String::from_utf8(byte.to_vec()).unwrap());
                                    }
                                }
                                Err(_err) => {
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
        let addr = "127.0.0.1:".to_owned() + &TestSession::free_tcp_port_str();
        let received = Arc::new(AtomicUsize::new(0));
        let test_data_len = test_data.len();
        let total = count * test_data_len;
        mock_tcp_server(addr.to_string(), count, &test_data, received.clone());
        {
            println!("\nReading from BufReader<stream>.read(byte)...");
            let time = Instant::now();
            'main: loop {
                match TcpStream::connect(&addr) {
                    Ok(stream) => {
                        let mut buffer = vec![];
                        let mut byte = [0u8];
                        let mut buf_reader = BufReader::new(stream);
                        'read: loop {
                            match buf_reader.read(&mut byte) {
                                Ok(_) => match byte[0] {
                                    JDS_END_OF_TRANSMISSION => {
                                        received.fetch_add(1, Ordering::SeqCst);
                                        // debug!("socket read - received: {:?}", received.load(Ordering::SeqCst));
                                        if received.load(Ordering::SeqCst) >= total {
                                            break 'read;
                                        }
                                        let msg = String::from_utf8(buffer).unwrap();
                                        let recv_index = (received.load(Ordering::SeqCst) - 1) % test_data_len;
                                        trace!("socket read - received[{}]: {:?}", recv_index, msg);
                                        assert!(msg == test_data[recv_index].0);
                                        buffer = vec![];
                                    }
                                    _ => {
                                        buffer.push(byte[0]);
                                        // println!("byte[0]: {:?} => {}", byte[0], String::from_utf8(byte.to_vec()).unwrap());
                                    }
                                }
                                Err(_err) => {
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
        //
        // reading from stream.bytes
        let addr = "127.0.0.1:".to_owned() + &TestSession::free_tcp_port_str();
        let received = Arc::new(AtomicUsize::new(0));
        let test_data_len = test_data.len();
        let total = count * test_data_len;
        mock_tcp_server(addr.to_string(), count, &test_data, received.clone());
        thread::sleep(Duration::from_micros(100));
        {
            println!("\nReading from stream.bytes...");
            let time = Instant::now();
            'main: loop {
                match TcpStream::connect(&addr) {
                    Ok(stream) => {
                        let mut buffer = vec![];
                        // let mut byte = [0u8];
                        for byte in stream.bytes() {
                            match byte {
                                Ok(byte) => match byte {
                                    JDS_END_OF_TRANSMISSION => {
                                        received.fetch_add(1, Ordering::SeqCst);
                                        // debug!("socket read - received: {:?}", received.load(Ordering::SeqCst));
                                        if received.load(Ordering::SeqCst) >= total {
                                            break;
                                        }
                                        let msg = String::from_utf8(buffer).unwrap();
                                        let recv_index = (received.load(Ordering::SeqCst) - 1) % test_data_len;
                                        trace!("socket read - received[{}]: {:?}", recv_index, msg);
                                        assert!(msg == test_data[recv_index].0);
                                        buffer = vec![];
                                    }
                                    _ => {
                                        buffer.push(byte);
                                        // println!("byte[0]: {:?} => {}", byte[0], String::from_utf8(byte.to_vec()).unwrap());
                                    }
                                }
                                Err(_err) => {
                                    break;
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
        //
        // reading from BufReader<stream>
        let addr = "127.0.0.1:".to_owned() + &TestSession::free_tcp_port_str();
        let received = Arc::new(AtomicUsize::new(0));
        // let count = 10000;
        let test_data_len = test_data.len();
        let total = count * test_data_len;
        mock_tcp_server(addr.to_string(), count, &test_data, received.clone());
        thread::sleep(Duration::from_micros(100));
        {
            println!("\nreading from BufReader<stream>...");
            let time = Instant::now();
            'main: loop {
                match TcpStream::connect(&addr) {
                    Ok(stream) => {
                        let mut buffer = vec![];
                        // let mut byte = [0u8];
                        let buf_reader = BufReader::new(stream);
                        for byte in buf_reader.bytes() {
                            match byte {
                                Ok(byte) => match byte {
                                    JDS_END_OF_TRANSMISSION => {
                                        received.fetch_add(1, Ordering::SeqCst);
                                        // debug!("socket read - received: {:?}", received.load(Ordering::SeqCst));
                                        if received.load(Ordering::SeqCst) >= total {
                                            break;
                                        }
                                        let msg = String::from_utf8(buffer).unwrap();
                                        let recv_index = (received.load(Ordering::SeqCst) - 1) % test_data_len;
                                        trace!("socket read - received[{}]: {:?}", recv_index, msg);
                                        assert!(msg == test_data[recv_index].0);
                                        buffer = vec![];
                                    }
                                    _ => {
                                        buffer.push(byte);
                                        // println!("byte[0]: {:?} => {}", byte[0], String::from_utf8(byte.to_vec()).unwrap());
                                    }
                                }
                                Err(_err) => {
                                    break;
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
        // for (json, target) in test_data {
        //     let result = PointType::fromJsonBytes(json.as_bytes().to_vec()).unwrap();
        //     assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        // }
    }
    ///
    /// Behaves like a TcpServer
    fn mock_tcp_server(addr: String, count: usize, test_data: &[(String, PointType)], received: Arc<AtomicUsize>) {
        let mut sent = 0;
        let test_data = test_data.to_owned().clone();
        thread::spawn(move || {
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
                                let eot = [4];
                                for _ in 0..count {
                                    for (msg, _) in &test_data {
                                        // for e in buf.iter_mut() {*e = 0;}
                                        let bytes = msg.as_bytes();
                                        match _socket.write(&[bytes, &eot].concat()) {
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
