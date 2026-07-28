use serde::{Deserialize, Serialize};
use std::time::Duration;
///
/// ### Creates `UdpClient` config from serde_yaml::Value
/// 
/// **Example**
/// 
/// ```yaml
/// reconnect: 1000 ms                      # reconnect timeout when connection is lost
/// protocol: 'udp-raw'                     # udp-raw
/// local-address: 192.168.100.100:15180    # Local machine address
/// remote-address: 192.168.100.241:15180   # IP Address of the vibro-sensor ADC unit
/// mtu: 1500                               # Maximum Transmission Unit, default 1500
/// ```
/// 
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct UdpClientConf {
    pub description: String,
    pub cycle: Option<Duration>,
    pub reconnect: Duration,
    pub protocol: String,
    pub local_addr: String,
    pub remote_addr: String,
    /// Maximum Transmission Unit, default 1500, [Resolve IPv4 Fragmentation, MTU...](https://www.cisco.com/c/en/us/support/docs/ip/generic-routing-encapsulation-gre/25885-pmtud-ipfrag.html)
    pub mtu: usize,
}
