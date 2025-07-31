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
                let pos = pos + self.conf.camera_offset.as_m();
                let slice_ix = pos / self.conf.scan.segment.as_m();
                if slice_ix.fract() < self.conf.scan.segment_threshold.as_m() {
                    Some(slice_ix.trunc() as usize)
                } else {
                    None
                }
            }
            None => None,
        }
    }
}