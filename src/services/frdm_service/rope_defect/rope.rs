use std::sync::Arc;
use sal_core::dbg::Dbg;
use sal_sync::services::conf::ConfDistance;
use crate::{domain::RwLock, services::frdm_service::RopeDeprecation};

///
/// Rope representation
/// - Current rope position
/// - Segmentation of the rope
pub struct Rope {
    /// Camera position from the begin of the rope (hook side), mm
    camera_offset: f64,
    segment: f64,
    segment_threshold: f64,
    pos: Arc<RopeDeprecation>,
    dbg: Dbg,
}
//
//
impl Rope {
    ///
    /// Returns [RopeSegment] new instance
    /// - `camera_offset` - Camera position from the begin of the rope (hook side)
    /// - `segment` - Whole rope will divided by the segments for the Camera defect detection, recomended: `segment length = camera.width * 0.10..0.20`
    /// - `segment_threshold` - Acceptable camera position error in relation to exact segment position
    /// - `pos` - Position of the rope, meters
    pub fn new(parent: impl Into<String>, camera_offset: ConfDistance, segment: ConfDistance, segment_threshold: ConfDistance, pos: Arc<RopeDeprecation>) -> Self {
        let dbg = Dbg::new(parent, "Rope");
        Self {
            camera_offset: camera_offset.as_mm(),
            segment: segment.as_mm(),
            segment_threshold: segment_threshold.as_mm(),
            pos,
            dbg,
        }
    }
    ///
    /// Returns cerrent rope position (mm) if already received, else `None`
    pub fn pos(&self) -> Option<f64> {
        self.pos.rope_pos().map(|pos| pos * 1000.0)
    }
    ///
    /// Returns cerrent rope position (mm) under camera if already received, else `None`
    pub fn pos_at_camera(&self) -> Option<f64> {
        self.pos.rope_pos().map(|pos| pos * 1000.0 + self.camera_offset)
    }
    ///
    /// Returns segment index from 0, 
    /// - `pos` - camera position from the begining of the rope (hook side) in meters
    /// - Returns Some(index) if camera position located at the begining of segment with acceptable error in relation to exact segment position
    pub fn segment_index(&self) -> Option<usize> {
        // Index of the current slice located under the camera (from hook)
        match self.pos.rope_pos() {
            Some(pos) => {
                // rope pos in millimeters
                let pos = pos * 1000.0 + self.camera_offset;
                // Slices under current pos
                let slices = pos / self.segment;
                // Slice index under current pos
                let ix = slices.round();
                // Current rope pos Delta in relation to exact segment position
                let delta = (slices - ix).abs() * self.segment;
                if delta <= self.segment_threshold {
                    log::trace!("{}.segment_index | pos: {:.2}mm ({:.4}m), slices: {:.4},  delta: {:.2}mm - Use slice {ix}", self.dbg, pos, pos * 0.001, slices, delta);
                    Some(ix as usize)
                } else {
                    log::trace!("{}.segment_index | pos: {:.2}mm ({:.4}m), slices: {:.4},  delta: {:.2}mm - Skip", self.dbg, pos, pos * 0.001, slices, delta);
                    None
                }
            }
            None => None,
        }
    }
}