use sal_sync::services::conf::ConfDuration;
use serde::{Deserialize, Deserializer, Serialize};
use std::{str::FromStr, time::Duration};
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
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UdpClientConf {
    pub description: Option<String>,
    pub reconnect: ConfDuration,
    pub protocol: String,
    #[serde(alias = "local-address")]
    pub local_addr: String,
    #[serde(alias = "remote-address")]
    pub remote_addr: String,
    /// Maximum Transmission Unit, default 1500, [Resolve IPv4 Fragmentation, MTU...](https://www.cisco.com/c/en/us/support/docs/ip/generic-routing-encapsulation-gre/25885-pmtud-ipfrag.html)
    pub mtu: usize,
}
