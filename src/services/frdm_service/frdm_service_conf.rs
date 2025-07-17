use frdm_tools::{camera::CameraConf, conf::FastScanConf};
use sal_sync::services::{conf::{ConfTree, ConfTreeGet}, entity::Name, LinkName};
use std::{fs, str::FromStr, time::Duration};
use crate::services::{RopeConf, TablesConf};

///
/// Config for FrdmService format:
/// ```yaml
/// service FrdmService FrdmService1:
///     cycle: 100 ms
///     send-to: /App/ApiClient.in-queue
///     tables:
///         defect: public.frdm_defect
///         defect-image: public.frdm_defect_image
///         deprecation: public.frdm_deprecation
///     crane:
///         main-boom-abgle: point real 'App/Load.MainBoomAngle'        # degrees, current angle of the main boom to vertical axis
///         rotary-boom-abgle: point real 'App/Load.RotaryBoomAngle'    # degrees, current angle of the rotary boom (jib) to boom axis
///     rope:
///         width: 35 mm        # Diameter of the rome
///         length: 3000 m      # Total working length of the rope
///         segment: 100 mm     # Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
///         bendings:           # Rope bloks with diameter, inter and exit
///               Block Diameter   inter   exit
///             - D200mm           5.0  .. 5.15 m
///             - D300mm           7.23 .. 7.30 mm
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
    pub tables: TablesConf,
    pub cycle: Option<Duration>,
    pub rope: RopeConf,
    pub fast_scan: FastScanConf,
    pub camera: CameraConf,
    // pub subscribe: ConfSubscribe,
}
//
// 
impl FrdmServiceConf {
    ///
    /// Returns [FrdmServiceConf] built from `ConfTree`:
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let me = conf.sufix_or(conf.name().unwrap());
        let dbg = format!("FrdmServiceConf({})", me);
        log::trace!("{dbg}.new | conf: {:?}", conf);
        let name = Name::new(parent, me);
        log::debug!("{dbg}.new | name: {:?}", name);
        let send_to: String = conf.get("send-to").unwrap();
        let send_to = LinkName::from_str(&send_to).unwrap();
        log::debug!("{dbg}.new | send-to: {}", send_to);
        let tables: TablesConf = conf.parse("tables").unwrap();
        log::debug!("{dbg}.new | table defect: {}", tables.defect);
        log::debug!("{dbg}.new | table defect-image: {}", tables.defect_image);
        log::debug!("{dbg}.new | table deprecation: {}", tables.deprecation);
        let cycle = conf.get_duration("cycle").ok();
        log::debug!("{dbg}.new | cycle: {:?}", cycle);
        let rope = conf.get("rope").expect(&format!("{dbg}.new | 'rope' - not found or wrong configuration"));
        let rope = RopeConf::new(&name, rope);
        log::trace!("{dbg}.new | rope: {:?}", rope);
        let camera: ConfTree = conf.get("camera").expect(&format!("{dbg}.new | 'camera' - not found or wrong configuration"));
        let camera = CameraConf::new(&name, &camera);
        log::debug!("{dbg}.new | camera: {:#?}", camera);
        let fast_scan: ConfTree = conf.get("fast-scan").expect(&format!("{dbg}.new | 'fast-scan' - not found or wrong configuration"));
        let fast_scan = FastScanConf::new(&name, fast_scan);
        log::debug!("{dbg}.new | fast-scan: {:?}", fast_scan);
        let fine_scan: ConfTree = conf.get("fast-scan").expect(&format!("{dbg}.new | 'fine-scan' - not found or wrong configuration"));
        let fine_scan = FastScanConf::new(&name, fine_scan);
        log::debug!("{dbg}.new | fine-scan: {:?}", fine_scan);
        Self {
            name,
            send_to,
            tables,
            cycle,
            rope,
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
