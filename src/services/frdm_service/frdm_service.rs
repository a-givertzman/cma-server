//!
//! # FRDM (Fiber Rope Defects Monitoring)
//! 
//! - Communication with Camera 
//! - Receives current rope position
//! - Scanning the rope for defects
//! - Calculates Rope Depreciation Rate
//! 
//! ## Basic configuration parameters:
//! 
//! ```yaml
//! service FrdmService FrdmService1:
//!     parameter: value    # meaning
//!     parameter: value    # meaning
//! ```
//! 
use std::{path::Path, sync::{atomic::{AtomicBool, Ordering}, Arc}};
use frdm_tools::{camera::Camera, AutoBrightnessAndContrast, AutoGamma, ContextRead, DetectingContoursCv, EdgeDetection, Eval, GeometryDefect, GeometryDefectCtx, Initial, InitialCtx, Mad};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    kernel::state::ChangeNotify,
    services::{conf::{ConfDistance, ConfDistanceUnit}, entity::{Cot, Name, Object, Point, PointTxId}, Service, Services, SubscriptionCriteria, RECV_TIMEOUT},
    sync::Handles, thread_pool::Scheduler,
};
use crate::{domain::RwLock, services::{FrdmServiceConf, RopeDeprecationRate, RopeDeprecationRateConf}};
///
/// FRDM Service (Fiber Rope Defects Monitoring)
/// 
pub struct FrdmService {
    name: Name,
    tx_id: usize,
    conf: FrdmServiceConf,
    services: Arc<Services>,
    scheduler: Scheduler,
    handles: Arc<Handles<()>>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
//
//
impl FrdmService {
    //
    /// Crteates new instance of the FrdmService 
    pub fn new(conf: FrdmServiceConf, services: Arc<Services>, scheduler: Scheduler) -> Self {
        let tx_id = PointTxId::from_str(&conf.name.join());
        let dbg = Dbg::new(conf.name.parent(), conf.name.me());
        Self {
            name: conf.name.clone(),
            tx_id,
            conf,
            services,
            scheduler,
            handles: Arc::new(Handles::new(&dbg)),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
}
//
//
impl Object for FrdmService {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl std::fmt::Debug for FrdmService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FrdmService")
            .field("dbg", &self.dbg)
            .finish()
    }
}
///
/// Used for logging
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum NotifyState {
    Start,
    Exit,
    CameraError,
}
//
//
// static SELF_ID: std::sync::LazyLock<RwLock<Dbg>> = std::sync::LazyLock::new(|| RwLock::new(Dbg::own("")));
//
// 
impl Service for FrdmService {
    //
    // 
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let name = self.name.clone();
        let tx_id = self.tx_id;
        let conf = self.conf.clone();
        let table_defect = conf.tables.defect.clone();
        let table_defect_image = conf.tables.defect_image.clone();
        let exit = self.exit.clone();
        let services = self.services.clone();
        let scheduler = self.scheduler.clone();
        let handles_clone = self.handles.clone();
        let path = Path::new("./files").join(
            name.join()
                .chars()
                .enumerate()
                .filter(|(ix, ch)| !((*ix == 0) & (*ch == '/')))
                .map(|(_, ch)| ch)
                .collect::<String>()
        );
        let (_, rope_pos_recv) = services.subscribe(&conf.crane.rope.pos.service(), &name.join(), &[SubscriptionCriteria::new(conf.crane.rope.pos.link(), Cot::Inf)]);
        let rope_segment = ConfDistance::new(100.0, ConfDistanceUnit::Millimeter); 
        let rope_pos = Arc::new(RwLock::new(None::<f64>));
        let rope_pos_clone = rope_pos.clone();
        let rope_deprecation = RopeDeprecationRate::new(
            &dbg,
            RopeDeprecationRateConf::new(&name, conf.crane, conf.send_to.clone(), conf.tables.deprecation),
            move |rope_pos: f64| {
                *rope_pos_clone.write() = Some(rope_pos);
            },
            services.clone(),
            scheduler,
        );
        let _ = rope_deprecation.run()?;
        log::debug!("{}.run | Preparing thread...", dbg);
        let handle = self.scheduler.spawn(move || {
            let dbg = &dbg;
            let notify: ChangeNotify<_, String> = ChangeNotify::new(dbg, NotifyState::Start, vec![
                (NotifyState::Start,          Box::new(|message| log::info!("{message}"))),
                (NotifyState::Exit,           Box::new(|message| log::info!("{message}"))),
                (NotifyState::CameraError,    Box::new(|message| log::error!("{message}"))),
            ]);

            let send_to = services
                .get_link(&conf.send_to)
                .unwrap_or_else(|err| panic!("{}.run | Link {} - Not found, error: {}", dbg, conf.send_to.name(), err));
            let mut camera = Camera::new(conf.cameras.first().unwrap().to_owned());
            let camera_stream = camera.stream();
            let defect = GeometryDefect::new(
                conf.scan.fast_scan.geometry_defect_threshold,
                *Box::new(Mad::new()),
                EdgeDetection::new(
                    DetectingContoursCv::new(
                        conf.scan.detecting_contours.clone(),
                        AutoBrightnessAndContrast::new(
                            conf.scan.detecting_contours.brightness_contrast.histogram_clipping,
                            AutoGamma::new(
                                Initial::new(
                                    InitialCtx::new(),
                                ),
                            ),
                        ),
                    ),
                ),
            );
            'main: loop {
                log::debug!("{dbg}.run | Starting camera...");
                match camera.read() {
                    Ok(handle) => {
                        log::debug!("{dbg}.run | Starting camera - Ok");
                        handles_clone.push(handle);
                        let camera_id = 0;
                        log::debug!("{dbg}.run | Receiving frames from camera...");
                        'camera: loop {
                            match camera_stream.recv_timeout(RECV_TIMEOUT) {
                                Ok(frame) => {
                                    match *rope_pos.read() {
                                        Some(rope_pos) => {
                                            // Position of the rope under the camera
                                            let pos = rope_pos + conf.camera_offset.as_m();
                                            // Index of the current slice located under the camera (from hook)
                                            let slice_ix = (pos / rope_segment.as_m()).trunc();
                                            match defect.eval(frame) {
                                                Ok(ctx) => {
                                                    let geometry_defect_ctx: &GeometryDefectCtx = ctx.read();
                                                    let defects = geometry_defect_ctx.result;
                                                    if !defects.is_empty() {
                                                        defects.iter().for_each(|defect| {
                                                            let defect_image_path = path.join(format!("defect_image/{}.jpeg", slice_ix));
                                                            let defect_id = match defect {
                                                                frdm_tools::GeometryDefectType::Expansion => "expansion",
                                                                frdm_tools::GeometryDefectType::Compressing => "compressing",
                                                                frdm_tools::GeometryDefectType::Hill => "hill",
                                                                frdm_tools::GeometryDefectType::Pit => "pit",
                                                            };
                                                            match defect_image_path.into_os_string().into_string() {
                                                                Ok(image_path) => {
                                                                    let sql = format!(r"
                                                                        begin;
                                                                            insert into {table_defect} (id, defect, first, last, count)
                                                                                values ({slice_ix}, {defect_id}, current_timestamp, current_timestamp, 1)
                                                                            on conflict (id, defect) do update 
                                                                                set (last, count, acknowledged, deleted) = (current_timestamp, count + 1);
                                                                            insert into {table_defect_image} (frdm_defect_id, camera_id, path)
                                                                                values ({slice_ix}, {camera_id}, '{image_path}')
                                                                        commit;
                                                                    ");
                                                                    if let Err(err) = send_to.send(Point::new(tx_id, &Name::new(dbg, "sql").join(), sql)) {
                                                                        log::warn!("{dbg}.run | Send sql error: {:?}", err);
                                                                    }
                                                                }
                                                                Err(err) => log::warn!("{dbg}.run | Image path {} error: {:?}", defect_image_path.display(), err),
                                                            };
                                                        });
                                                    }
                                                }
                                                Err(err) => log::debug!("{dbg}.run | {}, Defect detection error: {:?}", camera.name(), err),
                                            }
                                        }
                                        None => {
                                            // rope position not received yet
                                        }
                                    }
                                    if exit.load(Ordering::Acquire) {
                                        camera.exit();
                                        rope_deprecation.exit();
                                        break 'main;
                                    }
                                }
                                Err(err) => {
                                    match err {
                                        crate::domain::RecvTimeoutError::Timeout => {}
                                        _ => {
                                            camera.exit();
                                            break 'camera;
                                        }
                                    }
                                }
                            }
                            if exit.load(Ordering::Acquire) {
                                camera.exit();
                                rope_deprecation.exit();
                                break 'main;
                            }
                        }
                    }
                    Err(err) => {
                        log::info!("{dbg}.run | Camera error: {:?}", err);
                    }
                }
            }
            log::info!("{dbg}.run | Exit");
            Ok(())
        });
        match handle {
            Ok(handle) => {
                log::info!("{}.run | Starting - ok", self.dbg);
                self.handles.push(handle);
                Ok(())
            }
            Err(err) => {
                let err = Error::new(&self.dbg, "run").pass_with("Start failed", err.to_string());
                log::warn!("{}", err);
                return Err(err)
            }
        }
    }
    //
    //
    fn wait(&self) -> Result<(), Error> {
        self.handles.wait()
    }
    //
    //
    fn is_finished(&self) -> bool {
        self.handles.is_finished()
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::Release);
    }    
}
