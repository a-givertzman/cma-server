use sal_core::dbg::Dbg;
use sal_sync::services::entity::Name;
use crate::services::DefectDetectionConf;

///
/// Segmentation of the rope
pub struct RopeSegment {
    name: Name,
    conf: DefectDetectionConf,
    // segment: ConfDistance,
    dbg: Dbg,
}
//
//
impl RopeSegment {
    ///
    /// Returns [RopeSegment] new instance
    pub fn new(parent: impl Into<String>, conf: DefectDetectionConf) -> Self {
        let name = Name::new(parent, "RopeSegment");
        let dbg = Dbg::new(name.parent(), name.me());
        Self {
            name,
            conf,
            // segment,
            dbg,
        }
    }
    ///
    /// Returns segment index from 0, 
    /// - `pos` - camera position from the begining of the rope (hook side) in meters
    /// - Returns Some(index) if camera position located at the begining of segment with acceptable error in relation to exact segment position
    pub fn segment_index(&self, pos: f64) -> Option<usize> {
        // Index of the current slice located under the camera (from hook)
        let slice_ix = pos / self.conf.scan.segment.as_m();
        if slice_ix.fract() < self.conf.scan.segment_threshold.as_m() {
            Some(slice_ix.trunc() as usize)
        } else {
            None
        }
    }
}