#[cfg(test)]

mod tcp_stream {
    use log::{info, warn, debug};
    use sal_sync::services::service::service_handles::ServiceHandles;
    use std::{sync::Once, net::{TcpStream, TcpListener}, io::{Read, Write, BufReader}, thread, time::Duration};
    use testing::{session::test_session::TestSession, stuff::{wait::WaitTread, max_test_duration::TestDuration}};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::core_::{constants::constants::RECV_TIMEOUT, failure::errors_limit::ErrorLimit};
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
    /// Reading from socket (after set timeout) using tcp_stream.bytes()
    #[ignore = "Learn - all must be ignored"]
    #[test]
    fn strean_bytes() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test TcpStream read on close";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let tcp_port = TestSession::free_tcp_port_str();
        let tcp_addr = format!("127.0.0.1:{}", tcp_port);
        let handle = server(&tcp_addr, vec![0, 1, 2, 3]).unwrap();
        thread::sleep(Duration::from_millis(100));
        match TcpStream::connect(tcp_addr) {
            Ok(stream) => {
                match stream.set_read_timeout(Some(RECV_TIMEOUT)) {
                    Ok(_) => {
                        info!("{}.setStreamTimout | Socket set read timeout {:?} - ok", self_id, RECV_TIMEOUT);
                    }
                    Err(err) => {
                        warn!("{}.setStreamTimout | Socket set read timeout error {:?}", self_id, err);
                    }
                }
                let stream = BufReader::new(stream);
                for byte in stream.bytes() {
                    debug!("{}.run | received byte: {:?}", self_id, byte);
                }
            }
            Err(err) => {
                panic!("{}.run | TcpStream::connect error: {:?}", self_id, err);
            }
        }
        debug!("{}.run | TcpStream::read finished", self_id);
        handle.wait().unwrap();
        test_duration.exit();
    }
    ///
    /// Reading from socket (after set timeout) using tcp_stream.read()
    // #[ignore = "Learn - all must be ignored"]
    #[test]
    fn stream_read() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test TcpStream read on close";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let tcp_port = TestSession::free_tcp_port_str();
        let tcp_addr = format!("127.0.0.1:{}", tcp_port);
        let handle = server(&tcp_addr, vec![0, 1, 2, 3, 4]).unwrap();
        thread::sleep(Duration::from_millis(100));
        match TcpStream::connect(tcp_addr) {
            Ok(stream) => {
                match stream.set_read_timeout(Some(RECV_TIMEOUT)) {
                    Ok(_) => {
                        info!("{}.setStreamTimout | Socket set read timeout {:?} - ok", self_id, RECV_TIMEOUT);
                    }
                    Err(err) => {
                        warn!("{}.setStreamTimout | Socket set read timeout error {:?}", self_id, err);
                    }
                }
                let mut err_limit = ErrorLimit::new(3);
                let mut stream = BufReader::new(stream);
                loop {
                    let mut bytes = vec![0u8; 2];
                    match stream.read(&mut bytes) {
                        Ok(0) => {
                            debug!("{}.run | Ok(0) received", self_id);
                            if err_limit.add().is_err() {
                                debug!("{}.run | Ok(0) received - socket closed, exiting...", self_id);
                                break;
                            }
                        }
                        Ok(len) => {
                            debug!("{}.run | Bytes({}) received: {:?}", self_id, len, bytes);
                        }
                        Err(err) => {
                            debug!("{}.run | Error received: {:?}", self_id, err);
                        }
                    }
                }
            }
            Err(err) => {
                panic!("{}.run | TcpStream::connect error: {:?}", self_id, err);
            }
        }
        debug!("{}.run | TcpStream::read finished", self_id);
        handle.wait().unwrap();
        test_duration.exit();
    }
    ///
    ///
    fn server(addr: &str, mut send_bytes: Vec<u8>) -> Result<ServiceHandles<()>, String> {
        let self_id = "Emuleted TcpServer";
        let addr = addr.to_string();
        info!("{}.run | Preparing thread...", self_id);
        let handle = thread::Builder::new().name(format!("{}.run", self_id)).spawn(move || {
            info!("{}.run | Preparing thread - ok", self_id);
            match TcpListener::bind(addr.clone()) {
                Ok(listener) => {
                    info!("{}.run | Open socket {} - ok", self_id, addr);
                    for stream in listener.incoming() {
                        // if exit.load(Ordering::SeqCst) {
                        //     debug!("{}.run | Detected exit", self_id);
                        //     break;
                        // }
                        // match stream {
                        //     Ok(mut stream) => {
                        //         let mut buf = vec![];
                        //         match stream.read(&mut buf) {
                        //             Ok(len) => {
                        //                 debug!("{}.run | received {} bytes", self_id, len);
                        //             }
                        //             Err(err) => {
                        //                 warn!("{}.run | TcpListener::bind error: {:?}", self_id, err);
                        //             }
                        //         }
                        //     }
                        //     Err(err) => {
                        //         panic!("{}.run | TcpListener::incoming error: {:?}", self_id, err);
                        //     }
                        // }
                        match stream {
                            Ok(mut stream) => {
                                match stream.set_read_timeout(Some(RECV_TIMEOUT)) {
                                    Ok(_) => {
                                        info!("{}.setStreamTimout | Socket set read timeout {:?} - ok", self_id, RECV_TIMEOUT);
                                    }
                                    Err(err) => {
                                        warn!("{}.setStreamTimout | Socket set read timeout error {:?}", self_id, err);
                                    }
                                }
                                match stream.write(&mut send_bytes) {
                                    Ok(len) => {
                                        // debug!("{}.run | received {} bytes", self_id, len);
                                        info!("{}.run | sent {} bytes - ok", self_id, len);
                                        thread::sleep(Duration::from_secs(3));
                                        drop(stream);
                                        info!("{}.run | socket closed", self_id);
                                    }
                                    Err(err) => {
                                        warn!("{}.run | TcpListener::bind error: {:?}", self_id, err);
                                    }
                                }
                            }
                            Err(err) => {
                                panic!("{}.run | TcpListener::incoming error: {:?}", self_id, err);
                            }
                        }
                        break;
                    }
                }
                Err(err) => {
                    warn!("{}.run | TcpListener::bind error: {:?}", self_id, err);
                }
            };
            info!("{}.run | Exit...", self_id);
        });
        match handle {
            Ok(handle) => {
                info!("{}.run | Starting - ok", self_id);
                Ok(ServiceHandles::new(vec![(self_id.to_owned(), handle)]))
            }
            Err(err) => {
                let message = format!("{}.run | Start failed: {:#?}", self_id, err);
                warn!("{}", message);
                Err(message)
            }
        }
    }
}
