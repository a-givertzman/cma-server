#[cfg(test)]

use sal_core::error::{Error, ErrorLimit};
use sal_sync::sync::Handles;
use std::{sync::Once, net::{TcpStream, TcpListener}, io::{Read, Write, BufReader}, thread, time::Duration};
use testing::{session::test_session::TestSession, stuff::max_test_duration::TestDuration};
use debugging::session::{DebugSession, LogLevel};
use crate::domain::RECV_TIMEOUT;
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
    DebugSession::new().filter(LogLevel::Info).init().unwrap();
    init_once();
    init_each();
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
                    log::info!("{}.setStreamTimout | Socket set read timeout {:?} - ok", self_id, RECV_TIMEOUT);
                }
                Err(err) => {
                    log::warn!("{}.setStreamTimout | Socket set read timeout error {:?}", self_id, err);
                }
            }
            let stream = BufReader::new(stream);
            for byte in stream.bytes() {
                log::debug!("{}.run | received byte: {:?}", self_id, byte);
            }
        }
        Err(err) => {
            panic!("{}.run | TcpStream::connect error: {:?}", self_id, err);
        }
    }
    log::debug!("{}.run | TcpStream::read finished", self_id);
    handle.wait().unwrap();
    test_duration.exit();
}
///
/// Reading from socket (after set timeout) using tcp_stream.read()
// #[ignore = "Learn - all must be ignored"]
#[test]
fn stream_read() {
    DebugSession::new().filter(LogLevel::Info).init().unwrap();
    init_once();
    init_each();
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
                    log::info!("{}.setStreamTimout | Socket set read timeout {:?} - ok", self_id, RECV_TIMEOUT);
                }
                Err(err) => {
                    log::warn!("{}.setStreamTimout | Socket set read timeout error {:?}", self_id, err);
                }
            }
            let mut err_limit = ErrorLimit::new(3);
            let mut stream = BufReader::new(stream);
            loop {
                let mut bytes = vec![0u8; 2];
                match stream.read(&mut bytes) {
                    Ok(0) => {
                        log::debug!("{}.run | Ok(0) received", self_id);
                        if err_limit.add().is_err() {
                            log::debug!("{}.run | Ok(0) received - socket closed, exiting...", self_id);
                            break;
                        }
                    }
                    Ok(len) => {
                        log::debug!("{}.run | Bytes({}) received: {:?}", self_id, len, bytes);
                    }
                    Err(err) => {
                        log::debug!("{}.run | Error received: {:?}", self_id, err);
                    }
                }
            }
        }
        Err(err) => {
            panic!("{}.run | TcpStream::connect error: {:?}", self_id, err);
        }
    }
    log::debug!("{}.run | TcpStream::read finished", self_id);
    handle.wait().unwrap();
    test_duration.exit();
}
///
///
fn server(addr: &str, mut send_bytes: Vec<u8>) -> Result<Handles<()>, Error> {
    let self_id = "Emuleted TcpServer";
    let addr = addr.to_string();
    log::info!("{}.run | Preparing thread...", self_id);
    let handle = thread::Builder::new().name(format!("{}.run", self_id)).spawn(move || {
        log::info!("{}.run | Preparing thread - ok", self_id);
        match TcpListener::bind(addr.clone()) {
            Ok(listener) => {
                log::info!("{}.run | Open socket {} - ok", self_id, addr);
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
                                    log::info!("{}.setStreamTimout | Socket set read timeout {:?} - ok", self_id, RECV_TIMEOUT);
                                }
                                Err(err) => {
                                    log::warn!("{}.setStreamTimout | Socket set read timeout error {:?}", self_id, err);
                                }
                            }
                            match stream.write(&mut send_bytes) {
                                Ok(len) => {
                                    // debug!("{}.run | received {} bytes", self_id, len);
                                    log::info!("{}.run | sent {} bytes - ok", self_id, len);
                                    thread::sleep(Duration::from_millis(300));
                                    let _ = stream.shutdown(std::net::Shutdown::Both);
                                    drop(stream);
                                    log::info!("{}.run | socket closed", self_id);
                                }
                                Err(err) => {
                                    log::warn!("{}.run | TcpListener::bind error: {:?}", self_id, err);
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
                log::warn!("{}.run | TcpListener::bind error: {:?}", self_id, err);
            }
        };
        log::info!("{}.run | Exit...", self_id);
    });
    match handle {
        Ok(handle) => {
            log::info!("{}.run | Starting - ok", self_id);
            Ok(Handles::from_vec(self_id, vec![handle]))
        }
        Err(err) => {
            let err = Error::new(self_id, "run").pass_with("Start failed", err.to_string());
            log::warn!("{}", err);
            Err(err)
        }
    }
}
