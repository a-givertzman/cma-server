use std::sync::{atomic::{AtomicBool, AtomicUsize, Ordering}};

///
/// Thread safe, lock free `AtomicUsize` `Option`
pub struct AtomicUsizeOption {
    val: AtomicUsize,
    option: AtomicBool,
}
//
//
impl AtomicUsizeOption {
    ///
    /// Returns [AtomicUsizeOption] new instance
    pub fn new(val: Option<usize>) -> Self {
        Self {
            val: AtomicUsize::new(val.unwrap_or(0)),
            option: AtomicBool::new(val.is_some()),
        }
    }
    ///
    /// Returns containing value
    pub fn load(&self) -> Option<usize> {
        match self.option.load(Ordering::Acquire) {
            true => Some(self.val.load(Ordering::Acquire)),
            false => None,
        }
    }
    ///
    /// Stores a containing value
    pub fn store(&self, val: Option<usize>) -> Option<usize> {
        match val {
            Some(val) => {
                if !self.option.load(Ordering::Acquire) {
                    self.option.store(true, Ordering::Release);
                }
                self.val.store(val, Ordering::Release);
            }
            None => {
                if self.option.load(Ordering::Acquire) {
                    self.option.store(false, Ordering::Release);
                }
            }
        }
        match self.option.load(Ordering::Acquire) {
            true => Some(self.val.load(Ordering::Acquire)),
            false => None,
        }
    }
}
//
//
#[cfg(test)]
///
/// Testing [AtomicUsizeOption]
#[test]
fn new() {
    use sal_core::dbg::Dbg;
    use testing::stuff::max_test_duration::TestDuration;
    use std::{sync::Arc, time::{Duration, Instant}};

    env_logger::Builder::new().filter_level(log::LevelFilter::Debug).init();
    let dbg = Dbg::own("AtomicUsizeOption-test");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        (00, Some(0)),
        (01, Some(1)),
        (02, Some(2)),
        (03, Some(3)),
        (04, Some(4)),
        (05, Some(5)),
        (06, Some(6)),
        (07, Some(7)),
        (08, Some(8)),
        (09, None),
    ];
    let (next_tx, next_rx) = std::sync::mpsc::channel();
    let val = Arc::new(AtomicUsizeOption::new(None));
    let val_ref = val.clone();
    let mut handles = vec![];
    let t = Instant::now();
    for i in 0..10 {
        let val = val.clone();
        let dbg = dbg.clone();
        let target = test_data[i].1;
        let next_tx = next_tx.clone();
        let handle = std::thread::spawn(move || {
            loop {
                let value = val.load();
                if value == target {
                    log::debug!("{dbg} | Exit {i}, Elapsed: {:?}", t.elapsed());
                    next_tx.send(value).unwrap();
                    break
                } else {
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
        });
        handles.push(handle);
    }
    for (step, val) in test_data {
        val_ref.store(val);
        let result = next_rx.recv().unwrap();
        assert!(result == val, "{dbg} | step {step}  \nresult: {:?}\ntarget: {:?}", result, val);
    }
    let _: Vec<()> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    log::debug!("{dbg} | Elapsed: {:?}", t.elapsed());
    test_duration.exit();
}
