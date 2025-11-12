#[cfg(test)]

use std::{str::FromStr, sync::Once, time::Duration};
use sal_sync::{collections::FxIndexMap, services::{conf::DiagKeywd, entity::{Name, {PointConf, PointConfHistory, PointConfType}}, LinkName, ConfSubscribe}};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
use crate::services::udp_client::UdpClientConf;
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
/// Testing such functionality / behavior
#[test]
fn new() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("");
    let dbg = "test";
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    // fn FxIndexMap() {
    //     IndexMap::with_hasher(BuildHasherDefault::)
    // }
    let test_data = [
        (
            01,
            format!(r#"service UdpClient:
                description: 'UDP-IED-01.01'
                subscribe: Multiqueue
                send-to: MultiQueue.in-queue
                cycle: 1 ms                         # operating cycle time of the device
                reconnect: 1000 ms                  # reconnect timeout when connection is lost
                protocol: 'udp-raw'
                local-address: 192.168.100.100:15180
                remote-address: 192.168.100.241:15180
                diagnosis:                          # internal diagnosis
                    point Status:                   # Ok(0) / Invalid(10)
                        type: 'Int'
                        # history: r
                    point Connection:               # Ok(0) / Invalid(10)
                        type: 'Int'
                        # history: r
                point Sensor1:                  # Device input sensor
                    type: 'Int'
                    input: 0                    # the number of input 0..8 (0 - first input channel)
                point Sensor2:                  # Device input sensor
                    type: 'Int'
                    input: 1                    # the number of input 0..8 (0 - first input channel)
            "#),
            UdpClientConf {
                name: Name::new(dbg, "UdpClient"),
                description: "UDP-IED-01.01".to_owned(),
                subscribe: ConfSubscribe::new(serde_yaml::from_str("Multiqueue").unwrap()),
                send_to: LinkName::from_str("MultiQueue.in-queue").unwrap(),
                cycle: Some(
                    Duration::from_millis(1),
                ),
                reconnect: Duration::from_secs(1),
                protocol: "udp-raw".to_owned(),
                local_addr: "192.168.100.100:15180".to_owned(),
                remote_addr: "192.168.100.241:15180".to_owned(),
                mtu: 1500,
                diagnosis: FxIndexMap::from_iter([
                    (DiagKeywd::Status, PointConf {
                        id: 0,
                        name: Name::new(dbg, "UdpClient/Status").join(),
                        type_: PointConfType::Int,
                        history: PointConfHistory::None,
                        alarm: None,
                        address: None,
                        filters: None,
                        comment: None,
                    }),
                    (DiagKeywd::Connection, PointConf {
                        id: 0,
                        name: Name::new(dbg, "UdpClient/Connection").join(),
                        type_: PointConfType::Int,
                        history: PointConfHistory::None,
                        alarm: None,
                        address: None,
                        filters: None,
                        comment: None,
                    }),
                ]),
                points: Vec::from([
                    PointConf {
                        id: 0,
                        name: Name::new(dbg, "UdpClient/Sensor1").join(),
                        type_: PointConfType::Int,
                        history: PointConfHistory::None,
                        alarm: None,
                        address: None,
                        filters: None,
                        comment: None,
                    },
                    PointConf {
                        id: 1,
                        name: Name::new(dbg, "UdpClient/Sensor2").join(),
                        type_: PointConfType::Int,
                        history: PointConfHistory::None,
                        alarm: None,
                        address: None,
                        filters: None,
                        comment: None,
                    },
                ]),
            },
        ),
        (
            02,
            format!(r#"service UdpClient UdpIed01:
                description: 'UDP-IED-01.01'
                subscribe:
                    Multiqueue:
                        Act: []
                send-to: MultiQueue.in-queue
                cycle: 10 ms                         # operating cycle time of the device
                reconnect: 3000 ms                  # reconnect timeout when connection is lost
                protocol: 'udp-raw'
                local-address: 192.168.100.100:15180
                remote-address: 192.168.100.241:15180
                mtu: 4096
                point Sensor1: 
                    type: 'Int'
                    input: 0                    # the number of input 0..8 (0 - first input channel)
                point Sensor2: 
                    type: 'Int'
                    input: 1                    # the number of input 0..8 (0 - first input channel)
                point Sensor3: 
                    type: 'Real'
                    input: 2                    # the number of input 0..8 (0 - first input channel)
                point Sensor4: 
                    type: 'Double'
                    input: 3                    # the number of input 0..8 (0 - first input channel)
            "#),
            UdpClientConf {
                name: Name::new(dbg, "UdpIed01"),
                description: "UDP-IED-01.01".to_owned(),
                subscribe: ConfSubscribe::new(serde_yaml::from_str(r#"Multiqueue: 
                                                                            Act: []"#).unwrap()),
                send_to: LinkName::from_str("MultiQueue.in-queue").unwrap(),
                cycle: Some(
                    Duration::from_millis(10),
                ),
                reconnect: Duration::from_secs(3),
                protocol: "udp-raw".to_owned(),
                local_addr: "192.168.100.100:15180".to_owned(),
                remote_addr: "192.168.100.241:15180".to_owned(),
                mtu: 4096,
                diagnosis: FxIndexMap::from_iter([]),
                points: Vec::from([
                    PointConf {
                        id: 0,
                        name: Name::new(dbg, "UdpIed01/Sensor1").join(),
                        type_: PointConfType::Int,
                        history: PointConfHistory::None,
                        alarm: None,
                        address: None,
                        filters: None,
                        comment: None,
                    },
                    PointConf {
                        id: 1,
                        name: Name::new(dbg, "UdpIed01/Sensor2").join(),
                        type_: PointConfType::Int,
                        history: PointConfHistory::None,
                        alarm: None,
                        address: None,
                        filters: None,
                        comment: None,
                    },
                    PointConf {
                        id: 2,
                        name: Name::new(dbg, "UdpIed01/Sensor3").join(),
                        type_: PointConfType::Real,
                        history: PointConfHistory::None,
                        alarm: None,
                        address: None,
                        filters: None,
                        comment: None,
                    },
                    PointConf {
                        id: 3,
                        name: Name::new(dbg, "UdpIed01/Sensor4").join(),
                        type_: PointConfType::Double,
                        history: PointConfHistory::None,
                        alarm: None,
                        address: None,
                        filters: None,
                        comment: None,
                    },
                ]),
            },                
        )
    ];
    for (step, conf, target) in test_data {
        let conf = serde_yaml::from_str(&conf).unwrap();
        let result = UdpClientConf::from_yaml(dbg, &conf);
        log::debug!("{}  |  conf: {:#?}", step, result);
        assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    }
    test_duration.exit();
}
