use frdm_tools::camera::CameraConf;
use sal_core::dbg::Dbg;
use sal_sync::services::{conf::ConfDistance, entity::Name, LinkName};
use crate::services::TablesConf;
///
/// ## The configuration parameters for the `DefectDetection`
/// 
/// ### Example:
/// ```yaml
/// rope:
///     width: 35 mm        # Diameter of the rome
///     length: 3000 m      # Total working length of the rope
///     segment: 100 mm     # Rope segmetn length. Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
///     pos: point real 'App/Winch.EncoderBR2'      # meters, current rope position
///     load: point real '/App/Winch.Load'          # tonn, current rope load 
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct DefectDetectionConf {
    pub name: Name,
    /// ApiClient input queue name, to communicate with the database
    pub send_to: LinkName,
    /// Names of the database tables used for storing defects and it's images 
    pub tables: TablesConf,
    /// id of the exact camera to used inthe sql database and folder name 
    pub camera_id: usize,
    /// Camera position from the begin of the rope (hook side)
    pub camera_offset: ConfDistance,
    pub camera: CameraConf,
    /// Configuration parameters for defect detection algorithms
    pub scan: frdm_tools::conf::Conf,
}
//
// 
impl DefectDetectionConf {
    ///
    /// Returns [DefectDetectionConf] built from `ConfTree`:
    pub fn new(
        parent: impl Into<String>,
        send_to: LinkName,
        tables: TablesConf,
        camera_id: usize,
        camera_offset: ConfDistance,
        camera: CameraConf,
        scan: frdm_tools::conf::Conf
    ) -> Self {
        let parent = parent.into();
        let me = "DefectDetectionConf";
        let dbg = Dbg::new(&parent, me);
        let name = Name::new(parent, me);
        log::debug!("{dbg}.new | name: {:?}", name);
        Self {
            name,
            send_to,
            tables,
            camera_id,
            camera_offset,
            camera,
            scan,
        }
    }
}
