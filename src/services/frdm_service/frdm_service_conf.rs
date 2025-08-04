use frdm_tools::camera::CameraConf;
use sal_sync::services::{conf::{ConfCustomKeywd, ConfTree, ConfTreeGet}, entity::Name};
use std::{fs, str::FromStr};
use crate::{infra::ApiClientConf, services::frdm_service::{rope_defect::RopeDefectConf, rope_deprecation::RopeDeprecationConf}};

///
/// Config for FrdmService format:
/// ```yaml
/// service FrdmService FrdmService1:
///     cycle: 100 ms
///     api:
///         address: "0.0.0.0:8080",
///         auth_token: "123!@#",
///         database: "cma",
///     rope-defect:
///         tables:
///             defect: 'public.frdm_defect'
///             defect-image: 'public.frdm_defect_image'
///         segment: 100 mm             # Whole rope will divided by the segments for the Camera defect detection, recomended: `segment length = camera.width * 0.10..0.20`
///         segment-threshold: 5 mm     # Acceptable camera position error in relation to exact segment position 
///         camera-offset: 5.5 m                        # camera position from the begin of the rope (hook side)
///         defect-detection:
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
///             fast-scan:
///                 geometry-defect-threshold: 1.2      # 1.1...1.3, absolute threshold to detect the geometry deffects
///             fine-scan:
///                 no-params: not implemented yet
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
/// 
///     rope-deprecation:
///         table: 'public.frdm_deprecation'
///         subscribe: MultiQueue                                          # Service name, to subscribe for rope positin and crane angles event's
///         crane:
///             bendings:           # Rope bloks with diameter, inter and exit
///                 # Block Diameter   inter   exit
///                 - D200mm           5.0  .. 5.15 m
///                 - D300mm           7.23 .. 7.30 mm
///             boom:
///                 main-len: 5.3 m                                         # length of the main boom
///                 main-angle: point real 'App/MultiQueue/Load.MainBoomAngle'        # degrees, current angle of the main boom to horisontal axis
///                 rotary-len: 2.1 m                                       # length of the rotary boom
///                 rotary-angle: point real 'App/MultiQueue/Load.RotaryBoomAngle'    # degrees, current angle of the rotary boom (jib) to horisontal axis
///             rope:
///                 width: 35 mm        # Diameter of the rome
///                 length: 3000 m      # Total working length of the rope
///                 segment: 100 mm     # Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
///                 pos: point real 'App/MultiQueue/Winch.EncoderBR2'      # meters, current rope position
///                 load: point real 'App/MultiQueue/Winch.Load'           # tonn, current rope load
///```
#[derive(Debug, PartialEq, Clone)]
pub struct FrdmServiceConf {
    pub name: Name,
    // pub cycle: Option<Duration>,
    /// API configuration parametes
    pub api: ApiClientConf,
    /// The configuration parameters for the `RopeDefect`
    pub rope_defect: Vec<RopeDefectConf>,
    /// The Config parameters for `RopeDeprecation`
    pub rope_deprecation: RopeDeprecationConf,
}
//
// 
impl FrdmServiceConf {
    ///
    /// Returns [FrdmServiceConf] built from `ConfTree`:
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let me = conf.sufix_or(conf.name().unwrap());
        let dbg = format!("FrdmServiceConf '{}'", me);
        log::trace!("{dbg}.new | conf: {:?}", conf);
        let name = Name::new(parent, me);
        log::debug!("{dbg}.new | name: {:?}", name);
        let api: ApiClientConf = conf.parse("api").expect(&format!("{dbg}.new | 'api' - not found or wrong configuration"));
        log::debug!("{dbg}.new | api: {:#?}", api);
        let rope_deprecation: ConfTree = conf.get("rope-deprecation").expect(&format!("{dbg}.new | 'rope-deprecation' - not found or wrong configuration"));
        let rope_deprecation = RopeDeprecationConf::new(&name, rope_deprecation, api.clone());
        log::trace!("{dbg}.new | rope-deprecation: {:#?}", rope_deprecation);
        let mut camera_id = 0;
        let mut rope_defect = vec![];
        match conf.sub_nodes() {
            Some(nodes) => {
                let conf: ConfTree = conf.get("rope-defect").expect(&format!("{dbg}.new | 'rope-defect' - not found or wrong configuration"));
                for node in nodes {
                    if let Ok(keywd) = ConfCustomKeywd::from_str(&node.key) {
                        if keywd.name() == "camera" {
                            let camera = CameraConf::new(&name, &node);
                            log::trace!("{dbg}.new | camera: {:#?}", camera);
                            let rope_defect_conf = RopeDefectConf::new(&name, conf.clone(), api.clone(), camera_id, camera);
                            log::trace!("{dbg}.new | rope-defect: {:#?}", rope_defect_conf);
                            rope_defect.push(rope_defect_conf);
                            camera_id +=1;
                        }
                    }
                }
            }
            None => log::warn!("{dbg}.new | No camera configurations"),
        }
        Self {
            name,
            api,
            rope_defect,
            rope_deprecation,
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
