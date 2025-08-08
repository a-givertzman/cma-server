use std::{fs, path::{Path, PathBuf}, sync::{atomic::{AtomicBool, Ordering}, Arc}, time::{Duration, Instant}};
use chrono::Datelike;
use frdm_tools::{camera::Camera, AutoBrightnessAndContrast, AutoGamma, ContextRead, DetectingContoursCv, EdgeDetection, Eval, GeometryDefect, GeometryDefectCtx, GeometryDefectType, Image, Initial, InitialCtx, Mad};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{entity::{Name, Object}, Service, ServiceWaiting}, sync::Handles, thread_pool::Scheduler};
use crate::{domain::constants::constants::RECV_TIMEOUT, infra::ApiClient, services::frdm_service::rope_defect::{Rope, RopeDefectConf}};

///
/// Dects defect on the frames coming from the camera
pub struct RopeDefect {
    name: Name,
    conf: RopeDefectConf,
    camera_id: usize,
    starage_path: PathBuf,
    rope: Arc<Rope>,
    api_client: Arc<ApiClient>,
    scheduler: Scheduler,
    handles: Arc<Handles<()>>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
//
//
impl RopeDefect {
    ///
    /// Crteates [RopeDefect] new instance
    pub fn new(
        parent: impl Into<String>,
        conf: RopeDefectConf,
        camera_id: usize,
        starage_path: impl AsRef<Path>,
        rope: Arc<Rope>,
        api_client: Arc<ApiClient>,
        scheduler: Scheduler,
    ) -> Self {
        let name = Name::new(parent, "RopeDefect");
        let dbg = Dbg::new(name.parent(), name.me());
        Self {
            name,
            conf,
            camera_id,
            starage_path: starage_path.as_ref().join("rope-defects"),
            rope,
            api_client,
            scheduler,
            handles: Arc::new(Handles::new(&dbg)),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
    ///
    /// Saving camera images to local store
    /// and clenong obsoleted images
    fn save_image(dbg: &Dbg, api_client: &ApiClient, defect_id: &str, camera_id: usize, frame: &Image, img_path: &str) -> Result<(), Error> {
        // let error = Error::new(dbg, "save_image");
        let sql = format!(r"select * from clean_frdm_defect_image({defect_id}, {camera_id})");
        api_client.fetch(&sql).then(
            |reply| match reply {
                Ok(reply) => {
                    for entry in reply {
                        match entry.get("path") {
                            Some(path) => {
                                match serde_json::from_value(path.to_owned()) {
                                    Ok(path) => {
                                        let path: String = path;
                                        if let Err(err) = fs::remove_file(&path) {
                                            log::warn!("{dbg}.save_image | Delete image '{path}' error: {:?}", err);
                                        }
                                    }
                                    Err(err) => {
                                        log::warn!("{dbg}.save_image | Deserialize path '{path}' error: {:?}", err);
                                    }
                                }
                            }
                            None => log::warn!("{dbg}.save_image | Field 'path' not found in sql reply: {:#?}", entry),
                        }
                    }
                }
                Err(err) => log::warn!("{dbg}.save_image | Sql error: {:?}", err),
            },
            |err| log::warn!("{dbg}.save_image | Error: {:?}", err),
        );
        frame.save(img_path)
    }
    ///
    /// Defect detection
    fn detection(
        dbg: &Dbg,
        frame: Image,
        defect: &GeometryDefect,
        rope: &Rope,
        camera_name: &str,
        camera_id: usize,
        table_defect: &str,
        table_defect_image: &str,
        storage_path: &PathBuf,
        api_client: &ApiClient,
        prev_index: Option<usize>,
    ) -> Option<usize> {
        // Position of the rope under the camera, meter
        // let rope_pos = rope.pos();
        // log::warn!("{dbg}.detection | Rope at: {:.2?} mm ({:.3?} m)...", rope_pos.map(|pos| pos).unwrap_or(-0.0), rope_pos.map(|pos| pos * 0.001).unwrap_or(-0.0));
        let time = Instant::now();
        let rope_pos = rope.pos_at_camera();
        // log::warn!("{dbg}.detection | Rope under camera at: {:.2?} mm ({:.3?} m)...", rope_pos.map(|pos| pos).unwrap_or(-0.0), rope_pos.map(|pos| pos * 0.001).unwrap_or(-0.0));
        match rope.segment_index() {
            Some(slice_ix) => {
                if Some(slice_ix) != prev_index {
                    log::debug!("{dbg}.detection | Analizing rope at: {:.2?} mm ({:.3?} m) index {slice_ix}, prev_ix {:?}...", rope_pos.map(|pos| pos).unwrap_or(-0.0), rope_pos.map(|pos| pos * 0.001).unwrap_or(-0.0), prev_index);
                    match defect.eval(frame.clone()) {
                        Ok(ctx) => {
                            let geometry_defect_ctx: &GeometryDefectCtx = ctx.read();
                            let defects = &geometry_defect_ctx.result;
                            if !defects.is_empty() {
                                defects.iter().for_each(|defect| {
                                    let defect_id = match defect {
                                        GeometryDefectType::Expansion => "expansion",
                                        GeometryDefectType::Compressing => "compressing",
                                        GeometryDefectType::Hill => "hill",
                                        GeometryDefectType::Pit => "pit",
                                    };
                                    let now = chrono::Utc::now();
                                    let img_name = format!("{:0>2}-{:0>2}-{:0>4}_{defect_id}.jpg", now.day(), now.month(), now.year());
                                    let img_path = storage_path.join(format!("{slice_ix}")).join(img_name);
                                    match img_path.as_path().to_str() {
                                        Some(img_path) => {
                                            let sql = format!(r"
                                                do $$
                                                begin
                                                    insert into {table_defect} (id, defect, first, last, score)
                                                        values ({slice_ix}, '{defect_id}', current_timestamp, current_timestamp, 1)
                                                    on conflict (id, defect) do update 
                                                        set (last, score) = (current_timestamp, {table_defect}.count + 1);
                                                    insert into {table_defect_image} (frdm_defect_id, camera, path)
                                                        values ({slice_ix}, {camera_id}, '{img_path}');
                                                    exception
                                                        when others then
                                                            rollback;
                                                end; $$
                                                language plpgsql;
                                            ");
                                            api_client.fetch(sql).then(
                                                |_| {
                                                    if let Err(err) = Self::save_image(
                                                        &dbg,
                                                        &api_client,
                                                        defect_id,
                                                        camera_id,
                                                        &frame,
                                                        &img_path,
                                                    ) {
                                                        log::warn!("{dbg}.run | Save image error: {:?}", err);
                                                    }
                                                },
                                                |err| {
                                                    log::warn!("{dbg}.run | Send sql error: {:?}", err);
                                                },
                                            );
                                        }
                                        None => log::warn!("{dbg}.run | Wrong image path {}", img_path.display()),
                                    };
                                });
                            } else {
                                log::warn!("{dbg}.run | Slice {slice_ix} - No defect detected");
                            }
                        }
                        Err(err) => log::debug!("{dbg}.run | {}, Defect detection error: {:?}", camera_name, err),
                    }
                    log::debug!("{dbg}.detection | Elapsed: {:?}", time.elapsed());
                    Some(slice_ix)
                } else {
                    log::trace!("{dbg}.detection | Elapsed: {:?}", time.elapsed());
                    prev_index
                }
            }
            None => {
                log::trace!("{dbg}.detection | Rope pos not under segment or not received");
                log::trace!("{dbg}.detection | Elapsed: {:?}", time.elapsed());
                prev_index
            }
        }
    }
}
//
//
impl Object for RopeDefect {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl std::fmt::Debug for RopeDefect {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RopeDefect")
            .field("dbg", &self.dbg)
            .finish()
    }
}
//
// 
impl Service for RopeDefect {
    //
    // 
    fn run(&self) -> Result<(), Error> {
        let camera_conf = self.conf.cameras[self.camera_id].1.clone();
        log::info!("{}.run | Starting {}[{}]...", self.dbg, camera_conf.name, self.camera_id);
        let dbg = self.dbg.clone();
        let name = self.name.clone();
        let conf = self.conf.clone();
        let camera_id = self.camera_id;
        let exit = self.exit.clone();
        let storage_path = self.starage_path.clone();
        let table_defect = conf.tables.defect.clone();
        let table_defect_image = conf.tables.defect_image.clone();
        let rope = self.rope.clone();
        let api_client = self.api_client.clone();
        let service_waiting = ServiceWaiting::new(&name, conf.wait_started);
        let service_release = service_waiting.release();
        let handles_clone = self.handles.clone();
        log::debug!("{}.run | Preparing thread...", dbg);
        let handle = self.scheduler.spawn(move || {
            let dbg = &dbg;
            let defect = GeometryDefect::new(
                conf.defect_detection.fast_scan.geometry_defect_threshold,
                *Box::new(Mad::new()),
                EdgeDetection::new(
                    DetectingContoursCv::new(
                        conf.defect_detection.detecting_contours.clone(),
                        AutoBrightnessAndContrast::new(
                            conf.defect_detection.detecting_contours.brightness_contrast.histogram_clipping,
                            AutoGamma::new(
                                Initial::new(
                                    InitialCtx::new(),
                                ),
                            ),
                        ),
                    ),
                ),
            );
            let mut prev_index = None;
            let mut camera = Camera::new(camera_conf.clone());
            let camera_name = camera.name().join();
            match &camera_conf.from_path {
                Some(path) => {
                    log::info!("{dbg}.run | Starting camera from path '{path}'...");
                    let frames = camera.from_images(path)?;
                    service_release.add(Ok(()));
                    for frame in frames {
                        prev_index = Self::detection(
                            dbg,
                            frame,
                            &defect,
                            &rope,
                            &camera_name,
                            camera_id,
                            &table_defect,
                            &table_defect_image,
                            &storage_path,
                            &api_client,
                            prev_index,
                        );
                        std::thread::sleep(Duration::from_millis(50));
                    }
                }
                None => {
                    let camera_stream = camera.stream();
                    service_release.add(Ok(()));
                    'main: loop {
                        log::debug!("{dbg}.run | Starting camera...");
                        match camera.read() {
                            Ok(handle) => {
                                log::debug!("{dbg}.run | Starting camera - Ok");
                                handles_clone.push(handle);
                                log::debug!("{dbg}.run | Receiving frames from camera...");
                                'camera: loop {
                                    match camera_stream.recv_timeout(RECV_TIMEOUT) {
                                        Ok(frame) => {
                                            prev_index = Self::detection(
                                                dbg,
                                                frame,
                                                &defect,
                                                &rope,
                                                &camera_name,
                                                camera_id,
                                                &table_defect,
                                                &table_defect_image,
                                                &storage_path,
                                                &api_client,
                                                prev_index,
                                            );
                                        }
                                        Err(err) => {
                                            match err {
                                                crate::domain::RecvTimeoutError::Timeout => {}
                                                _ => {
                                                    break 'camera;
                                                }
                                            }
                                        }
                                    }
                                    if exit.load(Ordering::Acquire) {
                                        break 'main;
                                    }
                                }
                                camera.exit();
                            }
                            Err(err) => {
                                log::info!("{dbg}.run | Camera '{}' error: {:?}", camera_conf.name, err);
                            }
                        }
                    }
                    camera.exit();
                }
            }
            log::info!("{dbg}.run | Exit");
            Ok(())
        });
        match handle {
            Ok(handle) => {
                self.handles.push(handle);
                let r = match conf.wait_started {
                    Some(_) => {
                        log::info!("{}.run | Waiting while starting...", self.dbg);
                        service_waiting.wait()
                    }
                    None => Ok(()),
                };
                log::info!("{}.run | Starting - ok", self.dbg);
                r
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
