use std::{str::FromStr, time::Duration};
use frdm_tools::camera::CameraConf;
use sal_core::dbg::Dbg;
use sal_sync::services::{conf::{ConfCustomKeywd, ConfDistance, ConfDistanceUnit, ConfTree, ConfTreeGet}, entity::Name};
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
///     camera-offset: 5.5 m        # camera position from the begin of the rope (hook side)
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
    /// ### Next service will wait until current completely started plus specified time, optional
    pub wait_started: Option<Duration>,
    /// ### API configuration parametes
    pub api: ApiClientConf,
    /// ### Names of the database tables used for storing defects and it's images
    pub tables: TablesConf,
    /// ### Rope segmetn length.
    /// Whole rope will divided by the segments for the Camera defect detection, recomended: `segment length = camera.width * 0.10..0.20`
    pub segment: ConfDistance,
    /// ### Acceptable camera position error in relation to exact segment position
    ///
    /// Default: 5% of `segment`
    pub segment_threshold: ConfDistance,
    /// ### Rope segment length for defect registration (DB). Typically 1m.
    ///
    /// | segment | register_segment |
    /// | ---     | ---              |
    /// |   24.. 500 mm  |  1.0 m    |
    /// |  500.. 700 mm  |  2.0 m    |
    /// |  700..1500 mm  |  3.0 m    |
    /// | 1500..5000 mm  | 10.0 m    |
    register_segment: ConfDistance,
    /// ### Camera position from the begin of the rope (hook side)
    pub camera_offset: ConfDistance,
    /// ### Configuration parameters for binarization and defect detection algorithms
    pub defect_detection: frdm_tools::conf::Conf,
    pub cameras: Vec<(CameraId, CameraConf)>,
}
//
impl RopeDefectConf {
    ///
    /// Returns [RopeDefectConf] built from `ConfTree`
    pub fn new(
        parent: impl Into<String>,
        conf: ConfTree,
        api: ApiClientConf,
    ) -> Self {
        let parent = parent.into();
        let me = "RopeDefectConf";
        let dbg = Dbg::new(&parent, me);
        let name = Name::new(parent, me);
        log::trace!("{dbg}.new | name: {:?}", name);
        let wait_started: Option<Duration> = conf.get_duration("wait-started").ok();
        log::trace!("{}.new | wait-started: {:?}", dbg, wait_started);
        let tables = conf.parse("tables").expect(&format!("{dbg}.new | 'tables' - not found or wrong configuration"));
        log::trace!("{dbg}.new | tables: {:?}", tables);
        let segment = conf.get_distance("segment").expect(&format!("{dbg}.new | 'segment' - not found or wrong configuration"));
        log::trace!("{dbg}.new | segment: {:?}", segment);
        let segment_threshold = conf.get_distance("segment-threshold").expect(&format!("{dbg}.new | 'segment-threshold' - not found or wrong configuration"));
        log::trace!("{dbg}.new | segment-threshold: {:?}", segment_threshold);
        let camera_offset = conf.get_distance("camera-offset").expect(&format!("{dbg}.new | 'camera-offset' - not found or wrong configuration"));
        log::trace!("{dbg}.new | camera-offset: {:?}", camera_offset);
        let defect_detection: ConfTree = conf.get("defect-detection").expect(&format!("{dbg}.new | 'defect-detection' - not found or wrong configuration"));
        let defect_detection = frdm_tools::conf::Conf::new(&name, defect_detection);
        log::trace!("{dbg}.new | defect-detection: {:#?}", defect_detection);
        let cameras: Vec<(CameraId, CameraConf)> = conf.nodes()
            .filter(|node| ConfCustomKeywd::from_str(&node.key).map_or(false, |keywd| keywd.name() == "camera"))
            .enumerate()
            .map(|(id, node)| {
                let camera = CameraConf::new(&name, &node);
                log::trace!("{dbg}.new | camera: {:#?}", camera);
                (CameraId(id), camera)
            })
            .collect();
        if cameras.is_empty() {
            log::warn!("{dbg}.new | No camera configurations");
        }
        Self {
            name,
            wait_started,
            api,
            tables,
            segment,
            segment_threshold,
            register_segment: match segment.as_mm() {
                  24.0 ..  500.0 => ConfDistance::new(1.0, ConfDistanceUnit::Meter),
                 500.0 ..  700.0 => ConfDistance::new(2.0, ConfDistanceUnit::Meter),
                 700.0 .. 1500.0 => ConfDistance::new(3.0, ConfDistanceUnit::Meter),
                1500.0 .. 5000.0 => ConfDistance::new(10.0, ConfDistanceUnit::Meter),
                _ => panic!("{dbg}.new | 'segment' length {:?} is unexpected or invalid. Expected 24..500 mm.", segment),
            },
            camera_offset,
            defect_detection,
            cameras,
        }
    }
    /// Возвращает расчетное количество сегментов каната для регистрации в БД
    /// с учетом общей длины каната и размера одного сегмента для регистрации
    /// - `rope_length` - Общая длина каната
    pub fn db_slices(&self, rope_length: ConfDistance) -> usize {
        (rope_length.as_m() / self.register_segment.as_m()).round() as usize
    }
    /// Переводит индекс (номер) сегмента каната из расчетного размера (поле `segment`)
    /// в размер для регистрации в БД (поле `register_segment`).
    pub fn scale_slice_to_db(&self, slice: usize) -> usize {
        let ratio = self.register_segment.as_mm() / self.segment.as_mm();
        ((slice as f64 + 0.5) / ratio).floor() as usize
    }
}
///
/// Camera unique identifier to be used in the sql database and folder name
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraId(pub usize);
// Implement the Default trait to provide a default value
impl Default for CameraId {
    fn default() -> Self {
        CameraId(0) // Default value for the wrapped usize
    }
}

// Implement Deref to allow immutable dereferencing to usize
impl std::ops::Deref for CameraId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0 // Dereference to the inner usize
    }
}

// Implement DerefMut to allow mutable dereferencing to usize
impl std::ops::DerefMut for CameraId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0 // Mutably dereference to the inner usize
    }
}
//
//
impl Default for RopeDefectConf {
    fn default() -> Self {
        Self {
            name: Name::new("", "RopeDefectConf"),
            wait_started: Default::default(),
            api: Default::default(),
            tables: Default::default(),
            segment: Default::default(),
            segment_threshold: Default::default(),
            register_segment: Default::default(),
            camera_offset: Default::default(),
            defect_detection: Default::default(),
            cameras: Default::default(),
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
    use testing::stuff::max_test_duration::TestDuration;

    ///
    /// Returns [RopeDefectConf] with the specified segment and register_segment lengths
    fn conf(segment_mm: f64, register_segment_m: f64) -> RopeDefectConf {
        RopeDefectConf {
            segment: ConfDistance::new(segment_mm, ConfDistanceUnit::Millimeter),
            register_segment: ConfDistance::new(register_segment_m, ConfDistanceUnit::Meter),
            ..Default::default()
        }
    }
    ///
    /// Testing [RopeDefectConf].scale_slice_to_db()
    /// - `segment: 100 mm`, `register_segment: 1.0 m`, `ratio = 10`
    #[test]
    fn test_scale_slice_to_db() {
        DebugSession::new().filter(LogLevel::Trace).init().unwrap();
        let dbg = Dbg::own("RopeDefectConf-test");
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
        test_duration.run().unwrap();
        let conf = conf(100.0, 1.0);
        let test_data = [
            //       detection slice ix   covered rope, mm    db slice
            (01,    0,                    0..100,             0),
            (02,    4,                    400..500,           0),
            (03,    5,                    500..600,           0),     // большая часть сегмента в первом метре
            (04,    9,                    900..1000,          0),
            (05,    10,                   1000..1100,         1),
            (06,    14,                   1400..1500,         1),
            (07,    15,                   1500..1600,         1),     // большая часть сегмента во втором метре
            (08,    19,                   1900..2000,         1),
            (09,    20,                   2000..2100,         2),
            // Последний detection slice каната 3000 m (3000 m / 100 mm - 1 = 29999)
            // не должен выйти за пределы количества db slices (2999)
            (10,    29999,                2999900..3000000,   2999),
        ];
        for (step, slice, _covered, target) in test_data {
            let time = Instant::now();
            let result = conf.scale_slice_to_db(slice);
            assert!(result == target, "{dbg} | step {step} \nresult: {result}\ntarget: {target}");
            log::debug!("{dbg} | step {step}  elapsed: {:?}", time.elapsed());
        }
    }
    ///
    /// Testing [RopeDefectConf].scale_slice_to_db() with the non-integer ratio
    /// - `segment: 300 mm`, `register_segment: 1.0 m`, `ratio = 10/3 ≈ 3.333`
    #[test]
    fn test_scale_slice_to_db_non_integer_ratio() {
        DebugSession::new().filter(LogLevel::Trace).init().unwrap();
        let dbg = Dbg::own("RopeDefectConf-test");
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
        test_duration.run().unwrap();
        let conf = conf(300.0, 1.0);
        let test_data = [
            //       detection slice ix   covered rope, mm    db slice
            (01,    0,                    0..300,             0),
            (02,    2,                    600..900,           0),     // целиком в первом метре
            (03,    3,                    900..1200,          1),     // 100 мм в первом, 200 мм во втором
            (04,    6,                    1800..2100,         1),     // 200 мм во втором, 100 мм в третьем
            (05,    7,                    2100..2400,         2),     // целиком в третьем метре
        ];
        for (step, slice, _covered, target) in test_data {
            let time = Instant::now();
            let result = conf.scale_slice_to_db(slice);
            assert!(result == target, "{dbg} | step {step} \nresult: {result}\ntarget: {target}");
            log::debug!("{dbg} | step {step}  elapsed: {:?}", time.elapsed());
        }
    }
    ///
    /// Testing [RopeDefectConf].db_slices()
    #[test]
    fn test_db_slices() {
        DebugSession::new().filter(LogLevel::Trace).init().unwrap();
        let dbg = Dbg::own("RopeDefectConf-test");
        log::debug!("\n{}", dbg);
        let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
        test_duration.run().unwrap();
        let test_data = [
            //       rope length, m   register_segment, m   db slices
            (01,    3000.0,            1.0,                  3000),
            (02,    3000.0,            10.0,                 300),
            (03,    3050.0,            1.0,                  3050),
            (04,    2999.4,            1.0,                  2999),
            (05,    100.5,             1.0,                  101),   // round: половина округляется вверх
        ];
        for (step, rope_length, register_segment, target) in test_data {
            let time = Instant::now();
            let conf = conf(100.0, register_segment);
            let result = conf.db_slices(ConfDistance::new(rope_length, ConfDistanceUnit::Meter));
            assert!(result == target, "{dbg} | step {step} \nresult: {result}\ntarget: {target}");
            log::debug!("{dbg} | step {step}  elapsed: {:?}", time.elapsed());
        }
    }
}
