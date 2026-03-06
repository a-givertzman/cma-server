#[cfg(test)]

use std::{sync::Once, thread, time::Duration};
use debugging::session::debug_session::{DebugSession, LogLevel};
use sal_core::dbg::Dbg;
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
fn init_each() {}
///
/// If thread is already finished, join() or wait() don't returns error
// #[ignore = "Learn - all must be ignored"]
#[test]
fn exiting() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    init_each();
    let dbg = Dbg::own("kanal_channel_test");
    log::debug!("{}", dbg);
    let (send, recv) = kanal::unbounded();
    let handler = thread::spawn(move|| {
        log::info!("thread | Started");
        _ = send.send(0);
        for i in 0..10 {
            log::info!("thread | iteration: {}", i);
        }
        std::thread::sleep(Duration::from_secs(3));
        drop(send);
        log::info!("thread | Finished");
    });
    loop {
        log::info!("{dbg} | loop | receiving...");
        match recv.recv() {
            Ok(v) => {
                log::warn!("{dbg} | Recv value: '{:?}'", v);
            }
            Err(err) => {
                log::warn!("{dbg} | Recv error: {:?}", err);
                break;
            }
        }
    }
    log::info!("{dbg} | loop | exited");
    std::thread::sleep(Duration::from_millis(3000));
    handler.join().unwrap();
    // assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
}
