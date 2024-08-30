#[cfg(test)]

mod udp_client_config {
    use std::{sync::Once, time::Duration};
    use sal_sync::{collections::map::IndexMapFxHasher, services::{entity::{name::Name, point::{point_config::PointConfig, point_config_history::PointConfigHistory, point_config_type::PointConfigType}}, service::link_name::LinkName, subscription::conf_subscribe::ConfSubscribe}};
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};

    use crate::conf::{diag_keywd::DiagKeywd, udp_client_config::{udp_client_config::UdpClientConfig, udp_client_db_config::UdpClientDbConfig}};
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
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let self_id = "test";
        log::debug!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(1));
        test_duration.run().unwrap();
        // fn indexMapFxHasher() {
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
                    db data:                            # multiple DB blocks are allowed, must have unique namewithing parent device
                        description: 'Data block of the device'
                        size: 1024                      # corresponding to the length of the array transmitted in the UDP message
                        point Sensor1: 
                            type: 'Int'
                            input: 0                    # the number of input 0..8 (0 - first input channel)
                        point Sensor2: 
                            type: 'Int'
                            input: 0                    # the number of input 0..8 (0 - first input channel)
                "#),
                UdpClientConfig {
                    name: Name::new(self_id, "UdpClient"),
                    description: "UDP-IED-01.01".to_owned(),
                    subscribe: ConfSubscribe::new(serde_yaml::from_str("Multiqueue").unwrap()),
                    send_to: LinkName::new("MultiQueue.in-queue").validate(),
                    cycle: Some(
                        Duration::from_millis(1),
                    ),
                    reconnect: Duration::from_secs(1),
                    protocol: "udp-raw".to_owned(),
                    local_addr: "192.168.100.100:15180".to_owned(),
                    remote_addr: "192.168.100.241:15180".to_owned(),
                    diagnosis: IndexMapFxHasher::from_iter([
                        (DiagKeywd::Status, PointConfig {
                            id: 0,
                            name: Name::new(self_id, "UdpClient/Status").join(),
                            type_: PointConfigType::Int,
                            history: PointConfigHistory::None,
                            alarm: None,
                            address: None,
                            filters: None,
                            comment: None,
                        }),
                        (DiagKeywd::Connection, PointConfig {
                            id: 0,
                            name: Name::new(self_id, "UdpClient/Connection").join(),
                            type_: PointConfigType::Int,
                            history: PointConfigHistory::None,
                            alarm: None,
                            address: None,
                            filters: None,
                            comment: None,
                        }),
                    ]),
                    dbs: IndexMapFxHasher::from_iter([
                        ("data".to_owned(), UdpClientDbConfig {
                            name: Name::new(self_id, "UdpClient/data"),
                            description: "Data block of the device".to_owned(),
                            size: 1024,
                            points: [
                                PointConfig {
                                    id: 0,
                                    name: Name::new(self_id, "UdpClient/data/Sensor1").join(),
                                    type_: PointConfigType::Int,
                                    history: PointConfigHistory::None,
                                    alarm: None,
                                    address: None,
                                    filters: None,
                                    comment: None,
                                },
                                PointConfig {
                                    id: 0,
                                    name: Name::new(self_id, "UdpClient/data/Sensor2").join(),
                                    type_: PointConfigType::Int,
                                    history: PointConfigHistory::None,
                                    alarm: None,
                                    address: None,
                                    filters: None,
                                    comment: None,
                                },
                            ].to_vec(),
                        }),
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
                    db data:                            # multiple DB blocks are allowed, must have unique namewithing parent device
                        description: 'Data block of the device'
                        size: 1024                      # corresponding to the length of the array transmitted in the UDP message
                        point Sensor1: 
                            type: 'Int'
                            input: 0                    # the number of input 0..8 (0 - first input channel)
                        point Sensor2: 
                            type: 'Int'
                            input: 1                    # the number of input 0..8 (0 - first input channel)
                        point Sensor3: 
                            type: 'Real'
                            input: 2                    # the number of input 0..8 (0 - first input channel)
                            history: rw
                        point Sensor4: 
                            type: 'Double'
                            input: 3                    # the number of input 0..8 (0 - first input channel)
                            history: r
                "#),
                UdpClientConfig {
                    name: Name::new(self_id, "UdpIed01"),
                    description: "UDP-IED-01.01".to_owned(),
                    subscribe: ConfSubscribe::new(serde_yaml::from_str(r#"Multiqueue: 
                                                                                Act: []"#).unwrap()),
                    send_to: LinkName::new("MultiQueue.in-queue").validate(),
                    cycle: Some(
                        Duration::from_millis(10),
                    ),
                    reconnect: Duration::from_secs(3),
                    protocol: "udp-raw".to_owned(),
                    local_addr: "192.168.100.100:15180".to_owned(),
                    remote_addr: "192.168.100.241:15180".to_owned(),
                    diagnosis: IndexMapFxHasher::from_iter([]),
                    dbs: IndexMapFxHasher::from_iter([
                        ("data".to_owned(), UdpClientDbConfig {
                            name: Name::new(self_id, "UdpIed01/data"),
                            description: "Data block of the device".to_owned(),
                            size: 1024,
                            points: [
                                PointConfig {
                                    id: 0,
                                    name: Name::new(self_id, "UdpIed01/data/Sensor1").join(),
                                    type_: PointConfigType::Int,
                                    history: PointConfigHistory::None,
                                    alarm: None,
                                    address: None,
                                    filters: None,
                                    comment: None,
                                },
                                PointConfig {
                                    id: 0,
                                    name: Name::new(self_id, "UdpIed01/data/Sensor2").join(),
                                    type_: PointConfigType::Int,
                                    history: PointConfigHistory::None,
                                    alarm: None,
                                    address: None,
                                    filters: None,
                                    comment: None,
                                },
                                PointConfig {
                                    id: 0,
                                    name: Name::new(self_id, "UdpIed01/data/Sensor3").join(),
                                    type_: PointConfigType::Real,
                                    history: PointConfigHistory::ReadWrite,
                                    alarm: None,
                                    address: None,
                                    filters: None,
                                    comment: None,
                                },
                                PointConfig {
                                    id: 0,
                                    name: Name::new(self_id, "UdpIed01/data/Sensor4").join(),
                                    type_: PointConfigType::Double,
                                    history: PointConfigHistory::Read,
                                    alarm: None,
                                    address: None,
                                    filters: None,
                                    comment: None,
                                },                                
                            ].to_vec(),
                        }),
                    ]),
                },                
            )
        ];
        for (step, conf, target) in test_data {
            let conf = serde_yaml::from_str(&conf).unwrap();
            let result = UdpClientConfig::from_yaml(self_id, &conf);
            log::debug!("{}  |  conf: {:#?}", step, result);
            assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
        }
        test_duration.exit();
    }
}
