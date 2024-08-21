#[cfg(test)]

mod tcp_client_connect {
    use crate::tcp::tcp_client_connect::TcpClientConnect;
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use log::{debug, info, warn};
    use std::{
        net::TcpListener,
        sync::{
            atomic::{AtomicBool, Ordering}, Arc, Mutex, Once
        },
        thread,
        time::Duration,
    };
    use testing::{session::test_session::TestSession, stuff::max_test_duration::TestDuration};
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
    /// Testing success connection case
    #[test]
    fn success_connection() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        println!("test success connection");
        let test_duration = TestDuration::new("tcp_client_connect/success_connection", Duration::from_secs(10));
        test_duration.run().unwrap();
        let addr = "127.0.0.1:".to_owned() + &TestSession::free_tcp_port_str();
        let timeout = Duration::from_millis(4500); // ms
        let exit = Arc::new(AtomicBool::new(false));
        let connect = TcpClientConnect::new(
            "test",
            &addr,
            Duration::from_millis(500),
            Some(exit.clone()),
        );
        let ok = Arc::new(AtomicBool::new(false));
        let ok_ref = ok.clone();
        // let connectExit = connect.exit();
        thread::spawn(move || {
            info!("Preparing test TCP server...");
            thread::sleep(Duration::from_millis(300));
            match TcpListener::bind(addr) {
                Ok(listener) => {
                    info!("Preparing test TCP server - ok");
                    match listener.accept() {
                        Ok((_socket, addr)) => {
                            info!("incoming connection - ok\n\t{:?}", addr);
                        }
                        Err(err) => {
                            info!("incoming connection - error: {:?}", err);
                        }
                    }
                }
                Err(err) => {
                    // connectExit.send(true);
                    ok_ref.store(false, Ordering::SeqCst);
                    panic!("Preparing test TCP server - error: {:?}", err);
                }
            };
        });
        let connect = Arc::new(Mutex::new(connect));
        let ok_ref = ok.clone();
        let exit_ref = exit.clone();
        thread::spawn(move || {
            info!("Waiting for connection...");
            thread::sleep(timeout);
            if !ok_ref.load(Ordering::SeqCst) {
                ok_ref.store(false, Ordering::SeqCst);
                warn!("Tcp socket was not connected in {:?}", timeout);
                debug!("stopping...");
                exit_ref.store(true, Ordering::SeqCst);
            }
        });
        info!("Connecting...");
        for _ in 0..10 {
            match connect.lock().unwrap().connect() {
                Some(tcp_stream) => {
                    ok.store(true, Ordering::SeqCst);
                    info!("connected: {:?}", tcp_stream);
                    exit.store(true, Ordering::SeqCst);
                    break;
                }
                None => {
                    warn!("not connected");
                }
            };
            thread::sleep(Duration::from_millis(100));
        }
        assert!(
            ok.load(Ordering::SeqCst) == true,
            "\nresult: connected - {:?}\ntarget: connected - {:?}",
            ok,
            true,
        );
        test_duration.exit();
    }
    ///
    /// Testing connection fail case
    #[test]
    fn failure_connection() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        println!("test failure connection");
        let test_duration = TestDuration::new("tcp_client_connect/failure_connection", Duration::from_secs(10));
        test_duration.run().unwrap();
        let timeout = Duration::from_millis(1500); // ms
        let addr = "127.0.0.1:".to_owned() + &TestSession::free_tcp_port_str();
        let exit = Arc::new(AtomicBool::new(false));
        let mut connect = TcpClientConnect::new("test", &addr, Duration::from_millis(500), Some(exit.clone()));
        let ok = Arc::new(AtomicBool::new(false));
        let ok_ref = ok.clone();
        thread::spawn(move || {
            info!("Waiting for connection...");
            thread::sleep(timeout);
            ok_ref.store(false, Ordering::SeqCst);
            warn!("Tcp socket was not connected in {:?}", timeout);
            debug!("Thread | stopping...");
            exit.store(true, Ordering::SeqCst);
            debug!("Thread | stopping - ok");
        });
        info!("Connecting...");
        match connect.connect() {
            Some(tcp_stream) => {
                ok.store(true, Ordering::SeqCst);
                info!("connected: {:?}", tcp_stream);
            }
            None => {
                warn!("not connected");
            }
        };
        assert!(
            ok.load(Ordering::SeqCst) == false,
            "\nresult: connected - {:?}\ntarget: connected - {:?}",
            ok,
            false,
        );
        test_duration.exit();
    }
}
