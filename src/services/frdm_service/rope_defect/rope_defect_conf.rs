use std::time::Duration;

use frdm_tools::camera::CameraConf;
use sal_core::dbg::Dbg;
use sal_sync::services::{conf::{ConfDistance, ConfTree, ConfTreeGet}, entity::Name};
use crate::{infra::ApiClientConf, services::frdm_service::rope_defect::tables_conf::TablesConf};
///
/// ## The configuration parameters for the `RopeDefect`
/// 
/// ### Conf example
/// ```yaml
/// rope-defect:
///     wait-started: 10 ms         # optional, next service will wait until current completely started plus specified time
///     tables:
///         defect: 'public.frdm_defect'
///         defect-image: 'public.frdm_defect_image'
///     segment: 100 mm             # Whole rope will divided by the segments for the Camera defect detection, recomended: `segment length = camera.width * 0.10..0.20`
///     segment-threshold: 5 mm     # Acceptable camera position error in relation to exact segment position 
///     camera-offset: 5.5 m                        # camera position from the begin of the rope (hook side)
///     defect-detection:
///         gamma:
///             no-param: not parameters implemented 
///         brightness-contrast:
///             histogram-clipping: 1     # optional histogram clipping, default = 0 %
///         gausian:
///             kernel-size:
///                 width: 3
///                 heidht: 3
///             sigma-x: 0.0
///             sigma-y: 0.0
///         sobel:
///             kernel-size: 3
///             scale: 1.0
///             delta: 0.0
///         overlay:
///             src1-weight: 0.5
///             src2-weight: 0.5
///             gamma: 0.0
///         fast-scan:
///             geometry-defect-threshold: 1.2      # 1.1...1.3, absolute threshold to detect the geometry deffects
///         fine-scan:
///             no-params: not implemented yet
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct RopeDefectConf {
    pub name: Name,
    /// Next service will wait until current completely started plus specified time, optional
    pub wait_started: Option<Duration>,
    /// API configuration parametes
    pub api: ApiClientConf,
    /// Names of the database tables used for storing defects and it's images 
    pub tables: TablesConf,
    /// Rope segmetn length.
    /// Whole rope will divided by the segments for the Camera defect detection, recomended: `segment length = camera.width * 0.10..0.20`
    pub segment: ConfDistance,
    /// Acceptable camera position error in relation to exact segment position
    /// 
    /// Default: 5% of `segment`
    pub segment_threshold: ConfDistance,
    /// Camera position from the begin of the rope (hook side)
    pub camera_offset: ConfDistance,
    /// Configuration parameters for binarization and defect detection algorithms
    pub defect_detection: frdm_tools::conf::Conf,
    /// Id of the exact camera to used inthe sql database and folder name 
    pub camera_id: usize,
    pub camera: CameraConf,
}
//
// 
impl RopeDefectConf {
    ///
    /// Returns [RopeDefectConf] built from `ConfTree`:
    pub fn new(
        parent: impl Into<String>,
        conf: ConfTree,
        api: ApiClientConf,
        camera_id: usize,
        camera: CameraConf,
    ) -> Self {
        let parent = parent.into();
        let me = "RopeDefectConf";
        let dbg = Dbg::new(&parent, me);
        let name = Name::new(parent, me);
        log::debug!("{dbg}.new | name: {:?}", name);
        let wait_started: Option<Duration> = conf.get_duration("wait-started").ok();
        log::debug!("{}.new | wait-started: {:?}", dbg, wait_started);
        let tables = conf.parse("tables").expect(&format!("{dbg}.new | 'tables' - not found or wrong configuration"));
        log::debug!("{dbg}.new | tables: {:?}", tables);
        let segment = conf.get_distance("segment").expect(&format!("{dbg}.new | 'segment' - not found or wrong configuration"));
        log::debug!("{dbg}.new | segment: {:?}", segment);
        let segment_threshold = conf.get_distance("segment-threshold").expect(&format!("{dbg}.new | 'segment-threshold' - not found or wrong configuration"));
        log::debug!("{dbg}.new | segment-threshold: {:?}", segment_threshold);
        let camera_offset = conf.get_distance("camera-offset").expect(&format!("{dbg}.new | 'camera-offset' - not found or wrong configuration"));
        log::debug!("{dbg}.new | camera-offset: {:?}", camera_offset);
        let defect_detection: ConfTree = conf.get("defect-detection").expect(&format!("{dbg}.new | 'defect-detection' - not found or wrong configuration"));
        let defect_detection = frdm_tools::conf::Conf::new(&name, defect_detection);
        log::trace!("{dbg}.new | defect-detection: {:#?}", defect_detection);
        Self {
            name,
            wait_started,
            api,
            tables,
            segment,
            segment_threshold,
            camera_offset,
            defect_detection,
            camera_id,
            camera,
        }
    }
}
