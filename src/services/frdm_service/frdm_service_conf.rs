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
///             rope:
///                 width: 35 mm            # Diameter of the rome
///                 length: 3000 m          # Total working length of the rope
///                 winch-length: 2985 m    # Length of the rope on the winch drum in the parking position, when rope pos is zero
///                 segment: 100 mm         # Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
///                 pos: point real 'Winch.EncoderBR2'      # meters, current rope position (длина каната размотанного с барабана считая от парковочного)
///                 load: point real 'Winch.Load'           # tonn, current rope load
///             booms:
///                 - Main-Boom:
///                     l1: 0.0 mm                  # Растояние от продольной оси стрелы до точки A (оси ее поворота), константа
///                     l2: 0.0 mm                  # Растояние по продольной оси стрелы от точки D (корня стрелы) до точки A (оси ее поворота), константа
///                     l3: 0.0 mm                  # Расстояние от точки A (ось поворота) стрелы до продольной оси предыдущей стрелы (до ГСК для первой срелы), константа
///                     l4: 10330.0 mm              # Расстояние от точки A (ось поворота) стрелы до перпендикуляра к продольной оси через точку G предыдущей стрелы (до ГСК для первой срелы), константа
///                     len: 11200.0 mm                                         # length of the boom
///                     angle: point real 'Load.MainBoomAngle'   # degrees, current angle of the boom (relative axis)
///                 - Rotary-Boom:
///                     l1: 0.0 mm                  # Растояние от продольной оси стрелы до точки A (оси ее поворота), константа
///                     l2: 0.0 mm                  # Растояние по продольной оси стрелы от точки D (корня стрелы) до точки A (оси ее поворота), константа
///                     l3: 0.0 mm                  # Расстояние от точки A (ось поворота) стрелы до продольной оси предыдущей стрелы (до ГСК для первой срелы), константа
///                     l4: 0.0 mm                  # Расстояние от точки A (ось поворота) стрелы до перпендикуляра к продольной оси через точку G предыдущей стрелы (до ГСК для первой срелы), константа
///                     len: 7984.1 mm                                          # length of the rotary boom
///                     angle: point real 'Load.RotaryBoomAngle' # degrees, current angle of the boom (relative axis)
///             blocks:
///                 - 1:
///                     lf: 1830.0 mm,  710.0 mm    # Растояние (x, y) от **конца** стрелы до оси блока, мм
///                     d: 844.0 mm                 # Диаметры блоков, мм
///                     schemes: TopTop             # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
///                     bind: Fixed                 # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
///                 - 2:
///                     lf: 308.0 mm, 1090.0 mm     # Растояние (x, y) от **конца** стрелы до оси блока, мм
///                     d: 816.0 mm                 # Диаметры блоков, мм
///                     schemes: TopTop             # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
///                     bind: Boom 0                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
///                 - 3:
///                     lf: -6550.0 mm, 1730.0 mm   # Растояние (x, y) от **конца** стрелы до оси блока, мм
///                     d: 816.0 mm                 # Диаметры блоков, мм
///                     schemes: TopTop             # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
///                     bind: Boom 1                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
///                 - 4:
///                     lf: -1121.0 mm, 973.0 mm    # Растояние (x, y) от **конца** стрелы до оси блока, мм
///                     d: 816.0 mm                 # Диаметры блоков, мм
///                     schemes: TopBottom          # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
///                     bind: Boom 2                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
///                 - 5:
///                     lf: 267.0 mm, 860.0 mm      # Растояние (x, y) от **конца** стрелы до оси блока, мм
///                     d: 816.0 mm                 # Диаметры блоков, мм
///                     schemes: BottomTop          # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
///                     bind: Boom 3                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
///                 - 6:
///                     lf: 136.0 mm, -35.0 mm      # Растояние (x, y) от **конца** стрелы до оси блока, мм
///                     d: 816.0 mm                 # Диаметры блоков, мм
///                     schemes: TopTop             # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
///                     bind: Boom 4                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
///                 - 7:
///                     lf: 0.0 mm, 0.0 mm          # Растояние (x, y) от **конца** стрелы до оси блока, мм
///                     d: 0.0 mm                   # Диаметры блоков, мм
///                     schemes: TopTop             # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
///                     bind: Hook                  # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
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
