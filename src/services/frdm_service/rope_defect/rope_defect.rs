use std::{fs, path::{Path, PathBuf}, sync::{atomic::{AtomicBool, Ordering}, Arc}, time::{Duration, Instant}};
use chrono::Datelike;
use frdm_tools::{camera::Camera, AutoGamma, Context, ContextRead, Cropping, CroppingCtx, Eval, FastScan, FineScan, FineScanCtx, Gray, Image, Initial, InitialCtx, MetaCtx, RopeDefectCtx, RopeDefectKind};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{entity::{Name, Object}, Service, ServiceWaiting}, sync::Handles, thread_pool::Scheduler};
use crate::{domain::RECV_TIMEOUT, infra::ApiClient, services::frdm_service::{rope_defect::RopeDefectConf, Inputs}};

///
/// Dects defect on the frames coming from the camera
pub struct RopeDefect {
    name: Name,
    conf: RopeDefectConf,
    camera_id: usize,
    storage_path: PathBuf,
    inputs: Arc<Inputs>,
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
        storage_path: impl AsRef<Path>,
        inputs: Arc<Inputs>,
        api_client: Arc<ApiClient>,
        scheduler: Scheduler,
    ) -> Self {
        let name = Name::new(parent, "RopeDefect");
        let dbg = Dbg::new(name.parent(), name.me());
        Self {
            name,
            conf,
            camera_id,
            storage_path: storage_path.as_ref().join("rope-defects"),
            inputs,
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
        let error = Error::new(dbg, "save_image");
        let sql = format!("select * from clean_frdm_defect_image('{defect_id}', {camera_id});");
        let result = api_client.fetch(&sql).then(
            |reply| match reply {
                Ok(reply) => {
                    let mut errors = vec![];
                    for entry in reply {
                        match entry.get("path") {
                            Some(path) => {
                                match serde_json::from_value(path.to_owned()) {
                                    Ok(path) => {
                                        let path: String = path;
                                        if let Err(err) = fs::remove_file(&path) {
                                            errors.push(error.pass_with(format!("Can't delete image '{path}'"), err.to_string()));
                                        }
                                    }
                                    Err(err) => errors.push(error.pass_with(format!("Can't deserialize path '{path}'"), err.to_string())),
                                }
                            }
                            None => errors.push(error.err(format!("Field 'path' not found in sql reply: {:#?}", entry))),
                        }
                    }
                    match errors.is_empty() {
                        true => Ok(()),
                        false => Err(error.err(errors.iter().fold(String::new(), |acc, err| format!("{acc}\n\t{err}")))),
                    }
                }
                Err(err) => Err(error.pass_with("Sql error", err)),
            },
            |err| Err(error.pass(err)),
        );
        frame.save(img_path).map_err(|err| error.pass(err))?;
        result
    }
    ///
    /// Defect detection
    fn detection(
        dbg: &Dbg,
        frame: Image,
        defect: &FineScan,
        prev_index: Option<usize>,
    ) -> Option<usize> {
        let time = Instant::now();
        let slice_ix = frame.meta;
        if Some(slice_ix) != prev_index {
            let frame = Image { mat: frame.mat, meta: slice_ix };
            defect.eval(frame.clone());
            log::debug!("{dbg}.detection | Rope slice {}, Elapsed: {:?}", frame.meta, time.elapsed());
            Some(slice_ix)
        } else {
            log::trace!("{dbg}.detection | Elapsed: {:?}", time.elapsed());
            prev_index
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
        let storage_path = self.storage_path.clone();
        let table_defect = conf.tables.defect.clone();
        let table_defect_image = conf.tables.defect_image.clone();
        let inputs = self.inputs.clone();
        let api_client = self.api_client.clone();
        let service_waiting = ServiceWaiting::new(&name, conf.wait_started);
        let service_release = service_waiting.release();
        let handles_clone = self.handles.clone();
        log::debug!("{}.run | Preparing thread...", dbg);
        let scheduler = self.scheduler.clone();
        let handle = self.scheduler.spawn(move || {
            let dbg1 = dbg.clone();
            let defect = FineScan::new(
                conf.defect_detection.fine_scan,
                scheduler.clone(),
                Some(move |ctx: &Context| {
                    let defects = ContextRead::<RopeDefectCtx<FineScanCtx>>::read(ctx).result.clone();
                    let slice_ix = *ContextRead::<MetaCtx>::read(ctx);
                    if !defects.is_empty() {
                        log::warn!("{dbg1}.run | Slice {slice_ix} - Defects detected");
                        defects.iter().for_each(|defect| {
                            log::warn!("{dbg1}.run | Slice {slice_ix} - Defect {:?} detected", defect);
                            let defect_id = match defect {
                                RopeDefectKind::Expansion(_, _) => "expansion",
                                RopeDefectKind::Compressing(_, _) => "compressing",
                                RopeDefectKind::Hill(_, _) => "hill",
                                RopeDefectKind::Pit(_, _) => "pit",
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
                                                set (last, score) = (current_timestamp, {table_defect}.score + 1);
                                            insert into {table_defect_image} (frdm_defect_id, camera, path)
                                                values ('{defect_id}', {camera_id}, '{img_path}');
                                            exception
                                                when others then
                                                    rollback;
                                        end; $$
                                        language plpgsql;
                                    ");
                                    api_client.fetch(sql).then(
                                        |_| {
                                            let frame = &ContextRead::<CroppingCtx>::read(ctx).result;
                                            if let Err(err) = Self::save_image(
                                                &dbg1,
                                                &api_client,
                                                defect_id,
                                                camera_id,
                                                frame,
                                                &img_path,
                                            ) {
                                                log::warn!("{dbg1}.run | Save image error: {:?}", err);
                                            }
                                        },
                                        |err| {
                                            log::warn!("{dbg1}.run | Send sql error: {:?}", err);
                                        },
                                    );
                                }
                                None => log::warn!("{dbg1}.run | Wrong image path {}", img_path.display()),
                            };
                        });
                    } else {
                        log::info!("{dbg1}.run | Slice {slice_ix} - No defects detected");
                    }
                }),
                FastScan::new(
                    conf.defect_detection.fast_scan,
                    scheduler,
                    Gray::new(
                        AutoGamma::new(
                            conf.defect_detection.normalize.gamma.factor,
                            Cropping::new(
                                conf.defect_detection.normalize.cropping.x,
                                conf.defect_detection.normalize.cropping.width,
                                conf.defect_detection.normalize.cropping.y,
                                conf.defect_detection.normalize.cropping.height,
                                Initial::new(
                                    InitialCtx::new(),
                                ),
                                true,
                            ),
                            false,
                        ),
                    ),
                    false,
                ),
                false,
            );
            let mut prev_index = None;
            let mut camera = Camera::new(inputs.cam_segment_ix().clone(), camera_conf.clone());
            match &camera_conf.from_path {
                Some(path) => {
                    log::info!("{dbg}.run | Starting camera from path '{path}'...");
                    let frames = camera.from_images(path).unwrap();
                    service_release.add(Ok(()));
                    for frame in frames {
                        log::debug!("{dbg}.run | Receiving frames from camera - Ok");
                        prev_index = Self::detection(
                            &dbg,
                            frame,
                            &defect,
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
                                                &dbg,
                                                frame,
                                                &defect,
                                                prev_index,
                                            );
                                        }
                                        Err(err) => {
                                            match err {
                                                sal_sync::sync::channel::RecvTimeoutError::Timeout => {}
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
