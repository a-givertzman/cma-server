#[cfg(test)]

mod cma_recorder {
    use log::{info, trace};
    use regex::Regex;
    use std::{env, fs, sync::{Arc, Mutex, Once, RwLock}, thread, time::{Duration, Instant}};
    use testing::{entities::test_value::Value, stuff::{max_test_duration::TestDuration, wait::WaitTread}};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::{api_client_config::ApiClientConfig, multi_queue_config::MultiQueueConfig, point_config::name::Name, task_config::TaskConfig},
        core_::point::point_type::PointType,
        services::{
            api_cient::api_client::ApiClient, multi_queue::multi_queue::MultiQueue, safe_lock::SafeLock, service::service::Service, services::Services, task::{task::Task, task_test_receiver::TaskTestReceiver}
        },
        tests::unit::services::task::task_test_producer::TaskTestProducer,
        // tests::unit::services::cma_recorder::task_test_producer::TaskTestProducer
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
    fn operating_metric() {
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
        let path = "./src/tests/unit/services/task/cma_recorder/operating-metric.yaml";
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
            ("00.-4",    format!("/{}/Load.Nom", self_id),   Value::Real(  150.00),     0,       00.0000,                0.0f32),
            ("00.-3",    format!("/{}/Winch1.Load.Nom", self_id),   Value::Real(  150.00),     0,       00.0000,                0.0f32),
            ("00.-2",    format!("/{}/Winch2.Load.Nom", self_id),   Value::Real(  150.00),     0,       00.0000,                0.0f32),
            ("00.-1",    format!("/{}/Winch3.Load.Nom", self_id),   Value::Real(  150.00),     0,       00.0000,                0.0f32),
            ("00.0",    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       00.0000,                0.0),
            ("01.0",    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       00.0000,                0.0),
            ("02.0",    format!("/{}/Load", self_id),       Value::Real(  3.30),       0,       00.0000,                0.4125),
            ("03.0",    format!("/{}/Load", self_id),       Value::Real(  0.10),       0,       00.0000,                0.3734375),
            ("04.0",    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       00.0000,                0.3267578125),
            ("05.0",    format!("/{}/Load", self_id),       Value::Real(  1.60),       0,       00.0000,                0.4859130859375),
            ("06.0",    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       00.0000,                0.425173950195313),
            ("07.0",    format!("/{}/Load", self_id),       Value::Real(  7.20),       0,       00.0000,                1.2720272064209),
            ("08.0",    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       00.0000,                1.11302380561829),
            ("09.0",    format!("/{}/Load", self_id),       Value::Real(  0.30),       0,       00.0000,                1.011395829916),
            ("10.0",    format!("/{}/Load", self_id),       Value::Real(  2.20),       0,       00.0000,                1.1599713511765),
            ("11.0",    format!("/{}/Load", self_id),       Value::Real(  8.10),       0,       00.0000,                2.02747493227944),
            ("12.0",    format!("/{}/Load", self_id),       Value::Real(  1.90),       0,       00.0000,                2.01154056574451),
            ("13.0",    format!("/{}/Load", self_id),       Value::Real(  0.10),       0,       00.0000,                1.77259799502644),
            ("14.0",    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       00.0000,                1.55102324564814),
            ("15.0",    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       00.0000,                1.35714533994212),
            ("16.0",    format!("/{}/Load", self_id),       Value::Real(  5.00),       0,       00.0000,                1.81250217244936),
            ("17.0",    format!("/{}/Load", self_id),       Value::Real(  2.00),       0,       00.0000,                1.83593940089319),
            ("17.1",    format!("/{}/Winch1.Load.Limiter.Trip", self_id),       Value::Bool(true),       0,       00.0000,                1.83593940089319),
            ("17.2",    format!("/{}/Winch1.Load.Limiter.Trip", self_id),       Value::Bool(false),       0,       00.0000,                1.83593940089319),
            ("17.3",    format!("/{}/Winch2.Load.Limiter.Trip", self_id),       Value::Bool(true),       0,       00.0000,                1.83593940089319),
            ("17.4",    format!("/{}/Winch2.Load.Limiter.Trip", self_id),       Value::Bool(false),       0,       00.0000,                1.83593940089319),
            ("17.5",    format!("/{}/Winch3.Load.Limiter.Trip", self_id),       Value::Bool(true),       0,       00.0000,                1.83593940089319),
            ("17.6",    format!("/{}/Winch3.Load.Limiter.Trip", self_id),       Value::Bool(false),       0,       00.0000,                1.83593940089319),
            ("18.0",    format!("/{}/Load", self_id),       Value::Real(  1.00),       0,       00.0000,                1.73144697578154),
            ("19.0",    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       00.0000,                1.51501610380885),
            ("20.0",    format!("/{}/Load", self_id),       Value::Real(  2.00),       0,       00.0000,                1.57563909083274),
            ("21.0",    format!("/{}/Load", self_id),       Value::Real(  4.00),       0,       00.0000,                1.87868420447865),
            ("22.0",    format!("/{}/Load", self_id),       Value::Real(  6.00),       0,       00.0000,                2.39384867891882),
            ("23.0",    format!("/{}/Load", self_id),       Value::Real( 12.00),       0,       00.0000,                3.59461759405396),
            ("24.0",    format!("/{}/Load", self_id),       Value::Real( 64.00),       1,       11.1452903947972,       11.1452903947972),
            ("25.0",    format!("/{}/Load", self_id),       Value::Real(128.00),       1,       25.7521290954476,       25.7521290954476),
            ("26.0",    format!("/{}/Load", self_id),       Value::Real(120.00),       1,       37.5331129585166,       37.5331129585166),
            ("27.0",    format!("/{}/Load", self_id),       Value::Real(133.00),       1,       49.466473838702,       49.466473838702),
            ("28.0",    format!("/{}/Load", self_id),       Value::Real(121.00),       1,       58.4081646088643,       58.4081646088643),
            // ("29.0",    format!("/{}/Load", self_id),       Value::Real(330.00),       1,       67.3571440327563,       67.3571440327563),
            ("29.0",    format!("/{}/Load", self_id),       Value::Real(130.00),       1,       67.3571440327563,       67.3571440327563),
            ("30.0",    format!("/{}/Load", self_id),       Value::Real(127.00),       1,       67.3571440327563,       74.8125010286617),
            ("31.0",    format!("/{}/Load", self_id),       Value::Real(123.00),       1,       80.835938400079,       80.835938400079),
            ("32.0",    format!("/{}/Load", self_id),       Value::Real(122.00),       1,       80.835938400079,       85.9814461000691),
            ("33.0",    format!("/{}/Load", self_id),       Value::Real(120.00),       1,       90.2337653375605,       90.2337653375605),
            ("34.0",    format!("/{}/Load", self_id),       Value::Real( 64.00),       1,       90.2337653375605,       86.9545446703654),
            ("35.0",    format!("/{}/Load", self_id),       Value::Real( 32.00),       1,       80.0852265865698,       80.0852265865698),
            ("36.0",    format!("/{}/Load", self_id),       Value::Real( 24.00),       1,       80.0852265865698,       73.0745732632485),
            ("37.0",    format!("/{}/Load", self_id),       Value::Real( 12.00),       1,       65.4402516053425,       65.4402516053425),
            ("38.0",    format!("/{}/Load", self_id),       Value::Real(  8.00),       1,       65.4402516053425,       58.2602201546747),
            ("39.0",    format!("/{}/Load", self_id),       Value::Real( 17.00),       1,       53.1026926353403,       53.1026926353403),
            ("40.0",    format!("/{}/Load", self_id),       Value::Real( 10.00),       1,       53.1026926353403,       47.7148560559228),
            ("41.0",    format!("/{}/Load", self_id),       Value::Real(  7.00),       1,       42.6254990489324,       42.6254990489324),
            ("42.0",    format!("/{}/Load", self_id),       Value::Real(  3.00),       1,       42.6254990489324,       37.6723116678159),
            ("43.0",    format!("/{}/Load", self_id),       Value::Real(  6.00),       1,       33.7132727093389,       33.7132727093389),
            ("44.0",    format!("/{}/Load", self_id),       Value::Real(  4.00),       1,       33.7132727093389,       29.9991136206715),
            ("45.0",    format!("/{}/Load", self_id),       Value::Real(  2.00),       1,       33.7132727093389,       26.4992244180876),
            ("46.0",    format!("/{}/Load", self_id),       Value::Real(  0.00),       1,       23.1868213658266,       23.1868213658266),
            ("47.0",    format!("/{}/Load", self_id),       Value::Real(  4.00),       1,       23.1868213658266,       20.7884686950983),
            ("47.1",    format!("/{}/Winch1.Load.Limiter.Trip", self_id),       Value::Bool(true),       1,       23.1868213658266,       20.7884686950983),
            ("48.0",    format!("/{}/Load", self_id),       Value::Real(  2.00),       1,       23.1868213658266,       18.439910108211),
            ("49.0",    format!("/{}/Load", self_id),       Value::Real(  1.00),       1,       23.1868213658266,       16.2599213446847),
            ("50.0",    format!("/{}/Load", self_id),       Value::Real(  3.00),       1,       14.6024311765991,       14.6024311765991),
            ("51.0",    format!("/{}/Load", self_id),       Value::Real(  0.00),       1,       14.6024311765991,       12.7771272795242),
            ("52.0",    format!("/{}/Load", self_id),       Value::Real(  2.00),       1,       14.6024311765991,       11.4299863695837),
            ("53.0",    format!("/{}/Load", self_id),       Value::Real(  1.00),       1,       14.6024311765991,       10.1262380733857),
            ("54.0",    format!("/{}/Load", self_id),       Value::Real(  0.70),       1,       14.6024311765991,       8.94795831421249),
            ("55.0",    format!("/{}/Load", self_id),       Value::Real(  0.80),       1,       14.6024311765991,       7.92946352493593),
            ("56.0",    format!("/{}/Load", self_id),       Value::Real(  0.40),       0,       6.98828058431894,       6.98828058431894),
            ("57.0",    format!("/{}/Load", self_id),       Value::Real(  0.30),       0,       6.98828058431894,       6.15224551127907),
            ("58.0",    format!("/{}/Load", self_id),       Value::Real(  0.20),       0,       6.98828058431894,       5.40821482236919),
            ("59.0",    format!("/{}/Load", self_id),       Value::Real(  0.10),       0,       6.98828058431894,       4.74468796957304),
            ("60.0",    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       6.98828058431894,       4.15160197337641),
            ("61.0",    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       6.98828058431894,       3.63265172670436),
            ("62.0",    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       6.98828058431894,       3.17857026086631),
            ("63.0",    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       6.98828058431894,       2.78124897825802),

            ("64.0",    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       00.0000,                0.0),
            ("65.0",    format!("/{}/Load", self_id),       Value::Real(  3.30),       0,       00.0000,                0.4125),
            ("66.0",    format!("/{}/Load", self_id),       Value::Real(  0.10),       0,       00.0000,                0.3734375),
            ("67.0",    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       00.0000,                0.3267578125),
            ("68.0",    format!("/{}/Load", self_id),       Value::Real(  1.60),       0,       00.0000,                0.4859130859375),
            ("69.0",    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       00.0000,                0.425173950195313),
            ("70.0",    format!("/{}/Load", self_id),       Value::Real(  7.20),       0,       00.0000,                1.2720272064209),
            ("71.0",    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       00.0000,                1.11302380561829),
            ("72.0",    format!("/{}/Load", self_id),       Value::Real(  0.30),       0,       00.0000,                1.011395829916),
            ("73.0",    format!("/{}/Load", self_id),       Value::Real(  2.20),       0,       00.0000,                1.1599713511765),
            ("74.0",    format!("/{}/Load", self_id),       Value::Real(  8.10),       0,       00.0000,                2.02747493227944),
            ("75.0",    format!("/{}/Load", self_id),       Value::Real(  1.90),       0,       00.0000,                2.01154056574451),
            ("76.0",    format!("/{}/Load", self_id),       Value::Real(  0.10),       0,       00.0000,                1.77259799502644),
            ("77.0",    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       00.0000,                1.55102324564814),
            ("78.0",    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       00.0000,                1.35714533994212),
            ("79.0",    format!("/{}/Load", self_id),       Value::Real(  5.00),       0,       00.0000,                1.81250217244936),
            ("80.0",    format!("/{}/Load", self_id),       Value::Real(  2.00),       0,       00.0000,                1.83593940089319),
            ("81.1",    format!("/{}/Winch1.Load.Limiter.Trip", self_id),       Value::Bool(true),       0,       00.0000,                1.83593940089319),
            ("82.2",    format!("/{}/Winch1.Load.Limiter.Trip", self_id),       Value::Bool(false),       0,       00.0000,                1.83593940089319),
            ("83.3",    format!("/{}/Winch2.Load.Limiter.Trip", self_id),       Value::Bool(true),       0,       00.0000,                1.83593940089319),
            ("84.4",    format!("/{}/Winch2.Load.Limiter.Trip", self_id),       Value::Bool(false),       0,       00.0000,                1.83593940089319),
            ("85.5",    format!("/{}/Winch3.Load.Limiter.Trip", self_id),       Value::Bool(true),       0,       00.0000,                1.83593940089319),
            ("86.6",    format!("/{}/Winch3.Load.Limiter.Trip", self_id),       Value::Bool(false),       0,       00.0000,                1.83593940089319),
            ("87.0",    format!("/{}/Load", self_id),       Value::Real(  1.00),       0,       00.0000,                1.73144697578154),
            ("88.0",    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       00.0000,                1.51501610380885),
            ("89.0",    format!("/{}/Load", self_id),       Value::Real(  2.00),       0,       00.0000,                1.57563909083274),
            ("90.0",    format!("/{}/Load", self_id),       Value::Real(  4.00),       0,       00.0000,                1.87868420447865),
            ("91.0",    format!("/{}/Load", self_id),       Value::Real(  6.00),       0,       00.0000,                2.39384867891882),
            ("92.0",    format!("/{}/Load", self_id),       Value::Real( 12.00),       0,       00.0000,                3.59461759405396),
            ("93.0",    format!("/{}/Load", self_id),       Value::Real( 64.00),       1,       11.1452903947972,       11.1452903947972),
            ("94.0",    format!("/{}/Load", self_id),       Value::Real(128.00),       1,       25.7521290954476,       25.7521290954476),
            ("95.0",    format!("/{}/Load", self_id),       Value::Real(120.00),       1,       37.5331129585166,       37.5331129585166),
            ("96.0",    format!("/{}/Load", self_id),       Value::Real(133.00),       1,       49.466473838702,       49.466473838702),
            ("97.0",    format!("/{}/Load", self_id),       Value::Real(121.00),       1,       58.4081646088643,       58.4081646088643),
            ("98.0",    format!("/{}/Load", self_id),       Value::Real(130.00),       1,       67.3571440327563,       67.3571440327563),
            ("99.0",    format!("/{}/Load", self_id),       Value::Real(127.00),       1,       67.3571440327563,       74.8125010286617),
            ("100.0",    format!("/{}/Load", self_id),       Value::Real(123.00),       1,       80.835938400079,       80.835938400079),
            ("101.0",    format!("/{}/Load", self_id),       Value::Real(122.00),       1,       80.835938400079,       85.9814461000691),
            ("102.0",    format!("/{}/Load", self_id),       Value::Real(120.00),       1,       90.2337653375605,       90.2337653375605),
            ("103.0",    format!("/{}/Load", self_id),       Value::Real( 64.00),       1,       90.2337653375605,       86.9545446703654),
            ("104.0",    format!("/{}/Load", self_id),       Value::Real( 32.00),       1,       80.0852265865698,       80.0852265865698),
            ("105.0",    format!("/{}/Load", self_id),       Value::Real( 24.00),       1,       80.0852265865698,       73.0745732632485),
            ("106.0",    format!("/{}/Load", self_id),       Value::Real( 12.00),       1,       65.4402516053425,       65.4402516053425),
            ("107.0",    format!("/{}/Load", self_id),       Value::Real(  8.00),       1,       65.4402516053425,       58.2602201546747),
            ("108.0",    format!("/{}/Load", self_id),       Value::Real( 17.00),       1,       53.1026926353403,       53.1026926353403),
            ("109.0",    format!("/{}/Load", self_id),       Value::Real( 10.00),       1,       53.1026926353403,       47.7148560559228),
            ("110.0",    format!("/{}/Load", self_id),       Value::Real(  7.00),       1,       42.6254990489324,       42.6254990489324),
            ("111.0",    format!("/{}/Load", self_id),       Value::Real(  3.00),       1,       42.6254990489324,       37.6723116678159),
            ("112.0",    format!("/{}/Load", self_id),       Value::Real(  6.00),       1,       33.7132727093389,       33.7132727093389),
            ("113.0",    format!("/{}/Load", self_id),       Value::Real(  4.00),       1,       33.7132727093389,       29.9991136206715),
            ("114.0",    format!("/{}/Load", self_id),       Value::Real(  2.00),       1,       33.7132727093389,       26.4992244180876),
            ("115.0",    format!("/{}/Load", self_id),       Value::Real(  0.00),       1,       23.1868213658266,       23.1868213658266),
            ("116.0",    format!("/{}/Load", self_id),       Value::Real(  4.00),       1,       23.1868213658266,       20.7884686950983),
            ("117.1",    format!("/{}/Winch1.Load.Limiter.Trip", self_id),       Value::Bool(true),       1,       23.1868213658266,       20.7884686950983),
            ("118.0",    format!("/{}/Load", self_id),       Value::Real(  2.00),       1,       23.1868213658266,       18.439910108211),
            ("119.0",    format!("/{}/Load", self_id),       Value::Real(  1.00),       1,       23.1868213658266,       16.2599213446847),
            ("120.0",    format!("/{}/Load", self_id),       Value::Real(  3.00),       1,       14.6024311765991,       14.6024311765991),
            ("121.0",    format!("/{}/Load", self_id),       Value::Real(  0.00),       1,       14.6024311765991,       12.7771272795242),
            ("122.0",    format!("/{}/Load", self_id),       Value::Real(  2.00),       1,       14.6024311765991,       11.4299863695837),
            ("123.0",    format!("/{}/Load", self_id),       Value::Real(  1.00),       1,       14.6024311765991,       10.1262380733857),
            ("124.0",    format!("/{}/Load", self_id),       Value::Real(  0.70),       1,       14.6024311765991,       8.94795831421249),
            ("125.0",    format!("/{}/Load", self_id),       Value::Real(  0.80),       1,       14.6024311765991,       7.92946352493593),
            ("126.0",    format!("/{}/Load", self_id),       Value::Real(  0.40),       0,       6.98828058431894,       6.98828058431894),
            ("127.0",    format!("/{}/Load", self_id),       Value::Real(  0.30),       0,       6.98828058431894,       6.15224551127907),
            ("128.0",    format!("/{}/Exit", self_id),       Value::String("exit".to_owned()),       0,       6.98828058431894,       2.78124897825802),

            ("64.0",    format!("/{}/Exit", self_id),       Value::String("exit".to_owned()),       0,       6.98828058431894,       2.78124897825802),
        ];
        let total_count = test_data.len();
        let (len, sum) = test_data.iter().fold((0, 0.0), |(mut len, mut sum), (i, _name, value, _op_cycle, _thrd, _smooth)| {
            if _name == &format!("/{}/Load", self_id) {
                if _op_cycle > &0 {
                    len += 1;
                    sum += value.as_real();
                }
            }
            println!("{}\taverage: {}", i, sum / (len as f32));
            (len, sum)
        });
        let target_average = sum / (len as f32);
        let target_thrd: Vec<(&str, f32)> = test_data.iter().filter_map(|(i, _name, _value, _op_cycle, thrd, _smooth)| {
            Some((*i, thrd.clone()))
        }).collect();
        let target_smooth: Vec<(&str, f32)> = test_data.iter().filter_map(|(i, _name, _value, _op_cycle, _thrd, smooth)| {
            Some((*i, smooth.clone()))
        }).collect();
        let target_op_cycle: Vec<(&str, i32)> = test_data.iter().filter_map(|(i, _name, _value, op_cycle, _thrd, _smooth)| {
            Some((*i, op_cycle.clone()))
        }).collect();
        let target_thrd_count = target_thrd.len();
        let target_smooth_count = target_smooth.len();
        let receiver = Arc::new(Mutex::new(TaskTestReceiver::new(
            self_id,
            "",
            "in-queue",
            total_count * 1000,
        )));
        services.wlock(self_id).insert(receiver.clone());
        let test_data: Vec<(String, Value)> = test_data.into_iter().map(|(_, name, value, _, _, _)| {
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
        thread::sleep(Duration::from_millis(300));
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
        println!("target smooth   : {:?}", target_smooth_count);
        println!("target threshold: {:?}", target_thrd_count);
        for (i, result) in receiver.lock().unwrap().received().lock().unwrap().iter().enumerate() {
            println!("received: {}\t|\t{}\t|\t{:?}", i, result.name(), result.value());
            // assert!(result.name() == target_name, "step {} \nresult: {:?}\ntarget: {:?}", step, result.name(), target_name);
        };
        // assert!(sent == total_count, "\nresult: {:?}\ntarget: {:?}", sent, total_count);
        // assert!(result >= total_count, "\nresult: {:?}\ntarget: {:?}", result, total_count);
        let target_received = target_received();
        let received: Vec<String> = receiver
            .lock().unwrap().received()
            .lock().unwrap().clone().iter()
            .map(|p| p.to_string().as_string().value)
            .collect();
        for target in target_received {
            let target = Regex::new(target).unwrap();
            let mut matched = false;
            for result in &received {
                if target.is_match(result) {
                    matched = true;
                    break;
                }
            }
            assert!(matched, "\n results does not contains required value '{}'", target);
        }
        let smooth: Vec<PointType> = receiver.lock().unwrap().received().lock().unwrap().iter().cloned().filter(|point| {
            point.name() == format!("/{}/RecorderTask/Smooth", self_id)
        }).collect();
        for (i, result) in smooth.iter().enumerate() {
            println!("smooth: {}\t|\t{}\t|\t{:?}", i, result.name(), result.value());
        };
        let thrd: Vec<PointType> = receiver.lock().unwrap().received().lock().unwrap().iter().cloned().filter(|point| {
            point.name() == format!("/{}/RecorderTask/Threshold", self_id)
        }).collect();
        for (i, result) in thrd.iter().enumerate() {
            println!("threshold: {}\t|\t{}\t|\t{:?}", i, result.name(), result.value());
        };
        let op_cycle_is_active: Vec<PointType> = receiver.lock().unwrap().received().lock().unwrap().iter().cloned().filter(|point| {
            point.name() == format!("/{}/RecorderTask/OpCycleIsActive", self_id)
        }).collect();
        for (i, result) in op_cycle_is_active.iter().enumerate() {
            println!("op cycle: {}\t|\t{}\t|\t{:?}", i, result.name(), result.value());
        };
        let op_cycle_sql: Vec<PointType> = receiver.lock().unwrap().received().lock().unwrap().iter().cloned().filter(|point| {
            point.name() == format!("/{}/RecorderTask/OpCycleSql", self_id)
        }).collect();
        for (i, result) in op_cycle_sql.iter().enumerate() {
            println!("op cycle SQL: {}\t|\t{}\t|\t{:?}", i, result.name(), result.value());
        };
        println!("target average: {}", target_average);
        let target_name = "/AppTest/RecorderTask/OpCycleIsActive";
        for (i, result) in op_cycle_is_active.iter().enumerate() {
            let (step, _target) = target_op_cycle[i].clone();
            // assert!(result.value().as_bool() == (target > 0), "step {} \nresult op cycle: {:?}\ntarget op cycle: {:?}", step, result.value().as_bool(), target);
            assert!(result.name() == target_name, "step {} \nresult: {:?}\ntarget: {:?}", step, result.name(), target_name);
        };
        test_duration.exit();
        // loop {
        //     thread::sleep(Duration::from_millis(100));
        // }
    }
    ///
    /// Values must to be in the 'received'
    fn target_received() -> [&'static str; 6] {
        [
            r"insert into public\.rec_operating_cycle \(id, timestamp_start, timestamp_stop, alarm_class\) values \(\d+, '\d+-\d+-\d+ \d+:\d+:\d+\.\d+ UTC', '\d+-\d+-\d+ \d+:\d+:\d+\.\d+ UTC', \d+\);",
            r"insert into public\.rec_operating_metric \(operating_cycle_id, metric_id, value\) values \(\d+, 'average_load', \d+.\d+\);",
            r"insert into public\.rec_operating_metric \(operating_cycle_id, metric_id, value\) values \(\d+, 'max_load', \d+.\d+\);",
            r"insert into public\.rec_operating_cycle \(id, timestamp_start, timestamp_stop, alarm_class\) values \(\d+, '\d+-\d+-\d+ \d+:\d+:\d+\.\d+ UTC', '\d+-\d+-\d+ \d+:\d+:\d+\.\d+ UTC', \d+\);",
            r"insert into public\.rec_operating_metric \(operating_cycle_id, metric_id, value\) values \(\d+, 'average_load', \d+.\d+\);",
            r"insert into public\.rec_operating_metric \(operating_cycle_id, metric_id, value\) values \(\d+, 'max_load', \d+.\d+\);",
        ]
    }
}

