use frdm_tools::{camera::CameraConf, conf::FastScanConf};
use sal_sync::services::{conf::{ConfTree, ConfTreeGet}, entity::Name, LinkName};
use std::{fs, str::FromStr, time::Duration};
use crate::services::{BendingsConf, RopeConf};

///
/// Config for FrdmService format:
/// ```yaml
/// service FrdmService FrdmService1:
///     cycle: 100 ms
///     send-to: /App/ApiClient.in-queue
///     table: public.frdm
///     rope:
///         width: 35 mm        # Diameter of the rome
///         length: 3000 m      # Total working length of the rope
///         segment: 100 mm     # Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
///         pos: point real 'App/Winch.EncoderBR2'      # meters, current rope position
///         load: point real '/App/Winch.Load'          # tonn, current rope load 
///     fast-scan:
///         geometry-defect-threshold: 1.2      # 1.1..1.3, absolute threshold to detect the geometry deffects
///     fine-scan:
///         no-params: not implemented yet
///     camera:
///         fps: Max                    # Max / Min / 30.0
///         resolution: 
///             width: 1200
///             height: 800
///         index: 0
///         # address: 192.168.10.12:2020
///         # Mono8/10/12/16, Bayer8/10/12/16, RGB8, BGR8, YCbCr8, YCbCr411, YUV422, YUV411 | Default and fastest BayerRG8
///         # pixel-format:  Mono8
///         # pixel-format:  BayerRG8
///         # pixel-format:  QOI_Mono8
///         pixel-format:  QOI_BayerRG8
///         exposure:
///             auto: Off                   # Off / Continuous
///             time: 26000                   # microseconds
///         auto-packet-size: true          # StreamAutoNegotiatePacketSize
///         channel-packet-size: Max        # Maximizing packet size increases frame rate
///         resend-packet: true             # StreamPacketResendEnable
///                         ...
#[derive(Debug, PartialEq, Clone)]
pub struct FrdmServiceConf {
    pub name: Name,
    pub send_to: LinkName,
    pub table: String,
    pub cycle: Option<Duration>,
    pub rope: RopeConf,
    pub bendings: BendingsConf,
    pub fast_scan: FastScanConf,
    pub camera: CameraConf,
    // pub subscribe: ConfSubscribe,
}
//
// 
impl FrdmServiceConf {
    ///
    /// Returns [FrdmServiceConf] built from `ConfTree`:
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> FrdmServiceConf {
        let me = conf.sufix_or(conf.name().unwrap());
        let dbg = format!("FrdmServiceConf({})", me);
        log::trace!("{}.new | conf: {:?}", dbg, conf);
        let name = Name::new(parent, me);
        log::debug!("{}.new | name: {:?}", dbg, name);
        let send_to: String = conf.get("send-to").unwrap();
        let send_to = LinkName::from_str(&send_to).unwrap();
        log::debug!("{}.new | send-to: {}", dbg, send_to);
        let table: String = conf.get("table").unwrap();
        log::debug!("{}.new | table: {}", dbg, table);
        let cycle = conf.get_duration("cycle").ok();
        log::debug!("{}.new | cycle: {:?}", dbg, cycle);
        let rope = conf.get("rope").expect(&format!("{dbg}.new | 'rope' - not found or wrong configuration"));
        let rope = RopeConf::new(&name, rope);
        log::trace!("{dbg}.new | rope: {:?}", rope);
        let bendings = conf.get("bendings").expect(&format!("{dbg}.new | 'bendings' - not found or wrong configuration"));
        let bendings = BendingsConf::new(&name, bendings);
        log::debug!("{dbg}.new | bendings: {:#?}", bendings);
        let camera: ConfTree = conf.get("camera").expect(&format!("{dbg}.new | 'camera' - not found or wrong configuration"));
        let camera = CameraConf::new(&name, &camera);
        log::debug!("{dbg}.new | camera: {:#?}", camera);
        let fast_scan: ConfTree = conf.get("fast-scan").expect(&format!("{dbg}.new | 'fast-scan' - not found or wrong configuration"));
        let fast_scan = FastScanConf::new(&name, fast_scan);
        log::debug!("{dbg}.new | fast-scan: {:?}", fast_scan);
        // let subscribe = conf.get("subscribe").unwrap_or(serde_yaml::Value::Null);
        // let subscribe = ConfSubscribe::new(subscribe);
        // log::debug!("{}.new | subscribe: {:#?}", dbg, subscribe);
        FrdmServiceConf {
            name,
            send_to,
            table,
            cycle,
            rope,
            bendings,
            fast_scan,
            camera,
            // subscribe,
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub(crate) fn from_yaml(parent: impl Into<String>, value: &serde_yaml::Value) -> FrdmServiceConf {
        match value.as_mapping().unwrap().into_iter().next() {
            Some((key, value)) => {
                Self::new(parent, ConfTree::new(key.as_str().unwrap(), value.clone()))
            }
            None => {
                panic!("FrdmServiceConf.from_yaml | Format error or empty conf: {:#?}", value)
            }
        }        
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    pub fn read(parent: impl Into<String>, path: &str) -> FrdmServiceConf {
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        FrdmServiceConf::from_yaml(parent, &config)
                    }
                    Err(err) => {
                        panic!("FrdmServiceConf.read | Error in config: {:?}\n\terror: {:?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("FrdmServiceConf.read | File {} reading error: {:?}", path, err)
            }
        }
    }
}
