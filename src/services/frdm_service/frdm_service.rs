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
use frdm_tools::{camera::{Camera, CameraConf}, conf::{FastScanConf, FineScanConf}, DetectingContoursCv, EdgeDetection, Eval, GeometryDefect, Initial, InitialCtx, Mad};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    kernel::state::ChangeNotify,
    services::{entity::{Name, Object, PointTxId}, Service, ServiceCycle, Services}, sync::Handles, thread_pool::Scheduler,
};
use crate::{
    conf::udp_client_config::udp_client_config::UdpClientConfig,
    core_::RwLock,
};
///
/// FRDM Service (Fiber Rope Defects Monitoring)
/// 
pub struct FrdmService {
    tx_id: usize,
    name: Name,
    conf: UdpClientConfig,
    services: Arc<Services>,
    scheduler: Scheduler,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
//
//
impl FrdmService {
    //
    /// Crteates new instance of the FrdmService 
    pub fn new(conf: UdpClientConfig, services: Arc<Services>, scheduler: Scheduler) -> Self {
        let tx_id = PointTxId::from_str(&conf.name.join());
        let dbg = Dbg::new(conf.name.parent(), conf.name.me());
        Self {
            tx_id,
            name: conf.name.clone(),
            conf: conf.clone(),
            services,
            scheduler,
            handles: Handles::new(&dbg),
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
        log::debug!("{}.run | Preparing thread...", dbg);
        *SELF_ID.write() = dbg.clone();
        let handle = self.scheduler.spawn(move || {
            let dbg = &dbg;
            let notify: ChangeNotify<_, String> = ChangeNotify::new(dbg, NotifyState::Start, vec![
                (NotifyState::Start,          Box::new(|message| log::info!("{}", message))),
                (NotifyState::Exit,           Box::new(|message| log::info!("{}", message))),
                (NotifyState::CameraError,    Box::new(|message| log::error!("{}", message))),
            ]);

            let send = services
                .get_link(&conf.send_to)
                .unwrap_or_else(|err| panic!("{}.run | Link {} - Not found, error: {}", dbg, conf.send_to.name(), err));
            let conf = serde_yaml::from_str(r"
                service Camera Camera1:
                    fps: Max                    # Max / Min / 30.0
                    resolution: 
                        width: 1200
                        height: 800
                    index: 0
                    # address: 192.168.10.12:2020
                    # Mono8/10/12/16, Bayer8/10/12/16, RGB8, BGR8, YCbCr8, YCbCr411, YUV422, YUV411 | Default and fastest BayerRG8
                    # pixel-format:  Mono8
                    # pixel-format:  BayerRG8
                    # pixel-format:  QOI_Mono8
                    pixel-format:  QOI_BayerRG8
                    exposure:
                        auto: Off                   # Off / Continuous
                        time: 26000                   # microseconds
                    auto-packet-size: true          # StreamAutoNegotiatePacketSize
                    channel-packet-size: Max        # Maximizing packet size increases frame rate
                    resend-packet: true             # StreamPacketResendEnable
            ").unwrap();
            let conf = CameraConf::from_yaml(dbg, &conf);
            let mut camera = Camera::new(conf);
            let camera_stream = camera.stream();
            let handle = camera.read().unwrap();
            let conf = frdm_tools::conf::Conf {
                fast_scan: FastScanConf {
                    geometry_defect_threshold: frdm_tools::Threshold::min(),
                },
                fine_scan: FineScanConf {},
            };
            
            for frame in camera_stream {
                let result = GeometryDefect::new(
                    conf.fast_scan.geometry_defect_threshold,
                    *Box::new(Mad::new()),
                    EdgeDetection::new(
                        DetectingContoursCv::new(
                            Initial::new(
                                InitialCtx::new(frame),
                            ),
                        ),
                    ),
                )
                .eval(());
                _ = result;
                if exit.load(Ordering::SeqCst) {
                    break;
                }
                if exit.load(Ordering::SeqCst) {
                    break;
                }
            }
            // 'main: loop {
            // }
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
        self.exit.store(true, Ordering::SeqCst);
    }    
}
