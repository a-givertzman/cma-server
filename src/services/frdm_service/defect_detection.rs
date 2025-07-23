use std::{path::{Path, PathBuf}, sync::{atomic::{AtomicBool, Ordering}, Arc}};
use frdm_tools::{camera::Camera, AutoBrightnessAndContrast, AutoGamma, ContextRead, DetectingContoursCv, EdgeDetection, Eval, GeometryDefect, GeometryDefectCtx, Image, Initial, InitialCtx, Mad};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{conf::{ConfDistance, ConfDistanceUnit}, entity::{Cot, Name, Object, Point}, Service, Services, SubscriptionCriteria}, sync::Handles, thread_pool::Scheduler};
use crate::{domain::{constants::constants::RECV_TIMEOUT, RwLock}, services::FrdmServiceConf};

///
/// Dects defect on the frames comin from the camera
pub struct DefectDetection {
    name: Name,
    txid: usize,
    conf: FrdmServiceConf,
    starage_path: PathBuf,
    rope_pos: Arc<RwLock<Option<f64>>>,
    services: Arc<Services>,
    scheduler: Scheduler,
    handles: Arc<Handles<()>>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
//
//
impl DefectDetection {
    ///
    /// Crteates [DefectDetection] new instance
    pub fn new(parent: impl Into<String>, txid: usize, conf: FrdmServiceConf, starage_path: PathBuf, rope_pos: Arc<RwLock<Option<f64>>>, services: Arc<Services>, scheduler: Scheduler) -> Self {
        let name = Name::new(parent, "DefectDetection");
        let dbg = Dbg::new(name.parent(), name.me());
        Self {
            name,
            txid,
            conf,
            starage_path,
            rope_pos,
            services,
            scheduler,
            handles: Arc::new(Handles::new(&dbg)),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
    ///
    /// Saving camera images to local store and clenong obsoleted images
    fn save_image(dbg: &Dbg, frame: &Image, path: &Path) -> Result<(), Error> {
        let path = path.as_os_str().to_str().ok_or(Error::new(dbg, "save_image"))?;
        frame.save(path)
    }
}
//
//
impl Object for DefectDetection {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl std::fmt::Debug for DefectDetection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DefectDetection")
            .field("dbg", &self.dbg)
            .finish()
    }
}
//
// 
impl Service for DefectDetection {
    //
    // 
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let name = self.name.clone();
        let tx_id = self.txid;
        let conf = self.conf.clone();
        let services = self.services.clone();
        let exit = self.exit.clone();
        let storage_path = self.starage_path.clone();
        let table_defect = conf.tables.defect.clone();
        let table_defect_image = conf.tables.defect_image.clone();
        let send_to = services
            .get_link(&conf.send_to)
            .unwrap_or_else(|err| panic!("{}.run | Link {} - Not found, error: {}", dbg, conf.send_to.name(), err));
        let conf_service = conf.crane.rope.pos.service();
        let rope_pos_link = conf.crane.rope.pos.link();
        let rope_load_link = conf.crane.rope.load.link();
        let boom_main_angle_link = conf.crane.boom.main_angle.link();
        let boom_rotary_angle_link = conf.crane.boom.rotary_angle.link();
        let points = [
            &rope_pos_link,
            &rope_load_link,
            &boom_main_angle_link,
            &boom_rotary_angle_link,
        ].map(|point| SubscriptionCriteria::new(point, Cot::Inf));
        let rope_pos = self.rope_pos.clone();
        let rope_segment = ConfDistance::new(100.0, ConfDistanceUnit::Millimeter); 
        let handles_clone = self.handles.clone();
        log::debug!("{}.run | Preparing thread...", dbg);
        let handle = self.scheduler.spawn(move || {
            let dbg = &dbg;
            let (_, recv) = services.subscribe(&conf_service, &name.join(), &points);
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
                                            match defect.eval(frame.clone()) {
                                                Ok(ctx) => {
                                                    let geometry_defect_ctx: &GeometryDefectCtx = ctx.read();
                                                    let defects = &geometry_defect_ctx.result;
                                                    if !defects.is_empty() {
                                                        defects.iter().for_each(|defect| {
                                                            let defect_image_path = storage_path.join(format!("defect_image/{}.jpg", slice_ix));
                                                            let defect_id = match defect {
                                                                frdm_tools::GeometryDefectType::Expansion => "expansion",
                                                                frdm_tools::GeometryDefectType::Compressing => "compressing",
                                                                frdm_tools::GeometryDefectType::Hill => "hill",
                                                                frdm_tools::GeometryDefectType::Pit => "pit",
                                                            };
                                                            match defect_image_path.as_path().to_str() {
                                                                Some(image_path) => {
                                                                    let sql = format!(r"
                                                                        do $$
                                                                        begin
                                                                            insert into {table_defect} (id, defect, first, last, score)
                                                                                values ({slice_ix}, '{defect_id}', current_timestamp, current_timestamp, 1)
                                                                            on conflict (id, defect) do update 
                                                                                set (last, score) = (current_timestamp, {table_defect}.count + 1);
                                                                            insert into {table_defect_image} (frdm_defect_id, camera, path)
                                                                                values ({slice_ix}, {camera_id}, '{image_path}');
                                                                            EXCEPTION
                                                                                WHEN others then
                                                                                    rollback;
                                                                        end; $$
                                                                        language plpgsql;
                                                                    ");
                                                                    if let Err(err) = send_to.send(Point::new(tx_id, &Name::new(dbg, "sql").join(), sql)) {
                                                                        log::warn!("{dbg}.run | Send sql error: {:?}", err);
                                                                    }
                                                                    if let Err(err) = Self::save_image(&dbg, &frame, &defect_image_path) {
                                                                        log::warn!("{dbg}.run | Save image error: {:?}", err);
                                                                    }
                                                                }
                                                                None => log::warn!("{dbg}.run | Wrong image path {}", defect_image_path.display()),
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
            }
            Err(err) => {
                let err = Error::new(&self.dbg, "run").pass_with("Start failed", err.to_string());
                log::warn!("{}", err);
                return Err(err);
            }
        }
        Ok(())
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
