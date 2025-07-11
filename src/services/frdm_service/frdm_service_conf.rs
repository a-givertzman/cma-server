use frdm_tools::camera::CameraConf;
use indexmap::IndexMap;
use sal_sync::services::{conf::{ConfDistance, ConfTree, ConfTreeGet}, entity::{Name, PointConfig}, task::functions::{FnConfKind, FnConfig}, ConfSubscribe};
use std::{fs, time::Duration};
///
/// Config for FrdmService format:
/// ```yaml
/// service FrdmService FrdmService1:
///     cycle: 100 ms
///     rope-width: 35 mm
///     rope-length: point real 'App/Winch.EncoderBR2'      # in meters
///     rope-load: point real '/App/Winch.Load'             # in tonn
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
    pub(crate) name: Name,
    pub(crate) cycle: Option<Duration>,
    pub(crate) rope_width: ConfDistance,
    pub(crate) rope_length: PointConfig,
    pub(crate) camera: CameraConf,
    pub(crate) subscribe: ConfSubscribe,
}
//
// 
impl FrdmServiceConf {
    ///
    /// Returns [FrdmServiceConf] built from `ConfTree`:
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> FrdmServiceConf {
        let mut vars = vec![];
        let me = conf.sufix_or(conf.name().unwrap());
        let dbg = format!("FrdmServiceConf({})", me);
        log::trace!("{}.new | conf: {:?}", dbg, conf);
        let name = Name::new(parent, me);
        log::debug!("{}.new | name: {:?}", dbg, name);
        let cycle = conf.get_duration("cycle").ok();
        log::debug!("{}.new | cycle: {:?}", dbg, cycle);

        let rope_width = conf.get_distance("rope-width").unwrap();
        log::debug!("{dbg}.new | rope-width: {:?}", rope_width);

        let (_, rope_length) = conf.get_by_keywd("rope-length", "point").unwrap();
        let rope_length = PointConfig::new(name, &rope_length);
        log::debug!("{dbg}.new | rope_length: {:?}", rope_length);

        let camera: ConfTree = conf.get("camera").unwrap();
        let camera = CameraConf::new(name, &camera);
        log::debug!("{dbg}.new | camera: {:?}", camera);


        let subscribe = conf.get("subscribe").unwrap_or(serde_yaml::Value::Null);
        let subscribe = ConfSubscribe::new(subscribe);
        log::debug!("{}.new | subscribe: {:#?}", dbg, subscribe);

        FrdmServiceConf {
            name,
            cycle,
            rope_width,
            rope_length,
            camera,
            subscribe,
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
    ///
    /// Returns list of configurations of the defined points
    pub fn points(&self) -> Vec<PointConfig> {
        self.nodes.iter().fold(vec![], |mut points, (_node_name,node_conf)| {
            points.extend(node_conf.points());
            points
        })
    }
}
