#[cfg(test)]

mod cma_recorder {
    use log::{info, trace};
    use std::{env, fs, sync::{Arc, Mutex, Once, RwLock}, thread, time::{Duration, Instant}};
    use testing::{entities::test_value::Value, stuff::{max_test_duration::TestDuration, wait::WaitTread}};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::{api_client_config::ApiClientConfig, multi_queue_config::MultiQueueConfig, point_config::name::Name, task_config::TaskConfig},
        services::{
            api_cient::api_client::ApiClient, multi_queue::multi_queue::MultiQueue,
            safe_lock::SafeLock, service::service::Service, services::Services,
            task::{task::Task, task_test_receiver::TaskTestReceiver},
        },
        tests::unit::services::task::task_test_producer::TaskTestProducer,
    };
    ///
    ///
    static INIT: Once = Once::new();
    ///
    /// once called initialisation
    fn init_once() {
        INIT.call_once(|| {
            // implement your initialisation code to be called only once for current test file
        })
    }
    ///
    /// returns:
    ///  - ...
    fn init_each() -> () {}
    ///
    /// Testing the Recorder's SQL generated after detected operating cycle finished
    #[test]
    fn live_data() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "AppTest";
        let self_name = Name::new("", self_id);
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(20));
        test_duration.run().unwrap();
        //
        // can be changed
        trace!("dir: {:?}", env::current_dir());
        let services = Arc::new(RwLock::new(Services::new(self_id)));
        let mut tasks = vec![];
        let mut task_handles = vec![];
        let path = "./src/tests/unit/services/task/cma_recorder/operating-event.yaml";
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        let config: serde_yaml::Value = config;
                        for (key, config) in config.as_mapping().unwrap() {
                            let mut conf = serde_yaml::Mapping::new();
                            conf.insert(key.clone(), config.clone());
                            let config = TaskConfig::from_yaml(&self_name, &serde_yaml::Value::Mapping(conf));
                            let task = Arc::new(Mutex::new(Task::new(config, services.clone())));
                            services.wlock( self_id).insert(task.clone());
                            tasks.push(task);
                        }
                    }
                    Err(err) => panic!("{}.read | Error in config: {:?}\n\terror: {:?}", self_id, yaml_string, err),
                }
            }
            Err(err) => panic!("{}.read | File {} reading error: {:?}", self_id, path, err),
        }
        let conf = MultiQueueConfig::from_yaml(
            self_id,
            &serde_yaml::from_str(r"service MultiQueue:
                in queue in-queue:
                    max-length: 10000
                # send-to:
            ").unwrap(),
        );
        let multi_queue = Arc::new(Mutex::new(MultiQueue::new(conf, services.clone())));
        services.wlock(self_id).insert(multi_queue.clone());
        let conf = ApiClientConfig::from_yaml(
            self_id,
            &serde_yaml::from_str(r"service ApiClient:
                cycle: 100 ms
                reconnect: 1 s  # default 3 s
                address: 127.0.0.1:8080
                database: crane_data_server
                in queue in-queue:
                    max-length: 10000
                auth_token: 123!@#
                debug: true
            ").unwrap(),
        );
        let api_client = Arc::new(Mutex::new(ApiClient::new(conf)));
        services.wlock(self_id).insert(api_client.clone());
        let test_data = vec![
        //  step    nape                                input                    Pp Cycle   target_thrh             target_smooth
            ("00.-5",    format!("/{}/Load.Nom", self_id),   Value::Real(  150.00)),
            ("00.-3",    format!("/{}/Winch1.Load.Nom", self_id),   Value::Real(  150.00)),
            ("00.-2",    format!("/{}/Winch2.Load.Nom", self_id),   Value::Real(  150.00)),
            ("00.-1",    format!("/{}/Winch3.Load.Nom", self_id),   Value::Real(  150.00)),
            ("00.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("01.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("02.0",    format!("/{}/Load", self_id),       Value::Real(  3.30)),
            ("03.0",    format!("/{}/Load", self_id),       Value::Real(  0.10)),
            ("04.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("05.0",    format!("/{}/Load", self_id),       Value::Real(  1.60)),
            ("06.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("07.0",    format!("/{}/Load", self_id),       Value::Real(  7.20)),
            ("08.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("09.0",    format!("/{}/Load", self_id),       Value::Real(  0.30)),
            ("10.0",    format!("/{}/Load", self_id),       Value::Real(  2.20)),
            ("11.0",    format!("/{}/Load", self_id),       Value::Real(  8.10)),
            ("12.0",    format!("/{}/Load", self_id),       Value::Real(  1.90)),
            ("13.0",    format!("/{}/Load", self_id),       Value::Real(  0.10)),
            ("14.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("15.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("16.0",    format!("/{}/Load", self_id),       Value::Real(  5.00)),
            ("17.0",    format!("/{}/Load", self_id),       Value::Real(  2.00)),
            ("17.1",    format!("/{}/Winch1.Load.Limiter.Trip", self_id),       Value::Bool(true)),
            ("17.2",    format!("/{}/Winch1.Load.Limiter.Trip", self_id),       Value::Bool(false)),
            ("17.3",    format!("/{}/Winch2.Load.Limiter.Trip", self_id),       Value::Bool(true)),
            ("17.4",    format!("/{}/Winch2.Load.Limiter.Trip", self_id),       Value::Bool(false)),
            ("17.5",    format!("/{}/Winch3.Load.Limiter.Trip", self_id),       Value::Bool(true)),
            ("17.6",    format!("/{}/Winch3.Load.Limiter.Trip", self_id),       Value::Bool(false)),
            ("18.0",    format!("/{}/Load", self_id),       Value::Real(  1.00)),
            ("19.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("20.0",    format!("/{}/Load", self_id),       Value::Real(  2.00)),
            ("21.0",    format!("/{}/Load", self_id),       Value::Real(  4.00)),
            ("22.0",    format!("/{}/Load", self_id),       Value::Real(  6.00)),
            ("23.0",    format!("/{}/Load", self_id),       Value::Real( 12.00)),
            ("24.0",    format!("/{}/Load", self_id),       Value::Real( 64.00)),
            ("26.1",    format!("/{}/CraneMode.MOPS", self_id),       Value::Int(1)),
            ("25.0",    format!("/{}/Load", self_id),       Value::Real(128.00)),
            ("26.0",    format!("/{}/Load", self_id),       Value::Real(120.00)),
            ("26.1",    format!("/{}/CraneMode.AOPS", self_id),       Value::Int(1)),
            ("27.0",    format!("/{}/Load", self_id),       Value::Real(133.00)),
            ("28.0",    format!("/{}/Load", self_id),       Value::Real(121.00)),
            ("28.1",    format!("/{}/Load", self_id),       Value::Real(141.00)),
            ("29.0",    format!("/{}/Load", self_id),       Value::Real(130.00)),
            ("30.0",    format!("/{}/Load", self_id),       Value::Real(127.00)),
            ("31.0",    format!("/{}/Load", self_id),       Value::Real(123.00)),
            ("32.0",    format!("/{}/Load", self_id),       Value::Real(122.00)),
            ("33.0",    format!("/{}/Load", self_id),       Value::Real(120.00)),
            ("34.0",    format!("/{}/Load", self_id),       Value::Real( 64.00)),
            ("35.0",    format!("/{}/Load", self_id),       Value::Real( 32.00)),
            ("36.0",    format!("/{}/Load", self_id),       Value::Real( 24.00)),
            ("37.0",    format!("/{}/Load", self_id),       Value::Real( 12.00)),
            ("38.0",    format!("/{}/Load", self_id),       Value::Real(  8.00)),
            ("39.0",    format!("/{}/Load", self_id),       Value::Real( 17.00)),
            ("40.0",    format!("/{}/Load", self_id),       Value::Real( 10.00)),
            ("41.0",    format!("/{}/Load", self_id),       Value::Real(  7.00)),
            ("42.0",    format!("/{}/Load", self_id),       Value::Real(  3.00)),
            ("43.0",    format!("/{}/Load", self_id),       Value::Real(  6.00)),
            ("44.0",    format!("/{}/Load", self_id),       Value::Real(  4.00)),
            ("45.0",    format!("/{}/Load", self_id),       Value::Real(  2.00)),
            ("46.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("47.0",    format!("/{}/Load", self_id),       Value::Real(  4.00)),
            ("47.1",    format!("/{}/Winch1.Load.Limiter.Trip", self_id),       Value::Bool(true)),
            ("48.0",    format!("/{}/Load", self_id),       Value::Real(  2.00)),
            ("49.0",    format!("/{}/Load", self_id),       Value::Real(  1.00)),
            ("50.0",    format!("/{}/Load", self_id),       Value::Real(  3.00)),
            ("51.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("52.0",    format!("/{}/Load", self_id),       Value::Real(  2.00)),
            ("53.0",    format!("/{}/Load", self_id),       Value::Real(  1.00)),
            ("54.0",    format!("/{}/Load", self_id),       Value::Real(  0.70)),
            ("55.0",    format!("/{}/Load", self_id),       Value::Real(  0.80)),
            ("56.0",    format!("/{}/Load", self_id),       Value::Real(  0.40)),
            ("57.0",    format!("/{}/Load", self_id),       Value::Real(  0.30)),
            ("58.0",    format!("/{}/Load", self_id),       Value::Real(  0.20)),
            ("59.0",    format!("/{}/Load", self_id),       Value::Real(  0.10)),
            ("60.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("61.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("62.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("63.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),

            ("64.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("65.0",    format!("/{}/Load", self_id),       Value::Real(  3.30)),
            ("66.0",    format!("/{}/Load", self_id),       Value::Real(  0.10)),
            ("67.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("68.0",    format!("/{}/Load", self_id),       Value::Real(  1.60)),
            ("69.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("70.0",    format!("/{}/Load", self_id),       Value::Real(  7.20)),
            ("71.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("72.0",    format!("/{}/Load", self_id),       Value::Real(  0.30)),
            ("73.0",    format!("/{}/Load", self_id),       Value::Real(  2.20)),
            ("74.0",    format!("/{}/Load", self_id),       Value::Real(  8.10)),
            ("75.0",    format!("/{}/Load", self_id),       Value::Real(  1.90)),
            ("76.0",    format!("/{}/Load", self_id),       Value::Real(  0.10)),
            ("77.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("78.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("79.0",    format!("/{}/Load", self_id),       Value::Real(  5.00)),
            ("80.0",    format!("/{}/Load", self_id),       Value::Real(  2.00)),
            ("81.1",    format!("/{}/Winch1.Load.Limiter.Trip", self_id),       Value::Bool(true)),
            ("82.2",    format!("/{}/Winch1.Load.Limiter.Trip", self_id),       Value::Bool(false)),
            ("83.3",    format!("/{}/Winch2.Load.Limiter.Trip", self_id),       Value::Bool(true)),
            ("84.4",    format!("/{}/Winch2.Load.Limiter.Trip", self_id),       Value::Bool(false)),
            ("85.5",    format!("/{}/Winch3.Load.Limiter.Trip", self_id),       Value::Bool(true)),
            ("86.6",    format!("/{}/Winch3.Load.Limiter.Trip", self_id),       Value::Bool(false)),
            ("87.0",    format!("/{}/Load", self_id),       Value::Real(  1.00)),
            ("88.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("89.0",    format!("/{}/Load", self_id),       Value::Real(  2.00)),
            ("90.0",    format!("/{}/Load", self_id),       Value::Real(  4.00)),
            ("91.0",    format!("/{}/Load", self_id),       Value::Real(  6.00)),
            ("92.0",    format!("/{}/Load", self_id),       Value::Real( 12.00)),
            ("93.0",    format!("/{}/Load", self_id),       Value::Real( 64.00)),
            ("94.0",    format!("/{}/Load", self_id),       Value::Real(128.00)),
            ("95.0",    format!("/{}/Load", self_id),       Value::Real(120.00)),
            ("96.0",    format!("/{}/Load", self_id),       Value::Real(133.00)),
            ("97.0",    format!("/{}/Load", self_id),       Value::Real(121.00)),
            ("98.0",    format!("/{}/Load", self_id),       Value::Real(130.00)),
            ("99.0",    format!("/{}/Load", self_id),       Value::Real(127.00)),
            ("100.0",    format!("/{}/Load", self_id),       Value::Real(123.00)),
            ("101.0",    format!("/{}/Load", self_id),       Value::Real(122.00)),
            ("102.0",    format!("/{}/Load", self_id),       Value::Real(120.00)),
            ("103.0",    format!("/{}/Load", self_id),       Value::Real( 64.00)),
            ("104.0",    format!("/{}/Load", self_id),       Value::Real( 32.00)),
            ("105.0",    format!("/{}/Load", self_id),       Value::Real( 24.00)),
            ("106.0",    format!("/{}/Load", self_id),       Value::Real( 12.00)),
            ("107.0",    format!("/{}/Load", self_id),       Value::Real(  8.00)),
            ("108.0",    format!("/{}/Load", self_id),       Value::Real( 17.00)),
            ("109.0",    format!("/{}/Load", self_id),       Value::Real( 10.00)),
            ("110.0",    format!("/{}/Load", self_id),       Value::Real(  7.00)),
            ("111.0",    format!("/{}/Load", self_id),       Value::Real(  3.00)),
            ("112.0",    format!("/{}/Load", self_id),       Value::Real(  6.00)),
            ("113.0",    format!("/{}/Load", self_id),       Value::Real(  4.00)),
            ("114.0",    format!("/{}/Load", self_id),       Value::Real(  2.00)),
            ("115.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("116.0",    format!("/{}/Load", self_id),       Value::Real(  4.00)),
            ("117.1",    format!("/{}/Winch1.Load.Limiter.Trip", self_id),       Value::Bool(true)),
            ("118.0",    format!("/{}/Load", self_id),       Value::Real(  2.00)),
            ("119.0",    format!("/{}/Load", self_id),       Value::Real(  1.00)),
            ("120.0",    format!("/{}/Load", self_id),       Value::Real(  3.00)),
            ("121.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("122.0",    format!("/{}/Load", self_id),       Value::Real(  2.00)),
            ("123.0",    format!("/{}/Load", self_id),       Value::Real(  1.00)),
            ("124.0",    format!("/{}/Load", self_id),       Value::Real(  0.70)),
            ("125.0",    format!("/{}/Load", self_id),       Value::Real(  0.80)),
            ("126.0",    format!("/{}/Load", self_id),       Value::Real(  0.40)),
            ("127.0",    format!("/{}/Load", self_id),       Value::Real(  0.30)),


            ("64.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
            ("65.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  3.30)),
            ("66.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.10)),
            ("67.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
            ("68.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  1.60)),
            ("69.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
            ("70.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  7.20)),
            ("71.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
            ("72.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.30)),
            ("73.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  2.20)),
            ("74.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  8.10)),
            ("75.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  1.90)),
            ("76.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.10)),
            ("77.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
            ("78.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
            ("79.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  5.00)),
            ("80.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  2.00)),
            ("87.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  1.00)),
            ("88.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
            ("89.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  2.00)),
            ("90.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  4.00)),
            ("91.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  6.00)),
            ("92.0",    format!("/{}/Winch1.Load", self_id),       Value::Real( 12.00)),
            ("93.0",    format!("/{}/Winch1.Load", self_id),       Value::Real( 64.00)),
            ("94.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(128.00)),
            ("95.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(120.00)),
            ("96.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(133.00)),
            ("97.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(121.00)),
            ("98.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(130.00)),
            ("99.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(127.00)),
            ("100.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(123.00)),
            ("101.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(122.00)),
            ("102.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(120.00)),
            ("103.0",    format!("/{}/Winch1.Load", self_id),       Value::Real( 64.00)),
            ("104.0",    format!("/{}/Winch1.Load", self_id),       Value::Real( 32.00)),
            ("105.0",    format!("/{}/Winch1.Load", self_id),       Value::Real( 24.00)),
            ("106.0",    format!("/{}/Winch1.Load", self_id),       Value::Real( 12.00)),
            ("107.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  8.00)),
            ("108.0",    format!("/{}/Winch1.Load", self_id),       Value::Real( 17.00)),
            ("109.0",    format!("/{}/Winch1.Load", self_id),       Value::Real( 10.00)),
            ("110.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  7.00)),
            ("111.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  3.00)),
            ("112.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  6.00)),
            ("113.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  4.00)),
            ("114.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  2.00)),
            ("115.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
            ("116.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  4.00)),
            ("118.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  2.00)),
            ("119.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  1.00)),
            ("120.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  3.00)),
            ("121.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
            ("122.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  2.00)),
            ("123.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  1.00)),
            ("124.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.70)),
            ("125.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.80)),
            ("126.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.40)),
            ("127.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.30)),

            ("64.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
            ("65.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  3.30)),
            ("66.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.10)),
            ("67.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
            ("68.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  1.60)),
            ("69.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
            ("70.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  7.20)),
            ("71.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
            ("72.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.30)),
            ("73.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  2.20)),
            ("74.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  8.10)),
            ("75.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  1.90)),
            ("76.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.10)),
            ("77.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
            ("78.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
            ("79.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  5.00)),
            ("80.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  2.00)),
            ("87.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  1.00)),
            ("88.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
            ("89.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  2.00)),
            ("90.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  4.00)),
            ("91.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  6.00)),
            ("92.0",    format!("/{}/Winch2.Load", self_id),       Value::Real( 12.00)),
            ("93.0",    format!("/{}/Winch2.Load", self_id),       Value::Real( 64.00)),
            ("94.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(128.00)),
            ("95.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(120.00)),
            ("96.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(133.00)),
            ("97.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(121.00)),
            ("98.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(130.00)),
            ("99.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(127.00)),
            ("100.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(123.00)),
            ("101.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(122.00)),
            ("102.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(120.00)),
            ("103.0",    format!("/{}/Winch2.Load", self_id),       Value::Real( 64.00)),
            ("104.0",    format!("/{}/Winch2.Load", self_id),       Value::Real( 32.00)),
            ("105.0",    format!("/{}/Winch2.Load", self_id),       Value::Real( 24.00)),
            ("106.0",    format!("/{}/Winch2.Load", self_id),       Value::Real( 12.00)),
            ("107.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  8.00)),
            ("108.0",    format!("/{}/Winch2.Load", self_id),       Value::Real( 17.00)),
            ("109.0",    format!("/{}/Winch2.Load", self_id),       Value::Real( 10.00)),
            ("110.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  7.00)),
            ("111.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  3.00)),
            ("112.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  6.00)),
            ("113.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  4.00)),
            ("114.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  2.00)),
            ("115.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
            ("116.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  4.00)),
            ("118.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  2.00)),
            ("119.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  1.00)),
            ("120.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  3.00)),
            ("121.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
            ("122.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  2.00)),
            ("123.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  1.00)),
            ("124.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.70)),
            ("125.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.80)),
            ("126.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.40)),
            ("127.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.30)),

            ("64.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
            ("65.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  3.30)),
            ("66.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.10)),
            ("67.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
            ("68.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  1.60)),
            ("69.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
            ("70.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  7.20)),
            ("71.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
            ("72.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.30)),
            ("73.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  2.20)),
            ("74.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  8.10)),
            ("75.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  1.90)),
            ("76.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.10)),
            ("77.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
            ("78.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
            ("79.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  5.00)),
            ("80.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  2.00)),
            ("87.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  1.00)),
            ("88.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
            ("89.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  2.00)),
            ("90.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  4.00)),
            ("91.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  6.00)),
            ("92.0",    format!("/{}/Winch3.Load", self_id),       Value::Real( 12.00)),
            ("93.0",    format!("/{}/Winch3.Load", self_id),       Value::Real( 64.00)),
            ("94.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(128.00)),
            ("95.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(120.00)),
            ("96.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(133.00)),
            ("97.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(121.00)),
            ("98.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(130.00)),
            ("99.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(127.00)),
            ("100.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(123.00)),
            ("101.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(122.00)),
            ("102.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(120.00)),
            ("103.0",    format!("/{}/Winch3.Load", self_id),       Value::Real( 64.00)),
            ("104.0",    format!("/{}/Winch3.Load", self_id),       Value::Real( 32.00)),
            ("105.0",    format!("/{}/Winch3.Load", self_id),       Value::Real( 24.00)),
            ("106.0",    format!("/{}/Winch3.Load", self_id),       Value::Real( 12.00)),
            ("107.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  8.00)),
            ("108.0",    format!("/{}/Winch3.Load", self_id),       Value::Real( 17.00)),
            ("109.0",    format!("/{}/Winch3.Load", self_id),       Value::Real( 10.00)),
            ("110.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  7.00)),
            ("111.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  3.00)),
            ("112.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  6.00)),
            ("113.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  4.00)),
            ("114.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  2.00)),
            ("115.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
            ("116.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  4.00)),
            ("118.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  2.00)),
            ("119.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  1.00)),
            ("120.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  3.00)),
            ("121.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
            ("122.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  2.00)),
            ("123.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  1.00)),
            ("124.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.70)),
            ("125.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.80)),
            ("126.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.40)),
            ("127.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.30)),

            ("64.0",    format!("/{}/Exit", self_id),       Value::String("exit".to_owned())),
        ];
        let total_count = test_data.len();
        let receiver = Arc::new(Mutex::new(TaskTestReceiver::new(
            self_id,
            "",
            "in-queue",
            total_count * 1000,
        )));
        services.wlock(self_id).insert(receiver.clone());
        let test_data: Vec<(String, Value)> = test_data.into_iter().map(|(_, name, value)| {
            (name, value)
        }).collect();
        let producer = Arc::new(Mutex::new(TaskTestProducer::new(
            self_id,
            &format!("/{}/MultiQueue.in-queue", self_id),
            Duration::from_millis(10),
            services.clone(),
            &test_data,
        )));
        services.wlock(self_id).insert(producer.clone());
        thread::sleep(Duration::from_millis(100));
        let services_handle = services.wlock(self_id).run().unwrap();
        let multi_queue_handle = multi_queue.lock().unwrap().run().unwrap();
        let api_client_handle = api_client.lock().unwrap().run().unwrap();
        let receiver_handle = receiver.lock().unwrap().run().unwrap();
        info!("receiver runing - ok");
        for task in &tasks {
            let handle = task.lock().unwrap().run().unwrap();
            task_handles.push(handle);
        }
        info!("task runing - ok");
        thread::sleep(Duration::from_millis(600));
        let producer_handle = producer.lock().unwrap().run().unwrap();
        info!("producer runing - ok");
        thread::sleep(Duration::from_millis(1200));
        let time = Instant::now();
        receiver_handle.wait().unwrap();
        producer.lock().unwrap().exit();
        multi_queue.lock().unwrap().exit();
        for task in tasks {
            task.lock().unwrap().exit();
        }
        services.rlock(self_id).exit();
        for handle in task_handles {
            handle.wait().unwrap();
        }
        api_client.lock().unwrap().exit();
        api_client_handle.wait().unwrap();
        producer_handle.wait().unwrap();
        multi_queue_handle.wait().unwrap();
        services_handle.wait().unwrap();
        let sent = producer.lock().unwrap().sent().lock().unwrap().len();
        let result = receiver.lock().unwrap().received().lock().unwrap().len();
        println!(" elapsed: {:?}", time.elapsed());
        println!("    sent: {:?}", sent);
        println!("received: {:?}", result);
        for (i, result) in receiver.lock().unwrap().received().lock().unwrap().iter().enumerate() {
            println!("received: {}\t|\t{}\t|\t{:?}", i, result.name(), result.value());
            // assert!(result.name() == target_name, "step {} \nresult: {:?}\ntarget: {:?}", step, result.name(), target_name);
        };
        assert!(sent == total_count, "\nresult: {:?}\ntarget: {:?}", sent, total_count);
        // assert!(result >= total_count, "\nresult: {:?}\ntarget: {:?}", result, total_count);
        test_duration.exit();
        loop {
            thread::sleep(Duration::from_millis(100));
        }
    }
}

