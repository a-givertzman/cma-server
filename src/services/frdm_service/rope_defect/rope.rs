use sal_core::dbg::Dbg;
use sal_sync::services::conf::ConfDistance;

///
/// Rope representation
/// - Current rope position
/// - Segmentation of the rope
pub struct Rope {
    /// Camera position from the begin of the rope (hook side), mm
    camera_offset: f64,
    segment: f64,
    segment_threshold: f64,
    #[allow(unused)]
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
    pub fn new(parent: impl Into<String>, camera_offset: ConfDistance, segment: ConfDistance, segment_threshold: ConfDistance) -> Self {
        let dbg = Dbg::new(parent, "Rope");
        Self {
            camera_offset: camera_offset.as_mm(),
            segment: segment.as_mm(),
            segment_threshold: segment_threshold.as_mm(),
            dbg,
        }
    }
    ///
    /// Returns 
    ///
    /// Returns cerrent rope position (mm) under the camera
    /// - `pos` - Rope pos, mm (from zero parking position)
    pub fn pos_at_cam(&self, pos: usize) -> usize {
        (pos as f64 + self.camera_offset).round() as usize
    }
    ///
    /// Returns Rope segment index under the camera, from 0
    /// - `pos` - Rope pos from hook, mm (from zero parking position)
    /// 
    /// Returns `Some(ix)` if camera position aligned to the beginning of segment with acceptable accuracy,
    /// otherwise returns `None`
    pub fn segment_index(&self, pos: usize) -> Option<usize> {
        // rope pos in millimeters
        let pos = pos as f64 + self.camera_offset;
        // Slices under current pos
        let slices = pos / self.segment;
        // Slice index under current pos
        let ix = slices.round();
        // Current rope pos Delta in relation to exact segment position
        let delta = (slices - ix).abs() * self.segment;
        if delta <= self.segment_threshold {
            log::debug!("{}.detection | Rope position at camera: {:.2?} mm ({:.3?} m), index {ix}, delta {:.2} mm", self.dbg, pos, pos * 0.001, delta);
            Some(ix as usize)
        } else {
            // log::trace!("{}.segment_index | pos: {:.2}mm ({:.4}m), slices: {:.4},  delta: {:.2}mm - Skip", self.dbg, pos, pos * 0.001, slices, delta);
            None
        }
    }
}