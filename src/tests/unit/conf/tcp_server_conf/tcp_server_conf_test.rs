#[cfg(test)]

use std::sync::Once;
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
use crate::services::server::TcpServerConf;
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
///
#[test]
fn test_tcp_server_config() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    let self_id = "test TcpServerConfig";
    println!("\n{}", self_id);
    let test_data = [
        format!(r#"
            service TcpServer:
                cycle: 1 ms
                reconnect: 1 s  # default 3 s
                address: 127.0.0.1:8080
                auth: none      # auth: none / auth-secret: pass: ... / auth-ssh: path: ...
                in queue link:
                    max-length: 10000
                send-to: MultiQueue.in-queue
        "#),
        format!(r#"
            service TcpServer:
                cycle: 1 ms
                reconnect: 1 s  # default 3 s
                address: 127.0.0.1:8080
                auth-secret:    # auth: none / auth-secret: pass: ... / auth-ssh: path: ...
                    pass: secret-password
                in queue link:
                    max-length: 10000
                send-to: MultiQueue.in-queue
        "#),
        format!(r#"
            service TcpServer:
                cycle: 1 ms
                reconnect: 1 s  # default 3 s
                address: 127.0.0.1:8080
                auth-ssh:    # auth: none / auth-secret: pass: ... / auth-ssh: path: ...
                    path: /home/scada/.ssh/
                in queue link:
                    max-length: 10000
                send-to: MultiQueue.in-queue
        "#),
    ];
    for conf in test_data {
        let conf = serde_yaml::from_str(&conf).unwrap();
        let conf = TcpServerConf::from_yaml(self_id, &conf);
        log::info!("conf: \n{:?}", conf);
        // assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
    }
}
