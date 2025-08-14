use sal_sync::services::{conf::{ConfTree, ConfTreeGet}, entity::Name};
use std::{fs, time::Duration};
use crate::{infra::ApiClientConf, services::frdm_service::{rope_defect::RopeDefectConf, rope_deprecation::RopeDeprecationConf}};

///
/// Config for FrdmService format:
/// ```yaml
/// service FrdmService FrdmService1:
///     cycle: 100 ms
///     wait-started: 10 ms         # optional, next service will wait until current completely started plus specified time
///     api-client:
///         wait-started: 10 ms         # optional, next service will wait until current completely started plus specified time
///         address: "0.0.0.0:8081",
///         auth-token: "123!@#",
///         database: "cma",
///     table_settings: 'public.frdm_settings'
///     rope-defect:
///         tables:
///             defect: 'public.frdm_defect'
///             defect-image: 'public.frdm_defect_image'
///         segment: 100 mm             # Whole rope will divided by the segments for the Camera defect detection, recomended: `segment length = camera.width * 0.10..0.20`
///         segment-threshold: 5 mm     # Acceptable camera position error in relation to exact segment position 
///         camera-offset: 5.5 m                        # camera position from the begin of the rope (hook side)
///         defect-detection:
///             contours:
///                 gamma:
///                     no-param: not parameters implemented 
///                 brightness-contrast:
///                     histogram-clipping: 1     # optional histogram clipping, default = 0 %
///                 gausian:
///                     kernel-size:
///                         width: 3
///                         heidht: 3
///                     sigma-x: 0.0
///                     sigma-y: 0.0
///                 sobel:
///                     kernel-size: 3
///                     scale: 1.0
///                     delta: 0.0
///                 overlay:
///                     src1-weight: 0.5
///                     src2-weight: 0.5
///                     gamma: 0.0
///             edge-detection:
///                 threshold: 1                        # 0...255
///             fast-scan:
///                 geometry-defect-threshold: 1.2      # 1.1...1.3, absolute threshold to detect the geometry deffects
///             fine-scan:
///                 no-params: not implemented yet
///         camera Camera1:
///             fps: Max                    # Max / Min / 30.0
///             resolution: 
///                 width: 1200
///                 height: 800
///             index: 0
///             # address: 192.168.10.12:2020
///             # Mono8/10/12/16, Bayer8/10/12/16, RGB8, BGR8, YCbCr8, YCbCr411, YUV422, YUV411 | Default and fastest BayerRG8
///             # pixel-format:  Mono8
///             # pixel-format:  BayerRG8
///             # pixel-format:  QOI_Mono8
///             pixel-format:  QOI_BayerRG8
///             exposure:
///                 auto: Off                   # Off / Continuous
///                 time: 26000                   # microseconds
///             auto-packet-size: true          # StreamAutoNegotiatePacketSize
///             channel-packet-size: Max        # Maximizing packet size increases frame rate
///             resend-packet: true             # StreamPacketResendEnable
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
    /// Next service will wait until current completely started plus specified time, optional
    pub wait_started: Option<Duration>,
    // pub cycle: Option<Duration>,
    /// API configuration parametes
    pub api: ApiClientConf,
    /// Names of the database table used for storing common settings for the clients
    pub table_settings: String,
    /// The configuration parameters for the `RopeDefect`
    pub rope_defect: RopeDefectConf,
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
        log::trace!("{dbg}.new | name: {:?}", name);
        let wait_started: Option<Duration> = conf.get_duration("wait-started").ok();
        log::trace!("{}.new | wait-started: {:?}", dbg, wait_started);
        let api: ConfTree = conf.get("api-client").expect(&format!("{dbg}.new | 'api-client' - not found or wrong config"));
        let api = ApiClientConf::new(&name, api);
        log::trace!("{dbg}.new | api: {:#?}", api);
        let table_settings = conf.get("table-settings").expect(&format!("{dbg}.new | 'table-settings' - not found or wrong config"));
        log::trace!("{dbg}.new | table-settings: {:?}", table_settings);
        let rope_defect: ConfTree = conf.get("rope-defect").expect(&format!("{dbg}.new | 'rope-defect' - not found or wrong config"));
        let rope_defect = RopeDefectConf::new(&name, rope_defect, api.clone());
        log::trace!("{dbg}.new | rope-defect: {:#?}", rope_defect);
        let rope_deprecation: ConfTree = conf.get("rope-deprecation").expect(&format!("{dbg}.new | 'rope-deprecation' - not found or wrong config"));
        let rope_deprecation = RopeDeprecationConf::new(&name, rope_deprecation, api.clone());
        log::trace!("{dbg}.new | rope-deprecation: {:#?}", rope_deprecation);
        Self {
            name,
            wait_started,
            api,
            table_settings,
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
