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
    /// - `pos` - Rope pos, mm (from zero parking position)
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
///
/// Basic Tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    use debugging::session::{DebugSession, LogLevel};
    use sal_sync::services::conf::ConfDistanceUnit;
    use testing::stuff::max_test_duration::TestDuration;

    ///
    /// Testing [Rope].segment_index()
    #[test]
    fn test_segment_index() {
        DebugSession::new().filter(LogLevel::Trace).init().unwrap();
        let dbg = Dbg::own("RopePos-test");
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
        test_duration.run().unwrap();
        // Camera position from the begin of the rope (hook side), meters
        let camera_offset = 3.5;
        let rope = Rope::new(
            &dbg,
            ConfDistance::new(camera_offset, ConfDistanceUnit::Meter),
            ConfDistance::new(100.0, ConfDistanceUnit::Millimeter),
            ConfDistance::new(5.0, ConfDistanceUnit::Millimeter),
        );
        let test_data = [
            //       rope-pos(m)
            (01,     0.000f64,       Some(35)),
            (02,     0.001,          Some(35)),
            (03,     0.004,          Some(35)),
            (04,     0.005,          Some(35)),
            (05,     0.006,          None),
            (06,     0.050,          None),
            (06,     0.094,          None),
            (10,     0.095,          Some(36)),
            (11,     0.096,          Some(36)),
            (12,     0.100,          Some(36)),
            (13,     0.101,          Some(36)),
            (14,     0.104,          Some(36)),
            (15,     0.105,          Some(36)),
            (16,     0.106,          None),
            (16,     0.194,          None),
            (17,     0.195,          Some(37)),
            (18,     0.196,          Some(37)),
            (19,     0.200,          Some(37)),
        ];
        for (step, pos, segment_index) in test_data {
            let time = Instant::now();
            let result = rope.segment_index((pos * 1000.0).round() as usize);
            let target = segment_index;
            assert!(result == target, "{dbg} | step {step} \nresult: {:?}\ntarget: {:?}", result, target);
            log::debug!("{dbg} | step {step}  elapsed: {:?}", time.elapsed());
        }
        test_duration.exit();
    }
}
