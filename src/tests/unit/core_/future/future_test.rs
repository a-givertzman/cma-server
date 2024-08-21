#[cfg(test)]

mod future {
    use log::debug;
    use rand::Rng;
    use std::{sync::Once, thread::{self}, time::{Duration, Instant}};
    use testing::stuff::{max_test_duration::TestDuration, wait::WaitTread};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::core_::future::Future;
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
    /// Testing Future.then
    #[test]
    fn then() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = [
            (00, 123.0, 123.0 * 2.0),
            (01, 456.0, 456.0 * 2.0),
            (02, 789.0, 789.0 * 2.0),
            (03, -321.0, -321.0 * 2.0),
            (04, -435.0, -435.0 * 2.0),
        ];
        let mut handles = vec![];
        for (step, value, target) in test_data {
            let time = Instant::now();
            let result = calc(value);
            let h = thread::spawn(move || {
                let _ = result.then(
                    |event| {
                        let result = event.clone().unwrap();
                        debug!("step {}   |   value: {}   |   result: {}   |   elapsed {:?}", step, value, result, time.elapsed());
                        assert!(result == target, "step {}\nresult: {:?}\ntarget: {:?}", step, result, target);
                        event
                    },
                    |err| {
                        debug!("step {}   |   value: {}   |   error: {:?}   |   elapsed {:?}", step, value, err, time.elapsed());
                        Err(err)
                    },
                );
            });
            handles.push(h);
        }
        for h in handles {
            h.wait().unwrap();
        }
        test_duration.exit();
    }
    ///
    /// 
    fn calc(value: f64) -> Future<Result<f64, String>> {
        let (future, sink) = Future::<Result<f64, String>>::new();
        thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let millis = rng.gen_range(10u64..100);
            thread::sleep(Duration::from_millis(millis));
            sink.add(Ok(value * 2.0));
            
        });
        future
    }
}
