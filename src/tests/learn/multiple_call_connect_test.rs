#[cfg(test)]
mod tests {
    use rand::{Rng, RngExt};
    use std::{sync::{atomic::{AtomicUsize, Ordering}, Arc}, thread};
    use std::{sync::Once, time::Duration};
    use debugging::session::debug_session::{DebugSession, LogLevel};

    use crate::domain::Mutex;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    // use super::*;

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

    #[ignore = "Learn - all must be ignored"]
    #[test]
    fn test_task_cycle() {
        DebugSession::new().filter(LogLevel::Info).init();
        init_once();
        init_each();
        println!("test_task_cycle");
        let mut connect = TestConnect::new();
        let mut closed = false;
        for iteration in 0..10 {
            println!("\n iteration: '{}'", iteration);
                loop {
                    let mut connected = 0;
                    match connect.connect(closed) {
                        Ok(stream) => {
                            println!("stream 1: '{}'", stream);
                            connected += 1;
                            closed = false;
                        }
                        Err(err) => {
                            println!("error 1: '{}'", err);
                        }
                    };
                    match connect.connect(closed) {
                        Ok(stream) => {
                            println!("stream 2: '{}'", stream);
                            connected += 1;
                            closed = false;
                        }
                        Err(err) => {
                            println!("error 2: '{}'", err);
                        }
                    };
                    if connected >= 2 {break;}
                    thread::sleep(Duration::from_millis(300))
                }
           closed = true;
        }
        // assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
    }

    #[derive(Debug, PartialEq)]
    enum ConnectState {
        Closed,
        Connecting,
        Connected,
    }
    impl ConnectState {
        fn from(value: usize) -> Self {
            match value {
                0 => ConnectState::Closed,
                1 => ConnectState::Connecting,
                2 => ConnectState::Connected,
                _ => panic!("Invalid value: '{}'", value)
            }
        }
        fn value(&self) -> usize {
            match self {
                ConnectState::Closed => 0,
                ConnectState::Connecting => 1,
                ConnectState::Connected => 2,
            }
        }
    }
    struct TestConnect {
        state: Arc<AtomicUsize>,
        stream: Arc<Mutex<Vec<String>>>,
    }
    impl TestConnect {
        pub fn new() -> Self {
            Self {
                state: Arc::new(AtomicUsize::new(ConnectState::Closed.value())),
                stream: Arc::new(Mutex::new(Vec::new()))
            }
        }
        pub fn connect(&mut self, closed: bool) -> Result<String, String> {
            match ConnectState::from( self.state.load(Ordering::Relaxed) ) {
                ConnectState::Closed => {
                    self.connect_stream();
                }
                ConnectState::Connecting => {}
                ConnectState::Connected => {
                    if closed {
                        self.state.store(ConnectState::Closed.value(), Ordering::SeqCst);
                        self.connect_stream();
                    }
                }
            };
            match ConnectState::from( self.state.load(Ordering::Relaxed) ) {
                ConnectState::Connected => {
                    let stream = self.stream.lock().pop().unwrap();
                    let stream_clone= stream.clone();
                    self.stream.lock().push(stream);
                    Ok(stream_clone)
                }
                _ => Err(String::from(format!("{:?}", ConnectState::from( self.state.load(Ordering::Relaxed) )))),
            }
        }
        fn connect_stream(&self) {
            if ConnectState::from( self.state.load(Ordering::Relaxed) ) == ConnectState::Closed {
                self.state.store(ConnectState::Connecting.value(), Ordering::SeqCst);
                let state = self.state.clone();
                let stream = self.stream.clone();
                let h = thread::spawn(move || {
                    let mut rnd = rand::rng();
                    println!("TestConnect | connecting...");
                    thread::sleep(Duration::from_millis(20));
                    match rnd.random_bool(0.7) {
                        true => {
                            println!("TestConnect | connecting - ok");
                            stream.lock().push(format!("Stream"));
                            state.store(ConnectState::Connected.value(), Ordering::SeqCst)
                        }
                        false => {
                            state.store(ConnectState::Closed.value(), Ordering::SeqCst);
                            println!("TestConnect | connecting - error");
                        }
                    };
                });
                h.join().unwrap();
            }
        }
    }
}
