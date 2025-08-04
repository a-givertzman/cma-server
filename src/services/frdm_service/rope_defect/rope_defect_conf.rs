use frdm_tools::camera::CameraConf;
use sal_core::dbg::Dbg;
use sal_sync::services::{conf::{ConfDistance, ConfTree, ConfTreeGet}, entity::Name};
use crate::{infra::ApiClientConf, services::frdm_service::rope_defect::tables_conf::TablesConf};
///
/// ## The configuration parameters for the `RopeDefect`
#[derive(Debug, Clone, PartialEq)]
pub struct RopeDefectConf {
    pub name: Name,
    /// API configuration parametes
    pub api: ApiClientConf,
    /// Names of the database tables used for storing defects and it's images 
    pub tables: TablesConf,
    /// Whole rope will divided by the segments for the Camera defect detection, recomended: `segment length = camera.width * 0.10..0.20`
    pub segment: ConfDistance,
    /// Acceptable camera position error in relation to exact segment position 
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
    /// Returns [DefectDetectionConf] built from `ConfTree`:
    pub fn new(
        parent: impl Into<String>,
        conf: ConfTree,
        api: ApiClientConf,
        camera_id: usize,
        camera: CameraConf,
    ) -> Self {
        let parent = parent.into();
        let me = "DefectDetectionConf";
        let dbg = Dbg::new(&parent, me);
        let name = Name::new(parent, me);
        log::debug!("{dbg}.new | name: {:?}", name);
        let tables = conf.parse("tables").expect(&format!("{dbg}.new | 'tables' - not found or wrong configuration"));
        log::debug!("{dbg}.new | tables: {:?}", tables);
        let segment = conf.get_distance("segment").expect(&format!("{dbg}.new | 'segment' - not found or wrong configuration"));
        log::debug!("{dbg}.new | segment: {:#?}", segment);
        let segment_threshold = conf.get_distance("segment-threshold").expect(&format!("{dbg}.new | 'segment-threshold' - not found or wrong configuration"));
        log::debug!("{dbg}.new | segment-threshold: {:#?}", segment_threshold);
        let camera_offset = conf.get_distance("camera-offset").expect(&format!("{dbg}.new | 'camera-offset' - not found or wrong configuration"));
        log::debug!("{dbg}.new | camera-offset: {:?}", camera_offset);
        let defect_detection: ConfTree = conf.get("defect_detection").expect(&format!("{dbg}.new | 'defect_detection' - not found or wrong configuration"));
        let defect_detection = frdm_tools::conf::Conf::new(&name, defect_detection);
        log::trace!("{dbg}.new | defect_detection: {:#?}", defect_detection);
        Self {
            name,
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
