use std::sync::Arc;
use sal_core::dbg::Dbg;
use sal_sync::services::entity::Name;
use crate::{domain::RwLock, services::DefectDetectionConf};

///
/// Rope representation
/// - Current rope position
/// - Segmentation of the rope
pub struct Rope {
    name: Name,
    camera_offset: f64,
    segment: f64,
    segment_threshold: f64,
    conf: DefectDetectionConf,
    pos: Arc<RwLock<Option<f64>>>,
    dbg: Dbg,
}
//
//
impl Rope {
    ///
    /// Returns [RopeSegment] new instance
    pub fn new(parent: impl Into<String>, conf: DefectDetectionConf, pos: Arc<RwLock<Option<f64>>>,) -> Self {
        let name = Name::new(parent, "Rope");
        let dbg = Dbg::new(name.parent(), name.me());
        Self {
            name,
            camera_offset: conf.camera_offset.as_mm(),
            segment: conf.scan.segment.as_mm(),
            segment_threshold: conf.scan.segment_threshold.as_mm(),
            conf,
            pos,
            dbg,
        }
    }
    ///
    /// Returns cerrent rope position if already received, else `None`
    pub fn pos(&self) -> Option<f64> {
        *self.pos.read()
    }
    ///
    /// Returns segment index from 0, 
    /// - `pos` - camera position from the begining of the rope (hook side) in meters
    /// - Returns Some(index) if camera position located at the begining of segment with acceptable error in relation to exact segment position
    pub fn segment_index(&self) -> Option<usize> {
        // Index of the current slice located under the camera (from hook)
        match *self.pos.read() {
            Some(pos) => {
                // rope pos in millimeters
                let pos = pos * 1000.0 + self.camera_offset;
                // Slices under current pos
                let slices = pos / self.segment;
                // Slice index under current pos
                let ix = slices.round();
                // Current rope pos Delta in relation to exact segment position
                let delta = (slices - ix).abs() * self.segment;
                log::debug!("{}.segment_index | pos: {:.4} ({:.2}) m, slices: {:.4},  delta: {:.4}", self.dbg, pos, pos * 1000.0, slices, delta);
                if delta < self.segment_threshold {
                    Some(ix as usize)
                } else {
                    None
                }
            }
            None => None,
        }
    }
}