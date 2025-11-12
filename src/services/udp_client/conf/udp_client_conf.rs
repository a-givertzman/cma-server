use sal_sync::{collections::FxIndexMap, services::{ConfSubscribe, LinkName, conf::{ConfTree, ConfTreeGet, DiagKeywd}, entity::{Name, PointConf}, task::functions::{FnConfKeywd, FnConfKindName}}};
use std::{fs, str::FromStr, time::Duration};
///
/// Creates config from serde_yaml::Value
/// 
/// Example
/// 
/// ```yaml
/// service UdpClient UdpIed01:            # device will be executed in the independent thread, must have unique name
///    description: 'UDP-IED-01.01'
///    subscribe: Multiqueue
///    send-to: MultiQueue.in-queue
///    cycle: 1 ms                         # operating cycle time of the device
///    reconnect: 1000 ms                  # reconnect timeout when connection is lost
///    protocol: 'udp-raw'
///    local-address: 192.168.100.100:15180
///    remote-address: 192.168.100.241:15180
///    mtu: 1500                           # Maximum Transmission Unit, default 1500
///    diagnosis:                          # internal diagnosis
///        point Status:                   # Ok(0) / Invalid(10)
///            type: 'Int'
///            # history: r
///        point Connection:               # Ok(0) / Invalid(10)
///            type: 'Int'
///            # history: r
///    point Sensor1:                  # Device input signal config
///        type: 'Int'
///        id: 0                        # the number of input 0..8 (0 - first input channel)
///    point Sensor2:                  # Device input signal config
///        type: 'Int'
///        id: 1                        # the number of input 0..8 (0 - first input channel)
///```
/// 
#[derive(Debug, PartialEq, Clone)]
pub struct UdpClientConf {
    pub name: Name,
    pub description: String,
    pub subscribe: ConfSubscribe,
    /// Destination `Service`
    pub send_to: LinkName,
    pub cycle: Option<Duration>,
    pub reconnect: Duration,
    pub protocol: String,
    pub local_addr: String,
    pub remote_addr: String,
    /// Maximum Transmission Unit, default 1500, [Resolve IPv4 Fragmentation, MTU...](https://www.cisco.com/c/en/us/support/docs/ip/generic-routing-encapsulation-gre/25885-pmtud-ipfrag.html)
    pub mtu: usize,
    pub diagnosis: FxIndexMap<DiagKeywd, PointConf>,
    pub points: Vec<PointConf>,
}
//
// 
impl UdpClientConf {
    ///
    /// Creates new instance of the [UdpClientConfig]:
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let me = conf.sufix_or(conf.name().unwrap());
        let dbg = format!("UdpClientConfig({})", me);
        log::trace!("{}.new | conf: {:?}", dbg, conf);
        let name = Name::new(parent, me);
        log::trace!("{}.new | name: {:?}", dbg, name);
        let description = conf.get("description").unwrap_or_default();
        log::trace!("{}.new | description: {:?}", dbg, description);
        let subscribe = ConfSubscribe::new(conf.get("subscribe").unwrap_or(serde_yaml::Value::Null));
        log::trace!("{}.new | subscribe: {:?}", dbg, subscribe);
        let send_to: String = conf.get("send-to").unwrap();
        let send_to = LinkName::from_str(&send_to).unwrap();
        log::trace!("{}.new | send-to: {}", dbg, send_to);
        let cycle = conf.get_duration("cycle").ok();
        log::trace!("{}.new | cycle: {:?}", dbg, cycle);
        let reconnect = conf.get_duration("reconnect").map_or(Duration::from_secs(3), |reconnect| reconnect);
        log::trace!("{}.new | reconnect: {:?}", dbg, reconnect);
        let protocol = conf.get("protocol").unwrap();
        log::trace!("{}.new | protocol: {:?}", dbg, protocol);
        let local_address = conf.get("local-address").unwrap();
        log::trace!("{}.new | local-address: {:?}", dbg, local_address);
        let remote_address = conf.get("remote-address").unwrap();
        log::trace!("{}.new | remote-address: {:?}", dbg, remote_address);
        let mtu = conf.get("mtu");
        log::trace!("{}.new | mtu: {:?}", dbg, mtu);
        let diagnosis = conf.get_diagnosis(&name);
        log::trace!("{}.new | diagnosis: {:#?}", dbg, diagnosis);
        let points = conf.keys(&[] as &[&str; 0]).iter().filter_map(|key| {
            let keyword = FnConfKeywd::from_str(key).unwrap();
            if keyword.kind() == FnConfKindName::Point {
                let point: ConfTree = conf.get(key).expect(&format!("{dbg}.new | '{key}' - not found or wrong configuration"));
                log::trace!("{dbg}.new | Point '{}'", keyword.data());
                log::trace!("{dbg}.new | Point '{}'   |   conf: {:?}", keyword.data(), point);
                Some(PointConf::new(&name, &point))
            } else {
                log::warn!("{dbg}.new | Device input conf (point Sensor...) expected, but found {:?}", keyword);
                None
            }
        }).collect();
        UdpClientConf {
            name,
            description,
            subscribe,
            send_to,
            cycle,
            reconnect,
            protocol,
            local_addr: local_address,
            remote_addr: remote_address,
            mtu: mtu.unwrap_or(serde_yaml::Value::Null).as_u64().unwrap_or(1500) as usize,
            diagnosis,
            points,
        }
    }
    ///
    /// Returns config build from serde_yaml::Value
    pub(crate) fn from_yaml(parent: impl Into<String>, value: &serde_yaml::Value) -> UdpClientConf {
        match value.as_mapping().unwrap().into_iter().next() {
            Some((key, value)) => {
                Self::new(parent, ConfTree::new(key.as_str().unwrap(), value.clone()))
            }
            None => {
                panic!("UdpClientConfig.from_yaml | Format error or empty conf: {:#?}", value)
            }
        }
    }
    ///
    /// Returns config build from path
    #[allow(dead_code)]
    pub fn read(parent: impl Into<String>, path: &str) -> UdpClientConf {
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        UdpClientConf::from_yaml(parent, &config)
                    }
                    Err(err) => {
                        panic!("UdpClientConfig.read | Error in config: {:?}\n\terror: {:#?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("UdpClientConfig.read | File {} reading error: {:#?}", path, err)
            }
        }
    }
    ///
    /// Returns list of configurations of the defined points
    pub fn points(&self) -> Vec<PointConf> {
        self.points
            .iter()
            .cloned()
            .chain(self.diagnosis.values().cloned())
            .collect()
    }
}
