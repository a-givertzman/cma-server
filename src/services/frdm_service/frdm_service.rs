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
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use concat_string::concat_string;
use frdm_tools::{camera::Camera, DetectingContoursCv, EdgeDetection, Eval, GeometryDefect, Initial, InitialCtx, Mad};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    kernel::state::ChangeNotify,
    services::{entity::{Name, Object, Point, PointTxId}, Service, Services, RECV_TIMEOUT},
    sync::Handles, thread_pool::Scheduler,
};
use crate::{
    domain::RwLock, services::FrdmServiceConf,
};
///
/// FRDM Service (Fiber Rope Defects Monitoring)
/// 
pub struct FrdmService {
    tx_id: usize,
    name: Name,
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
            tx_id,
            name: conf.name.clone(),
            conf: conf.clone(),
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
            .field("id", &self.dbg)
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
static SELF_ID: std::sync::LazyLock<RwLock<Dbg>> = std::sync::LazyLock::new(|| RwLock::new(Dbg::own("")));
//
// 
impl Service for FrdmService {
    //
    // 
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let tx_id = self.tx_id;
        let conf = self.conf.clone();
        let exit = self.exit.clone();
        let services = self.services.clone();
        let handles_clone = self.handles.clone();
        log::debug!("{}.run | Preparing thread...", dbg);
        *SELF_ID.write() = dbg.clone();
        let handle = self.scheduler.spawn(move || {
            let dbg = &dbg;
            let notify: ChangeNotify<_, String> = ChangeNotify::new(dbg, NotifyState::Start, vec![
                (NotifyState::Start,          Box::new(|message| log::info!("{}", message))),
                (NotifyState::Exit,           Box::new(|message| log::info!("{}", message))),
                (NotifyState::CameraError,    Box::new(|message| log::error!("{}", message))),
            ]);

            let send_to = services
                .get_link(&conf.send_to)
                .unwrap_or_else(|err| panic!("{}.run | Link {} - Not found, error: {}", dbg, conf.send_to.name(), err));
            let mut camera = Camera::new(conf.camera);
            let mut camera_stream = camera.stream();
            let defect = GeometryDefect::new(
                conf.fast_scan.geometry_defect_threshold,
                *Box::new(Mad::new()),
                EdgeDetection::new(
                    DetectingContoursCv::new(
                        Initial::new(
                            InitialCtx::new(),
                        ),
                    ),
                ),
            );
            'main: loop {
                log::debug!("{dbg}.run | Starting camera...");
                let handle = camera.read().unwrap();
                log::debug!("{dbg}.run | Starting camera - Ok");
                handles_clone.push(handle);
                log::debug!("{dbg}.run | Receiving messages from camera...");
                'camera: loop {
                    match camera_stream.recv_timeout(RECV_TIMEOUT) {
                        Ok(frame) => {
                            let result = defect.eval(frame);
                            let sql = format!("{:?}", result);
                            let point = Point::new(
                                tx_id,
                                &concat_string!(dbg, "sql"),
                                sql,
                            );
                            if let Err(err) = send_to.send(point) {
                                log::warn!("{dbg}.run | Send sql error: {:?}", err);
                            }
                            if exit.load(Ordering::Acquire) {
                                camera.exit();
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
                        break 'main;
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
                Err(err)
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
