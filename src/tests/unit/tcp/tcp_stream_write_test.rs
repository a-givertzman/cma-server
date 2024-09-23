use sal_sync::services::entity::{name::Name, object::Object};
use crate::{
    core_::failure::recv_error::RecvError,
    tcp::steam_read::StreamRead,
};
#[cfg(test)]
mod tcp_stream_write {
    use crate::{
        core_::net::connection_status::ConnectionStatus,
        tcp::tcp_stream_write::{OpResult, TcpStreamWrite},
        tests::unit::tcp::tcp_stream_write_test::MockStreamRead,
    };
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use log::{debug, info, warn};
    use rand::Rng;
    use std::{
        io::Read,
        net::{TcpListener, TcpStream},
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc, Mutex, Once,
        },
        thread,
        time::{Duration, Instant},
    };
    use testing::session::test_session::TestSession;
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
    fn init_each() -> () {}
    ///
    static INDEX: AtomicUsize = AtomicUsize::new(0);
    ///
    fn random_bytes(len: usize) -> Vec<u8> {
        let mut rnd = rand::thread_rng();
        let mut bytes = vec![];
        let ix = INDEX.load(Ordering::SeqCst);
        for _ in ix..ix + len {
            let b = rnd.gen_range(0..255);
            bytes.push(b);
        }
        INDEX.fetch_add(10, Ordering::SeqCst);
        bytes
    }
    ///
    /// Testing TcpStreamWrite
    #[test]
    fn test() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test TcpStreamWrite";
        println!("\n{}", self_id);
        let count = 1000;
        let test_duration = Duration::from_secs(10);
        let sent = Arc::new(AtomicUsize::new(0));
        let received = Arc::new(Mutex::new(vec![]));
        let message_len = 10;
        let mut test_data = vec![];
        for _ in 0..count {
            test_data.push(random_bytes(message_len))
        }
        info!("test_data: {:?}", test_data);
        let mut tcp_stream_write = TcpStreamWrite::new(
            "test",
            true,
            Some(10000),
            Box::new(MockStreamRead::new(self_id, test_data.clone())),
        );
        let addr = "127.0.0.1:".to_owned() + &TestSession::free_tcp_port_str();
        mock_tcp_server(addr.clone(), count, message_len, received.clone());
        thread::sleep(Duration::from_micros(100));
        let timer = Instant::now();
        while sent.load(Ordering::SeqCst) < count {
            // warn!("sent: {}/{}", sent, count);
            match TcpStream::connect(&addr) {
                Ok(mut stream) => {
                    'inner: while sent.load(Ordering::SeqCst) < count {
                        match tcp_stream_write.write(&mut stream) {
                            ConnectionStatus::Active(result) => match result {
                                OpResult::Ok(_) => {
                                    sent.fetch_add(1, Ordering::SeqCst);
                                    debug!("sent: {}/{}", sent.load(Ordering::SeqCst), count);
                                }
                                OpResult::Err(err) => {
                                    warn!(
                                        "sent: {}/{}, socket write error: {}",
                                        sent.load(Ordering::SeqCst),
                                        count,
                                        err,
                                    );
                                }
                                OpResult::Timeout() => {}
                            },
                            ConnectionStatus::Closed(err) => {
                                warn!(
                                    "sent: {}/{}, socket closed, error: {}",
                                    sent.load(Ordering::SeqCst),
                                    count,
                                    err,
                                );
                                break 'inner;
                            }
                        };
                    }
                }
                Err(err) => {
                    warn!(
                        "sent: {}/{}, connection error: {}",
                        sent.load(Ordering::SeqCst),
                        count,
                        err,
                    );
                    thread::sleep(Duration::from_millis(100));
                }
            };
            assert!(
                timer.elapsed() < test_duration,
                "Transfering {}/{} messages taks too mach time {:?} of {:?}",
                received.lock().unwrap().len(),
                count,
                timer.elapsed(),
                test_duration,
            );
            // assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
            warn!("sent total: {}/{}", sent.load(Ordering::SeqCst), count);
        }
        let wait_duration = Duration::from_millis(10);
        let mut wait_attempts = test_duration.as_micros() / wait_duration.as_micros();
        while received.lock().unwrap().len() < count {
            debug!(
                "waiting while all data beeng received {}/{}...",
                received.lock().unwrap().len(),
                count,
            );
            thread::sleep(wait_duration);
            wait_attempts -= 1;
            assert!(
                wait_attempts > 0,
                "Transfering {}/{} messages taks too mach time {:?} of {:?}",
                received.lock().unwrap().len(),
                count,
                timer.elapsed(),
                test_duration,
            );
        }
        println!("elapsed: {:?}", timer.elapsed());
        println!("total test events: {:?}", count);
        println!("sent events: {:?}", sent);
        let mut received = received.lock().unwrap();
        println!("recv events: {:?}", received.len());
        assert!(
            sent.load(Ordering::SeqCst) == count,
            "sent: {:?}\ntarget: {:?}",
            sent,
            count,
        );
        assert!(
            received.len() == count,
            "received: {:?}\ntarget: {:?}",
            received.len(),
            count,
        );
        for target in test_data {
            let result = match received.first() {
                Some(bytes) => {
                    debug!("\nresult: {:?}\ntarget: {:?}", bytes, target);
                    bytes
                }
                None => panic!("received is empty"),
            };
            assert!(
                result == &target,
                "\nresult: {:?}\ntarget: {:?}",
                result,
                target,
            );
            received.remove(0);
        }
    }
    ///
    /// TcpServer setup
    fn mock_tcp_server(
        addr: String,
        count: usize,
        message_len: usize,
        received: Arc<Mutex<Vec<Vec<u8>>>>,
    ) {
        thread::spawn(move || {
            info!("TCP server | Preparing test server...");
            match TcpListener::bind(&addr) {
                Ok(listener) => {
                    info!("TCP server | Preparing test server - ok ({})", addr);
                    let mut accept_count = 2;
                    // let mut maxReadErrors = 3;
                    while accept_count > 0 {
                        accept_count -= 1;
                        info!("TCP server | waiting connection...");
                        match listener.accept() {
                            Ok((mut _socket, addr)) => {
                                info!("TCP server | accept connection - ok\n\t{:?}", addr);
                                let mut buffer = Vec::new();
                                while received.lock().unwrap().len() < count {
                                    let mut bytes = vec![0u8; message_len];
                                    match _socket.read(&mut bytes) {
                                        Ok(_) => {
                                            buffer.append(&mut bytes);
                                            if buffer.len() >= message_len {
                                                let v = buffer.drain(0..message_len).collect();
                                                debug!("TCP server | received: {:?}", v);
                                                received.lock().unwrap().push(v);
                                            }
                                        }
                                        Err(err) => {
                                            warn!("{:?}", err);
                                        }
                                    }
                                }
                                info!(
                                    "TCP server | all received: {:?}",
                                    received.lock().unwrap().len(),
                                );
                            }
                            Err(err) => {
                                warn!("TCP server | incoming connection - error: {:?}", err);
                            }
                        }
                    }
                }
                Err(err) => {
                    // connectExit.send(true).unwrap();
                    // okRef.store(false, Ordering::SeqCst);
                    panic!("TCP server | Preparing server - error: {:?}", err);
                }
            };
        });
    }
}
///
///
#[derive(Debug)]
struct MockStreamRead<T> {
    id: String,
    name: Name,
    buffer: Vec<T>,
}
//
//
impl<T> MockStreamRead<T> {
    pub fn new(parent: &str, buffer: Vec<T>) -> Self {
        let name = Name::new(parent, "MockStreamRead");
        Self {
            id: name.join(),
            name,
            buffer,
        }
    }
}
//
//
impl<T> Object for MockStreamRead<T> {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
//
impl<T: Send + std::fmt::Debug> StreamRead<T, RecvError> for MockStreamRead<T> {
    fn read(&mut self) -> Result<T, RecvError> {
        match self.buffer.first() {
            Some(_) => Ok(self.buffer.remove(0)),
            None => Err(RecvError::Timeout),
        }
    }
}
