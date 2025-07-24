#[cfg(test)]

mod cma_recorder {
    use sal_sync::{services::{
        conf::{ConfTree, ServicesConf}, entity::Name,
        MultiQueue, MultiQueueConf,
        Service, Services,
    }, thread_pool::ThreadPool};
    use std::{env, fs, sync::{Arc, Once}, thread, time::{Duration, Instant}};
    use testing::{entities::test_value::Value, stuff::max_test_duration::TestDuration};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::api_client_conf::ApiClientConf,
        services::{
            api_cient::api_client::ApiClient,
            task::{Task, TaskConf, TaskTestReceiver},
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
    /// Testing the Recorder | Operating event
    ///  - SQL of the Crane.Load live monitoring events per cycle
    ///  - SQL of the Winch1.Load live monitoring events per cycle
    ///  - SQL of the Winch2.Load live monitoring events per cycle
    ///  - SQL of the Winch3.Load live monitoring events per cycle  
    ///  ...to be extended
    #[test]
    fn operating_event() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        let dbg = "AppTest";
        let self_name = Name::new("", dbg);
        println!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(20));
        test_duration.run().unwrap();
        //
        // can be changed
        log::trace!("dir: {:?}", env::current_dir());
        let tp = ThreadPool::new(dbg, Some(8));
        let services = Arc::new(Services::new(dbg, ServicesConf::new(
            dbg, 
            ConfTree::new_root(serde_yaml::from_str(r#"
                retain:
                    path: assets/testing/retain/
                    point:
                        path: point/id.json
            "#).unwrap()),
        ), Some(tp.scheduler())));
        let mut tasks = vec![];
        let path = "./src/tests/unit/services/task/cma_recorder/operating-event.yaml";
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        let config: serde_yaml::Value = config;
                        for (key, config) in config.as_mapping().unwrap() {
                            let mut conf = serde_yaml::Mapping::new();
                            conf.insert(key.clone(), config.clone());
                            let config = TaskConf::from_yaml(&self_name, &serde_yaml::Value::Mapping(conf));
                            let task = Arc::new(Task::new(config, services.clone(), tp.scheduler()));
                            services.insert(task.clone());
                            tasks.push(task);
                        }
                    }
                    Err(err) => panic!("{}.read | Error in config: {:?}\n\terror: {:?}", dbg, yaml_string, err),
                }
            }
            Err(err) => panic!("{}.read | File {} reading error: {:?}", dbg, path, err),
        }
        let conf = MultiQueueConf::from_yaml(
            dbg,
            &serde_yaml::from_str(r"service MultiQueue:
                in queue in-queue:
                    max-length: 10000
            ").unwrap(),
        );
        let multi_queue = Arc::new(MultiQueue::new(conf, services.clone(), Some(tp.scheduler())));
        services.insert(multi_queue.clone());
        let conf = ApiClientConf::from_yaml(
            dbg,
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
        let api_client = Arc::new(ApiClient::new(conf, tp.scheduler()));
        services.insert(api_client.clone());
        let test_data = vec![
        //  step        nape                                input
            ("00.-5",    format!("/{}/Load.Nom", dbg),   Value::Real(  150.00)),
            ("00.-3",    format!("/{}/Winch1.Load.Nom", dbg),   Value::Real(  150.00)),
            ("00.-2",    format!("/{}/Winch2.Load.Nom", dbg),   Value::Real(  150.00)),
            ("00.-1",    format!("/{}/Winch3.Load.Nom", dbg),   Value::Real(  150.00)),
            ("00.0",    format!("/{}/Load", dbg),       Value::Real(  0.00)),
            ("01.0",    format!("/{}/Load", dbg),       Value::Real(  0.00)),
            ("02.0",    format!("/{}/Load", dbg),       Value::Real(  3.30)),
            ("03.0",    format!("/{}/Load", dbg),       Value::Real(  0.10)),
            ("04.0",    format!("/{}/Load", dbg),       Value::Real(  0.00)),
            ("05.0",    format!("/{}/Load", dbg),       Value::Real(  1.60)),
            ("06.0",    format!("/{}/Load", dbg),       Value::Real(  0.00)),
            ("07.0",    format!("/{}/Load", dbg),       Value::Real(  7.20)),
            ("08.0",    format!("/{}/Load", dbg),       Value::Real(  0.00)),
            ("09.0",    format!("/{}/Load", dbg),       Value::Real(  0.30)),
            ("10.0",    format!("/{}/Load", dbg),       Value::Real(  2.20)),
            ("11.0",    format!("/{}/Load", dbg),       Value::Real(  8.10)),
            ("12.0",    format!("/{}/Load", dbg),       Value::Real(  1.90)),
            ("13.0",    format!("/{}/Load", dbg),       Value::Real(  0.10)),
            ("14.0",    format!("/{}/Load", dbg),       Value::Real(  0.00)),
            ("15.0",    format!("/{}/Load", dbg),       Value::Real(  0.00)),
            ("16.0",    format!("/{}/Load", dbg),       Value::Real(  5.00)),
            ("17.0",    format!("/{}/Load", dbg),       Value::Real(  2.00)),
            ("17.1",    format!("/{}/Winch1.Load.Limiter.Trip", dbg),       Value::Bool(true)),
            ("17.2",    format!("/{}/Winch1.Load.Limiter.Trip", dbg),       Value::Bool(false)),
            ("17.3",    format!("/{}/Winch2.Load.Limiter.Trip", dbg),       Value::Bool(true)),
            ("17.4",    format!("/{}/Winch2.Load.Limiter.Trip", dbg),       Value::Bool(false)),
            ("17.5",    format!("/{}/Winch3.Load.Limiter.Trip", dbg),       Value::Bool(true)),
            ("17.6",    format!("/{}/Winch3.Load.Limiter.Trip", dbg),       Value::Bool(false)),
            ("18.0",    format!("/{}/Load", dbg),       Value::Real(  1.00)),
            ("19.0",    format!("/{}/Load", dbg),       Value::Real(  0.00)),
            ("20.0",    format!("/{}/Load", dbg),       Value::Real(  2.00)),
            ("21.0",    format!("/{}/Load", dbg),       Value::Real(  4.00)),
            ("22.0",    format!("/{}/Load", dbg),       Value::Real(  6.00)),
            ("23.0",    format!("/{}/Load", dbg),       Value::Real( 12.00)),
            ("24.0",    format!("/{}/Load", dbg),       Value::Real( 64.00)),
            ("26.1",    format!("/{}/CraneMode.MOPS", dbg),       Value::Int(1)),
            ("25.0",    format!("/{}/Load", dbg),       Value::Real(128.00)),
            ("26.0",    format!("/{}/Load", dbg),       Value::Real(120.00)),
            ("26.1",    format!("/{}/CraneMode.AOPS", dbg),       Value::Int(1)),
            ("27.0",    format!("/{}/Load", dbg),       Value::Real(133.00)),
            ("28.0",    format!("/{}/Load", dbg),       Value::Real(121.00)),
            // ("28.1",    format!("/{}/Load", self_id),       Value::Real(141.00)),
            ("29.0",    format!("/{}/Load", dbg),       Value::Real(130.00)),
            ("30.0",    format!("/{}/Load", dbg),       Value::Real(127.00)),
            ("31.0",    format!("/{}/Load", dbg),       Value::Real(123.00)),
            ("32.0",    format!("/{}/Load", dbg),       Value::Real(122.00)),
            ("33.0",    format!("/{}/Load", dbg),       Value::Real(120.00)),
            ("34.0",    format!("/{}/Load", dbg),       Value::Real( 64.00)),
            ("35.0",    format!("/{}/Load", dbg),       Value::Real( 32.00)),
            ("36.0",    format!("/{}/Load", dbg),       Value::Real( 24.00)),
            ("37.0",    format!("/{}/Load", dbg),       Value::Real( 12.00)),
            ("38.0",    format!("/{}/Load", dbg),       Value::Real(  8.00)),
            ("39.0",    format!("/{}/Load", dbg),       Value::Real( 17.00)),
            ("40.0",    format!("/{}/Load", dbg),       Value::Real( 10.00)),
            ("41.0",    format!("/{}/Load", dbg),       Value::Real(  7.00)),
            ("42.0",    format!("/{}/Load", dbg),       Value::Real(  3.00)),
            ("43.0",    format!("/{}/Load", dbg),       Value::Real(  6.00)),
            ("44.0",    format!("/{}/Load", dbg),       Value::Real(  4.00)),
            ("45.0",    format!("/{}/Load", dbg),       Value::Real(  2.00)),
            ("46.0",    format!("/{}/Load", dbg),       Value::Real(  0.00)),
            ("47.0",    format!("/{}/Load", dbg),       Value::Real(  4.00)),
            ("47.1",    format!("/{}/Winch1.Load.Limiter.Trip", dbg),       Value::Bool(true)),
            ("48.0",    format!("/{}/Load", dbg),       Value::Real(  2.00)),
            ("49.0",    format!("/{}/Load", dbg),       Value::Real(  1.00)),
            ("50.0",    format!("/{}/Load", dbg),       Value::Real(  3.00)),
            ("51.0",    format!("/{}/Load", dbg),       Value::Real(  0.00)),
            ("52.0",    format!("/{}/Load", dbg),       Value::Real(  2.00)),
            ("53.0",    format!("/{}/Load", dbg),       Value::Real(  1.00)),
            ("54.0",    format!("/{}/Load", dbg),       Value::Real(  0.70)),
            ("55.0",    format!("/{}/Load", dbg),       Value::Real(  0.80)),
            ("56.0",    format!("/{}/Load", dbg),       Value::Real(  0.40)),
            ("57.0",    format!("/{}/Load", dbg),       Value::Real(  0.30)),
            ("58.0",    format!("/{}/Load", dbg),       Value::Real(  0.20)),
            ("59.0",    format!("/{}/Load", dbg),       Value::Real(  0.10)),
            ("60.0",    format!("/{}/Load", dbg),       Value::Real(  0.00)),
            ("61.0",    format!("/{}/Load", dbg),       Value::Real(  0.00)),
            ("62.0",    format!("/{}/Load", dbg),       Value::Real(  0.00)),
            ("63.0",    format!("/{}/Load", dbg),       Value::Real(  0.00)),

            ("64.0",    format!("/{}/Load", dbg),       Value::Real(  0.00)),
            ("65.0",    format!("/{}/Load", dbg),       Value::Real(  3.30)),
            ("66.0",    format!("/{}/Load", dbg),       Value::Real(  0.10)),
            ("67.0",    format!("/{}/Load", dbg),       Value::Real(  0.00)),
            ("68.0",    format!("/{}/Load", dbg),       Value::Real(  1.60)),
            ("69.0",    format!("/{}/Load", dbg),       Value::Real(  0.00)),
            ("70.0",    format!("/{}/Load", dbg),       Value::Real(  7.20)),
            ("71.0",    format!("/{}/Load", dbg),       Value::Real(  0.00)),
            ("72.0",    format!("/{}/Load", dbg),       Value::Real(  0.30)),
            ("73.0",    format!("/{}/Load", dbg),       Value::Real(  2.20)),
            ("74.0",    format!("/{}/Load", dbg),       Value::Real(  8.10)),
            ("75.0",    format!("/{}/Load", dbg),       Value::Real(  1.90)),
            ("76.0",    format!("/{}/Load", dbg),       Value::Real(  0.10)),
            ("77.0",    format!("/{}/Load", dbg),       Value::Real(  0.00)),
            ("78.0",    format!("/{}/Load", dbg),       Value::Real(  0.00)),
            ("79.0",    format!("/{}/Load", dbg),       Value::Real(  5.00)),
            ("80.0",    format!("/{}/Load", dbg),       Value::Real(  2.00)),
            ("81.1",    format!("/{}/Winch1.Load.Limiter.Trip", dbg),       Value::Bool(true)),
            ("82.2",    format!("/{}/Winch1.Load.Limiter.Trip", dbg),       Value::Bool(false)),
            ("83.3",    format!("/{}/Winch2.Load.Limiter.Trip", dbg),       Value::Bool(true)),
            ("84.4",    format!("/{}/Winch2.Load.Limiter.Trip", dbg),       Value::Bool(false)),
            ("85.5",    format!("/{}/Winch3.Load.Limiter.Trip", dbg),       Value::Bool(true)),
            ("86.6",    format!("/{}/Winch3.Load.Limiter.Trip", dbg),       Value::Bool(false)),
            ("87.0",    format!("/{}/Load", dbg),       Value::Real(  1.00)),
            ("88.0",    format!("/{}/Load", dbg),       Value::Real(  0.00)),
            ("89.0",    format!("/{}/Load", dbg),       Value::Real(  2.00)),
            ("90.0",    format!("/{}/Load", dbg),       Value::Real(  4.00)),
            ("91.0",    format!("/{}/Load", dbg),       Value::Real(  6.00)),
            ("92.0",    format!("/{}/Load", dbg),       Value::Real( 12.00)),
            ("93.0",    format!("/{}/Load", dbg),       Value::Real( 64.00)),
            ("94.0",    format!("/{}/Load", dbg),       Value::Real(128.00)),
            ("95.0",    format!("/{}/Load", dbg),       Value::Real(120.00)),
            ("96.0",    format!("/{}/Load", dbg),       Value::Real(133.00)),
            ("97.0",    format!("/{}/Load", dbg),       Value::Real(121.00)),
            ("98.0",    format!("/{}/Load", dbg),       Value::Real(130.00)),
            ("99.0",    format!("/{}/Load", dbg),       Value::Real(127.00)),
            ("100.0",    format!("/{}/Load", dbg),       Value::Real(123.00)),
            ("101.0",    format!("/{}/Load", dbg),       Value::Real(122.00)),
            ("102.0",    format!("/{}/Load", dbg),       Value::Real(120.00)),
            ("103.0",    format!("/{}/Load", dbg),       Value::Real( 64.00)),
            ("104.0",    format!("/{}/Load", dbg),       Value::Real( 32.00)),
            ("105.0",    format!("/{}/Load", dbg),       Value::Real( 24.00)),
            ("106.0",    format!("/{}/Load", dbg),       Value::Real( 12.00)),
            ("107.0",    format!("/{}/Load", dbg),       Value::Real(  8.00)),
            ("108.0",    format!("/{}/Load", dbg),       Value::Real( 17.00)),
            ("109.0",    format!("/{}/Load", dbg),       Value::Real( 10.00)),
            ("110.0",    format!("/{}/Load", dbg),       Value::Real(  7.00)),
            ("111.0",    format!("/{}/Load", dbg),       Value::Real(  3.00)),
            ("112.0",    format!("/{}/Load", dbg),       Value::Real(  6.00)),
            ("113.0",    format!("/{}/Load", dbg),       Value::Real(  4.00)),
            ("114.0",    format!("/{}/Load", dbg),       Value::Real(  2.00)),
            ("115.0",    format!("/{}/Load", dbg),       Value::Real(  0.00)),
            ("116.0",    format!("/{}/Load", dbg),       Value::Real(  4.00)),
            ("117.1",    format!("/{}/Winch1.Load.Limiter.Trip", dbg),       Value::Bool(true)),
            ("118.0",    format!("/{}/Load", dbg),       Value::Real(  2.00)),
            ("119.0",    format!("/{}/Load", dbg),       Value::Real(  1.00)),
            ("120.0",    format!("/{}/Load", dbg),       Value::Real(  3.00)),
            ("121.0",    format!("/{}/Load", dbg),       Value::Real(  0.00)),
            ("122.0",    format!("/{}/Load", dbg),       Value::Real(  2.00)),
            ("123.0",    format!("/{}/Load", dbg),       Value::Real(  1.00)),
            ("124.0",    format!("/{}/Load", dbg),       Value::Real(  0.70)),
            ("125.0",    format!("/{}/Load", dbg),       Value::Real(  0.80)),
            ("126.0",    format!("/{}/Load", dbg),       Value::Real(  0.40)),
            ("127.0",    format!("/{}/Load", dbg),       Value::Real(  0.30)),


            ("64.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  0.00)),
            ("65.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  3.30)),
            ("66.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  0.10)),
            ("67.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  0.00)),
            ("68.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  1.60)),
            ("69.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  0.00)),
            ("70.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  7.20)),
            ("71.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  0.00)),
            ("72.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  0.30)),
            ("73.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  2.20)),
            ("74.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  8.10)),
            ("75.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  1.90)),
            ("76.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  0.10)),
            ("77.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  0.00)),
            ("78.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  0.00)),
            ("79.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  5.00)),
            ("80.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  2.00)),
            ("87.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  1.00)),
            ("88.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  0.00)),
            ("89.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  2.00)),
            ("90.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  4.00)),
            ("91.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  6.00)),
            ("92.0",    format!("/{}/Winch1.Load", dbg),       Value::Real( 12.00)),
            ("93.0",    format!("/{}/Winch1.Load", dbg),       Value::Real( 64.00)),
            ("94.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(128.00)),
            ("95.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(120.00)),
            ("96.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(133.00)),
            ("97.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(121.00)),
            ("98.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(130.00)),
            ("99.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(127.00)),
            ("100.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(123.00)),
            ("101.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(122.00)),
            ("102.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(120.00)),
            ("103.0",    format!("/{}/Winch1.Load", dbg),       Value::Real( 64.00)),
            ("104.0",    format!("/{}/Winch1.Load", dbg),       Value::Real( 32.00)),
            ("105.0",    format!("/{}/Winch1.Load", dbg),       Value::Real( 24.00)),
            ("106.0",    format!("/{}/Winch1.Load", dbg),       Value::Real( 12.00)),
            ("107.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  8.00)),
            ("108.0",    format!("/{}/Winch1.Load", dbg),       Value::Real( 17.00)),
            ("109.0",    format!("/{}/Winch1.Load", dbg),       Value::Real( 10.00)),
            ("110.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  7.00)),
            ("111.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  3.00)),
            ("112.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  6.00)),
            ("113.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  4.00)),
            ("114.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  2.00)),
            ("115.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  0.00)),
            ("116.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  4.00)),
            ("118.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  2.00)),
            ("119.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  1.00)),
            ("120.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  3.00)),
            ("121.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  0.00)),
            ("122.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  2.00)),
            ("123.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  1.00)),
            ("124.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  0.70)),
            ("125.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  0.80)),
            ("126.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  0.40)),
            ("127.0",    format!("/{}/Winch1.Load", dbg),       Value::Real(  0.30)),

            ("64.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  0.00)),
            ("65.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  3.30)),
            ("66.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  0.10)),
            ("67.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  0.00)),
            ("68.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  1.60)),
            ("69.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  0.00)),
            ("70.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  7.20)),
            ("71.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  0.00)),
            ("72.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  0.30)),
            ("73.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  2.20)),
            ("74.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  8.10)),
            ("75.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  1.90)),
            ("76.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  0.10)),
            ("77.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  0.00)),
            ("78.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  0.00)),
            ("79.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  5.00)),
            ("80.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  2.00)),
            ("87.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  1.00)),
            ("88.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  0.00)),
            ("89.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  2.00)),
            ("90.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  4.00)),
            ("91.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  6.00)),
            ("92.0",    format!("/{}/Winch2.Load", dbg),       Value::Real( 12.00)),
            ("93.0",    format!("/{}/Winch2.Load", dbg),       Value::Real( 64.00)),
            ("94.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(128.00)),
            ("95.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(120.00)),
            ("96.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(133.00)),
            ("97.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(121.00)),
            ("98.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(130.00)),
            ("99.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(127.00)),
            ("100.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(123.00)),
            ("101.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(122.00)),
            ("102.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(120.00)),
            ("103.0",    format!("/{}/Winch2.Load", dbg),       Value::Real( 64.00)),
            ("104.0",    format!("/{}/Winch2.Load", dbg),       Value::Real( 32.00)),
            ("105.0",    format!("/{}/Winch2.Load", dbg),       Value::Real( 24.00)),
            ("106.0",    format!("/{}/Winch2.Load", dbg),       Value::Real( 12.00)),
            ("107.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  8.00)),
            ("108.0",    format!("/{}/Winch2.Load", dbg),       Value::Real( 17.00)),
            ("109.0",    format!("/{}/Winch2.Load", dbg),       Value::Real( 10.00)),
            ("110.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  7.00)),
            ("111.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  3.00)),
            ("112.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  6.00)),
            ("113.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  4.00)),
            ("114.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  2.00)),
            ("115.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  0.00)),
            ("116.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  4.00)),
            ("118.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  2.00)),
            ("119.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  1.00)),
            ("120.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  3.00)),
            ("121.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  0.00)),
            ("122.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  2.00)),
            ("123.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  1.00)),
            ("124.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  0.70)),
            ("125.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  0.80)),
            ("126.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  0.40)),
            ("127.0",    format!("/{}/Winch2.Load", dbg),       Value::Real(  0.30)),

            ("64.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  0.00)),
            ("65.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  3.30)),
            ("66.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  0.10)),
            ("67.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  0.00)),
            ("68.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  1.60)),
            ("69.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  0.00)),
            ("70.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  7.20)),
            ("71.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  0.00)),
            ("72.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  0.30)),
            ("73.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  2.20)),
            ("74.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  8.10)),
            ("75.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  1.90)),
            ("76.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  0.10)),
            ("77.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  0.00)),
            ("78.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  0.00)),
            ("79.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  5.00)),
            ("80.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  2.00)),
            ("87.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  1.00)),
            ("88.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  0.00)),
            ("89.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  2.00)),
            ("90.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  4.00)),
            ("91.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  6.00)),
            ("92.0",    format!("/{}/Winch3.Load", dbg),       Value::Real( 12.00)),
            ("93.0",    format!("/{}/Winch3.Load", dbg),       Value::Real( 64.00)),
            ("94.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(128.00)),
            ("95.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(120.00)),
            ("96.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(133.00)),
            ("97.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(121.00)),
            ("98.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(130.00)),
            ("99.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(127.00)),
            ("100.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(123.00)),
            ("101.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(122.00)),
            ("102.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(120.00)),
            ("103.0",    format!("/{}/Winch3.Load", dbg),       Value::Real( 64.00)),
            ("104.0",    format!("/{}/Winch3.Load", dbg),       Value::Real( 32.00)),
            ("105.0",    format!("/{}/Winch3.Load", dbg),       Value::Real( 24.00)),
            ("106.0",    format!("/{}/Winch3.Load", dbg),       Value::Real( 12.00)),
            ("107.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  8.00)),
            ("108.0",    format!("/{}/Winch3.Load", dbg),       Value::Real( 17.00)),
            ("109.0",    format!("/{}/Winch3.Load", dbg),       Value::Real( 10.00)),
            ("110.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  7.00)),
            ("111.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  3.00)),
            ("112.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  6.00)),
            ("113.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  4.00)),
            ("114.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  2.00)),
            ("115.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  0.00)),
            ("116.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  4.00)),
            ("118.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  2.00)),
            ("119.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  1.00)),
            ("120.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  3.00)),
            ("121.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  0.00)),
            ("122.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  2.00)),
            ("123.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  1.00)),
            ("124.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  0.70)),
            ("125.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  0.80)),
            ("126.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  0.40)),
            ("127.0",    format!("/{}/Winch3.Load", dbg),       Value::Real(  0.30)),

            ("64.0",    format!("/{}/Exit", dbg),       Value::String("exit".to_owned())),
        ];
        let total_count = test_data.len();
        let receiver = Arc::new(TaskTestReceiver::new(
            dbg,
            "",
            "in-queue",
            total_count * 1000,
        ));
        services.insert(receiver.clone());
        let test_data: Vec<(String, Value)> = test_data.into_iter().map(|(_, name, value)| {
            (name, value)
        }).collect();
        let producer = Arc::new(TaskTestProducer::new(
            dbg,
            &format!("/{}/MultiQueue.in-queue", dbg),
            Duration::from_millis(10),
            services.clone(),
            &test_data,
        ));
        services.insert(producer.clone());
        thread::sleep(Duration::from_millis(100));
        services.run().unwrap();
        multi_queue.run().unwrap();
        api_client.run().unwrap();
        receiver.run().unwrap();
        log::info!("receiver runing - ok");
        for task in &tasks {
            task.run().unwrap();
        }
        log::info!("task runing - ok");
        thread::sleep(Duration::from_millis(600));
        producer.run().unwrap();
        log::info!("producer runing - ok");
        thread::sleep(Duration::from_millis(1200));
        let time = Instant::now();
        receiver.wait().unwrap();
        producer.exit();
        multi_queue.exit();
        for task in &tasks {
            task.exit();
        }
        services.exit();
        for task in tasks {
            task.wait().unwrap();
        }
        api_client.exit();
        api_client.wait().unwrap();
        producer.wait().unwrap();
        multi_queue.wait().unwrap();
        services.wait().unwrap();
        let sent = producer.sent_len();
        let result = receiver.received_len();
        println!(" elapsed: {:?}", time.elapsed());
        println!("    sent: {:?}", sent);
        println!("received: {:?}", result);
        for (i, result) in receiver.received().iter().enumerate() {
            println!("received: {}\t|\t{}\t|\t{:?}", i, result.name(), result.value());
            // assert!(result.name() == target_name, "step {} \nresult: {:?}\ntarget: {:?}", step, result.name(), target_name);
        };
        assert!(sent == total_count, "\nresult: {:?}\ntarget: {:?}", sent, total_count);
        // assert!(result >= total_count, "\nresult: {:?}\ntarget: {:?}", result, total_count);
        test_duration.exit();
        // loop {
        //     thread::sleep(Duration::from_millis(100));
        // }
    }
}

