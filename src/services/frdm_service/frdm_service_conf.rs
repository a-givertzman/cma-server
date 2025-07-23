use frdm_tools::camera::CameraConf;
use sal_sync::services::{conf::{ConfCustomKeywd, ConfDistance, ConfTree, ConfTreeGet}, entity::Name, LinkName};
use std::{fs, str::FromStr, time::Duration};
use crate::services::{CraneConf, TablesConf};

///
/// Config for FrdmService format:
/// ```yaml
/// service FrdmService FrdmService1:
///     cycle: 100 ms
///     send-to: /App/ApiClient.in-queue
///     subscribe: MultiQueue
///     tables:
///         defect: public.frdm_defect
///         defect-image: public.frdm_defect_image
///         deprecation: public.frdm_deprecation
///     crane:
///         bendings:           # Rope bloks with diameter, inter and exit
///             # Block Diameter   inter   exit
///             - D200mm           5.0  .. 5.15 m
///             - D300mm           7.23 .. 7.30 mm
///         boom:
///             main-angle: point real 'App/MultiQueue/Load.MainBoomAngle'        # degrees, current angle of the main boom to horisontal axis
///             rotary-angle: point real 'App/MultiQueue/Load.RotaryBoomAngle'    # degrees, current angle of the rotary boom (jib) to horisontal axis
///         rope:
///             width: 35 mm        # Diameter of the rome
///             length: 3000 m      # Total working length of the rope
///             segment: 100 mm     # Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
///             pos: point real 'App/MultiQueue/Winch.EncoderBR2'      # meters, current rope position
///             load: point real 'App/MultiQueue/Winch.Load'          # tonn, current rope load
///     scan:
///         detecting-contours:
///             gamma:
///                 no-param: not parameters implemented 
///             brightness-contrast:
///                 histogram-clipping: 1     # optional histogram clipping, default = 0 %
///             gausian:
///                 kernel-size:
///                     width: 3
///                     heidht: 3
///                 sigma-x: 0.0
///                 sigma-y: 0.0
///             sobel:
///                 kernel-size: 3
///                 scale: 1.0
///                 delta: 0.0
///             overlay:
///                 src1-weight: 0.5
///                 src2-weight: 0.5
///                 gamma: 0.0
///         fast-scan:
///             geometry-defect-threshold: 1.2      # 1.1...1.3, absolute threshold to detect the geometry deffects
///         fine-scan:
///             no-params: not implemented yet
///     camera-offset: 5.5 m                        # camera position from the begin of the rope (hook side)
///     camera Camera1:
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
///```
#[derive(Debug, PartialEq, Clone)]
pub struct FrdmServiceConf {
    pub name: Name,
    pub send_to: LinkName,
    pub subscribe: String,
    pub tables: TablesConf,
    pub cycle: Option<Duration>,
    pub crane: CraneConf,
    pub scan: frdm_tools::conf::Conf,
    pub camera_offset: ConfDistance,
    pub cameras: Vec<CameraConf>,
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
        let send_to: String = conf.get("send-to").expect(&format!("{dbg}.new | 'send-to' - not found or wrong configuration"));
        let send_to = LinkName::from_str(&send_to).unwrap();
        log::debug!("{dbg}.new | send-to: {}", send_to);
        let subscribe = conf.get("subscribe").expect(&format!("{dbg}.new | 'subscribe' - not found or wrong configuration"));
        log::debug!("{dbg}.new | subscribe: {:?}", subscribe);
        let tables: TablesConf = conf.parse("tables").expect(&format!("{dbg}.new | 'tables' - not found or wrong configuration"));
        log::debug!("{dbg}.new | table defect: {}", tables.defect);
        log::debug!("{dbg}.new | table defect-image: {}", tables.defect_image);
        log::debug!("{dbg}.new | table deprecation: {}", tables.deprecation);
        let cycle = conf.get_duration("cycle").ok();
        log::debug!("{dbg}.new | cycle: {:?}", cycle);
        let crane = conf.get("crane").expect(&format!("{dbg}.new | 'crane' - not found or wrong configuration"));
        let crane = CraneConf::new(&name, crane);
        log::trace!("{dbg}.new | crane: {:?}", crane);
        let scan: ConfTree = conf.get("scan").expect(&format!("{dbg}.new | 'scan' - not found or wrong configuration"));
        let scan = frdm_tools::conf::Conf::new(&name, scan);
        log::debug!("{dbg}.new | scan: {:#?}", scan);
        let camera_offset = conf.get_distance("camera-offset").expect(&format!("{dbg}.new | 'camera-offset' - not found or wrong configuration"));
        log::debug!("{dbg}.new | camera-offset: {:#?}", camera_offset);
        let mut cameras = vec![];
        match conf.sub_nodes() {
            Some(nodes) => {
                for node in nodes {
                    if let Ok(keywd) = ConfCustomKeywd::from_str(&node.key) {
                        if keywd.keywd() == "camera" {
                            let camera = CameraConf::new(&name, &node);
                            log::debug!("{dbg}.new | camera: {:#?}", camera);
                            cameras.push(camera);
                        }
                    }
                }
            }
            None => log::warn!("{dbg}.new | No camera configurations"),
        }
        Self {
            name,
            send_to,
            subscribe,
            tables,
            cycle,
            crane,
            scan,
            camera_offset,
            cameras,
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
