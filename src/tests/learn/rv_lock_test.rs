//!
//! Trying to estimate difference between accessing to the values stored in the HashMap / RwLock<HasMap> / Mutex<HasMap> or using match
#[cfg(test)]
mod tests {
    use hashers::fx_hash::FxHasher;
    use log::error;
    use std::{collections::HashMap, hash::BuildHasherDefault, sync::{mpsc, Arc, Mutex, Once, RwLock}, thread, time::{Duration, Instant}};
    use testing::{entities::test_value::Value, stuff::{max_test_duration::TestDuration, random_test_values::RandomTestValues, wait::WaitTread}};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};

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
    fn init_each(self_id: &str) -> Vec<Value> {
        let test_iterations = 1_000_000;
        RandomTestValues::new(self_id, vec![], test_iterations).collect()
    }
    ///
    ///
    #[ignore = "Learn - all must be ignored"]
    #[test]
    fn map_in_rv_lock() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        println!();
        let self_id = "test access to map behaind RvLock";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = init_each(self_id);

        let test_data_len = test_data.len();
        let map = Arc::new(
            RwLock::new(HashMap::with_hasher(BuildHasherDefault::<FxHasher>::default()))
        );
        let (send, recv) = mpsc::channel();
        for key in KEYS {
            map.write().unwrap().insert(key, send.clone());
        }
        let received = Arc::new(RwLock::new(vec![]));
        let received_clone = received.clone();
        let receiver_handle = thread::Builder::new().name(format!("{} - Read", self_id)).spawn(move || {
            let mut received_local = vec![];
            let mut received_local_len = 0;
            while received_local_len < test_data_len {
                match recv.recv_timeout(Duration::from_secs(10)) {
                    Ok(value) => {
                        received_local.push(value);
                        received_local_len += 1;
                    }
                    Err(err) => {
                        error!("Error receiving value: {:?}", err);
                    }
                }
            }
            *received_clone.write().unwrap() = received_local;
        }).unwrap();
        let sent = Arc::new(RwLock::new(0));
        let sent_clone = sent.clone();
        let timer = Instant::now();
        let sender_handle = thread::Builder::new().name(format!("{} - Read", self_id)).spawn(move || {
            let mut sent_local = 0;
            let mut key;
            let mut key_iter = KEYS.iter().cycle();
            for value in test_data {
                key = key_iter.next().unwrap();
                match map.read().unwrap().get(key) {
                    Some(send) => {
                        match send.send(value) {
                            Ok(_) => {
                                sent_local += 1;
                            }
                            Err(err) => {
                                error!("Error sending value to the sender '{:?}'", err);
                            }
                        }
                    }
                    None => {
                        error!("Error getting sender '{}'", key);
                    }
                }
            }
            *sent_clone.write().unwrap() = sent_local;
        }).unwrap();

        receiver_handle.wait().unwrap();
        sender_handle.wait().unwrap();

        println!("\n{}", self_id);
        println!("Elapsed: {:?}", timer.elapsed());
        println!("Elapsed per event: {:?}", timer.elapsed().div_f32(test_data_len as f32));
        println!("Total test events: {:?}", test_data_len);
        println!("Sent events: {:?}", sent.read().unwrap());
        println!("Received events: {:?}", received.read().unwrap().len());

        test_duration.exit();
    }
    ///
    ///
    #[ignore = "Learn - all must be ignored"]
    #[test]
    fn just_map() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        println!();
        let self_id = "test direct access to map";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = init_each(self_id);

        let test_data_len = test_data.len();
        let mut map = HashMap::with_hasher(BuildHasherDefault::<FxHasher>::default());
        let (send, recv) = mpsc::channel();
        for key in KEYS {
            map.insert(key, send.clone());
        }
        let received = Arc::new(RwLock::new(vec![]));
        let received_clone = received.clone();
        let receiver_handle = thread::Builder::new().name(format!("{} - Read", self_id)).spawn(move || {
            let mut received_local = vec![];
            let mut received_local_len = 0;
            while received_local_len < test_data_len {
                match recv.recv_timeout(Duration::from_secs(10)) {
                    Ok(value) => {
                        received_local.push(value);
                        received_local_len += 1;
                    }
                    Err(err) => {
                        error!("Error receiving value: {:?}", err);
                    }
                }
            }
            *received_clone.write().unwrap() = received_local;
        }).unwrap();
        let sent = Arc::new(RwLock::new(0));
        let sent_clone = sent.clone();
        let timer = Instant::now();
        let sender_handle = thread::Builder::new().name(format!("{} - Read", self_id)).spawn(move || {
            let mut sent_local = 0;
            let mut key;
            let mut key_iter = KEYS.iter().cycle();
            for value in test_data {
                key = key_iter.next().unwrap();
                match map.get(key) {
                    Some(send) => {
                        match send.send(value) {
                            Ok(_) => {
                                sent_local += 1;
                            }
                            Err(err) => {
                                error!("Error sending value to the sender '{:?}'", err);
                            }
                        }
                    }
                    None => {
                        error!("Error getting sender '{}'", key);
                    }
                }
            }
            *sent_clone.write().unwrap() = sent_local;
        }).unwrap();

        receiver_handle.wait().unwrap();
        sender_handle.wait().unwrap();

        println!("\n{}", self_id);
        println!("Elapsed: {:?}", timer.elapsed());
        println!("Elapsed per event: {:?}", timer.elapsed().div_f32(test_data_len as f32));
        println!("Total test events: {:?}", test_data_len);
        println!("Sent events: {:?}", sent.read().unwrap());
        println!("Received events: {:?}", received.read().unwrap().len());

        test_duration.exit();
    }
    ///
    ///
    #[ignore = "Learn - all must be ignored"]
    #[test]
    fn map_in_mutex() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        println!();
        let self_id = "test access to map behaind Mutex";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = init_each(self_id);

        let test_data_len = test_data.len();
        let map = Arc::new(
            Mutex::new(HashMap::with_hasher(BuildHasherDefault::<FxHasher>::default()))
        );
        let (send, recv) = mpsc::channel();
        for key in KEYS {
            map.lock().unwrap().insert(key, send.clone());
        }
        let received = Arc::new(RwLock::new(vec![]));
        let received_clone = received.clone();
        let receiver_handle = thread::Builder::new().name(format!("{} - Read", self_id)).spawn(move || {
            let mut received_local = vec![];
            let mut received_local_len = 0;
            while received_local_len < test_data_len {
                match recv.recv_timeout(Duration::from_secs(10)) {
                    Ok(value) => {
                        received_local.push(value);
                        received_local_len += 1;
                    }
                    Err(err) => {
                        error!("Error receiving value: {:?}", err);
                    }
                }
            }
            *received_clone.write().unwrap() = received_local;
        }).unwrap();
        let sent = Arc::new(RwLock::new(0));
        let sent_clone = sent.clone();
        let timer = Instant::now();
        let sender_handle = thread::Builder::new().name(format!("{} - Read", self_id)).spawn(move || {
            let mut sent_local = 0;
            let mut key;
            let mut key_iter = KEYS.iter().cycle();
            for value in test_data {
                key = key_iter.next().unwrap();
                match map.lock().unwrap().get(key) {
                    Some(send) => {
                        match send.send(value) {
                            Ok(_) => {
                                sent_local += 1;
                            }
                            Err(err) => {
                                error!("Error sending value to the sender '{:?}'", err);
                            }
                        }
                    }
                    None => {
                        error!("Error getting sender '{}'", key);
                    }
                }
            }
            *sent_clone.write().unwrap() = sent_local;
        }).unwrap();

        receiver_handle.wait().unwrap();
        sender_handle.wait().unwrap();

        println!("\n{}", self_id);
        println!("Elapsed: {:?}", timer.elapsed());
        println!("Elapsed per event: {:?}", timer.elapsed().div_f32(test_data_len as f32));
        println!("Total test events: {:?}", test_data_len);
        println!("Sent events: {:?}", sent.read().unwrap());
        println!("Received events: {:?}", received.read().unwrap().len());

        test_duration.exit();
    }
    ///
    ///
    #[ignore = "Learn - all must be ignored"]
    #[test]
    fn matching() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        println!();
        let self_id = "test access values using match";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = init_each(self_id);

        let test_data_len = test_data.len();
        let map = Arc::new(
            Mutex::new(HashMap::with_hasher(BuildHasherDefault::<FxHasher>::default()))
        );
        let (send, recv) = mpsc::channel();
        for key in KEYS {
            map.lock().unwrap().insert(key, send.clone());
        }
        let received = Arc::new(RwLock::new(vec![]));
        let received_clone = received.clone();
        let receiver_handle = thread::Builder::new().name(format!("{} - Read", self_id)).spawn(move || {
            let mut received_local = vec![];
            let mut received_local_len = 0;
            while received_local_len < test_data_len {
                match recv.recv_timeout(Duration::from_secs(10)) {
                    Ok(value) => {
                        received_local.push(value);
                        received_local_len += 1;
                    }
                    Err(err) => {
                        error!("Error receiving value: {:?}", err);
                    }
                }
            }
            *received_clone.write().unwrap() = received_local;
        }).unwrap();
        let sent = Arc::new(RwLock::new(0));
        let sent_clone = sent.clone();
        let timer = Instant::now();
        let sender_handle = thread::Builder::new().name(format!("{} - Read", self_id)).spawn(move || {
            let mut sent_local = 0;
            let mut key;
            let mut key_iter = KEYS.iter().cycle();
            for value in test_data {
                key = key_iter.next().unwrap();
                match *key {
                    "stream_01" => {
                        match send.send(value) {
                            Ok(_) => {
                                sent_local += 1;
                            }
                            Err(err) => error!("Error sending value to the sender '{:?}'", err),
                        }
                    }
                    "stream_03" => {
                        match send.send(value) {
                            Ok(_) => {
                                sent_local += 1;
                            }
                            Err(err) => error!("Error sending value to the sender '{:?}'", err),
                        }
                    }
                    "stream_04" => {
                        match send.send(value) {
                            Ok(_) => {
                                sent_local += 1;
                            }
                            Err(err) => error!("Error sending value to the sender '{:?}'", err),
                        };
                    }
                    "stream_05" => {
                        match send.send(value) {
                            Ok(_) => sent_local += 1,
                            Err(err) => error!("Error sending value to the sender '{:?}'", err),
                        }
                    }
                    "stream_06" => {
                        match send.send(value) {
                            Ok(_) => sent_local += 1,
                            Err(err) => error!("Error sending value to the sender '{:?}'", err),
                        }
                    }
                    "stream_07" => {
                        match send.send(value) {
                            Ok(_) => sent_local += 1,
                            Err(err) => error!("Error sending value to the sender '{:?}'", err),
                        }
                    }
                    "stream_08" => {
                        match send.send(value) {
                            Ok(_) => sent_local += 1,
                            Err(err) => error!("Error sending value to the sender '{:?}'", err),
                        }
                    }
                    "stream_09" => {
                        match send.send(value) {
                            Ok(_) => sent_local += 1,
                            Err(err) => error!("Error sending value to the sender '{:?}'", err),
                        }
                    }
                    "stream_10" => {
                        match send.send(value) {
                            Ok(_) => sent_local += 1,
                            Err(err) => error!("Error sending value to the sender '{:?}'", err),
                        }
                    }
                    "stream_11" => {
                        match send.send(value) {
                            Ok(_) => sent_local += 1,
                            Err(err) => error!("Error sending value to the sender '{:?}'", err),
                        }
                    }
                    "stream_12" => {
                        match send.send(value) {
                            Ok(_) => sent_local += 1,
                            Err(err) => error!("Error sending value to the sender '{:?}'", err),
                        }
                    }
                    "stream_13" => {
                        match send.send(value) {
                            Ok(_) => sent_local += 1,
                            Err(err) => error!("Error sending value to the sender '{:?}'", err),
                        }
                    }
                    "stream_14" => {
                        match send.send(value) {
                            Ok(_) => sent_local += 1,
                            Err(err) => error!("Error sending value to the sender '{:?}'", err),
                        }
                    }
                    "stream_15" => {
                        match send.send(value) {
                            Ok(_) => sent_local += 1,
                            Err(err) => error!("Error sending value to the sender '{:?}'", err),
                        }
                    }
                    _ => panic!("Unknown key '{}'", key),
                }
            }
            *sent_clone.write().unwrap() = sent_local;
        }).unwrap();

        receiver_handle.wait().unwrap();
        sender_handle.wait().unwrap();

        println!("\n{}", self_id);
        println!("Elapsed: {:?}", timer.elapsed());
        println!("Elapsed per event: {:?}", timer.elapsed().div_f32(test_data_len as f32));
        println!("Total test events: {:?}", test_data_len);
        println!("Sent events: {:?}", sent.read().unwrap());
        println!("Received events: {:?}", received.read().unwrap().len());

        test_duration.exit();
    }
    const KEYS: [&str; 14] = [
        "stream_01",
        "stream_03",
        "stream_04",
        "stream_05",
        "stream_06",
        "stream_07",
        "stream_08",
        "stream_09",
        "stream_10",
        "stream_11",
        "stream_12",
        "stream_13",
        "stream_14",
        "stream_15",
    ];
}
