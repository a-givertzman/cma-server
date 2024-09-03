#[cfg(test)]

mod cma_recorder {
    use log::{debug, info, trace};
    use sal_sync::services::{entity::{name::Name, point::point::Point}, retain::{retain_conf::RetainConf, retain_point_conf::RetainPointConf}, service::service::Service};
    use std::{env, sync::{Arc, Once, RwLock}, thread, time::{Duration, Instant}};
    use testing::{entities::test_value::Value, stuff::{max_test_duration::TestDuration, wait::WaitTread}};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::{multi_queue_config::MultiQueueConfig, task_config::TaskConfig}, core_::aprox_eq::aprox_eq::AproxEq,
        services::{multi_queue::multi_queue::MultiQueue, safe_lock::rwlock::SafeLock, services::Services, task::{task::Task, task_test_receiver::TaskTestReceiver}},
        tests::unit::services::task::cma_recorder::task_test_producer::TaskTestProducer
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
    /// Testing detection of the operating cycle
    /// - smoothing crane load value
    /// - filtering (thresholding filter) smoothed value
    #[test]
    fn detect_operating_cycle() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "App";
        let self_name = Name::new("", self_id);
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(20));
        test_duration.run().unwrap();
        //
        // can be changed
        trace!("dir: {:?}", env::current_dir());
        let services = Arc::new(RwLock::new(Services::new(self_id, RetainConf::new(
            Some("assets/testing/retain/"),
            Some(RetainPointConf::new("point/id.json", None))
        ))));
        let config = TaskConfig::from_yaml(
            &self_name,
            &serde_yaml::from_str(r"
                service Task RecorderTask:
                    cycle: 1 ms
                    in queue recv-queue:
                        max-length: 10000
                    subscribe:
                        /App/MultiQueue:                    # - multicast subscription to the MultiQueue
                            {cot: Inf}: []                      #   - on all points having Cot::Inf

                    #
                    # The nominal value of the crane load
                    let loadNom:
                        # input: const real 150
                        input: point real '/App/Load.Nom'

                    #
                    # 5 % of the nominal crane load - used for Op Cycle detection
                    let opCycleThreshold:
                        input fn Mul:              
                            input1: const real 0.05
                            input2: loadNom

                    let opCycleIsActive:
                        input fn Export:
                            send-to: /App/TaskTestReceiver.in-queue
                            conf point OpCycle:
                                type: 'Bool'
                            input fn Ge:
                                input2: opCycleThreshold
                                input1 fn Export:
                                    send-to: /App/TaskTestReceiver.in-queue
                                    conf point Threshold:
                                        type: 'Real'
                                    input fn Threshold:                 # Triggering threshold of the operating cycle detection function on the input value based on the nominal value
                                        threshold: opCycleThreshold
                                        input fn Export:
                                            send-to: /App/TaskTestReceiver.in-queue
                                            conf point Smooth:
                                                type: 'Real'
                                            input fn Smooth:
                                                factor: const real 0.125
                                                input: point real '/App/Load'
            ").unwrap(),
        );
        trace!("config: {:?}", config);
        debug!("Task config points: {:#?}", config.points());
        let task = Arc::new(RwLock::new(Task::new(config, services.clone())));
        debug!("Task points: {:#?}", task.read().unwrap().points());
        services.wlock(self_id).insert(task.clone());
        let conf = MultiQueueConfig::from_yaml(
            self_id,
            &serde_yaml::from_str(r"service MultiQueue:
                in queue in-queue:
                    max-length: 10000
            ").unwrap(),
        );
        let multi_queue = Arc::new(RwLock::new(MultiQueue::new(conf, services.clone())));
        services.wlock(self_id).insert(multi_queue.clone());
        let test_data = vec![
        //  step    nape                                input                    OpCycle    target_thrh             target_smooth
            (00,    format!("/{}/Load.Nom", self_id),   Value::Real(  150.00),     0,       00.0000,                0.0f32),
            (00,    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       00.0000,                0.0),
            (01,    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       00.0000,                0.0),
            (02,    format!("/{}/Load", self_id),       Value::Real(  3.30),       0,       00.0000,                0.4125),
            (03,    format!("/{}/Load", self_id),       Value::Real(  0.10),       0,       00.0000,                0.3734375),
            (04,    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       00.0000,                0.3267578125),
            (05,    format!("/{}/Load", self_id),       Value::Real(  1.60),       0,       00.0000,                0.4859130859375),
            (06,    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       00.0000,                0.425173950195313),
            (07,    format!("/{}/Load", self_id),       Value::Real(  7.20),       0,       00.0000,                1.2720272064209),
            (08,    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       00.0000,                1.11302380561829),
            (09,    format!("/{}/Load", self_id),       Value::Real(  0.30),       0,       00.0000,                1.011395829916),
            (10,    format!("/{}/Load", self_id),       Value::Real(  2.20),       0,       00.0000,                1.1599713511765),
            (11,    format!("/{}/Load", self_id),       Value::Real(  8.10),       0,       00.0000,                2.02747493227944),
            (12,    format!("/{}/Load", self_id),       Value::Real(  1.90),       0,       00.0000,                2.01154056574451),
            (13,    format!("/{}/Load", self_id),       Value::Real(  0.10),       0,       00.0000,                1.77259799502644),
            (14,    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       00.0000,                1.55102324564814),
            (15,    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       00.0000,                1.35714533994212),
            (16,    format!("/{}/Load", self_id),       Value::Real(  5.00),       0,       00.0000,                1.81250217244936),
            (17,    format!("/{}/Load", self_id),       Value::Real(  2.00),       0,       00.0000,                1.83593940089319),
            (18,    format!("/{}/Load", self_id),       Value::Real(  1.00),       0,       00.0000,                1.73144697578154),
            (19,    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       00.0000,                1.51501610380885),
            (20,    format!("/{}/Load", self_id),       Value::Real(  2.00),       0,       00.0000,                1.57563909083274),
            (21,    format!("/{}/Load", self_id),       Value::Real(  4.00),       0,       00.0000,                1.87868420447865),
            (22,    format!("/{}/Load", self_id),       Value::Real(  6.00),       0,       00.0000,                2.39384867891882),
            (23,    format!("/{}/Load", self_id),       Value::Real( 12.00),       0,       00.0000,                3.59461759405396),
            (24,    format!("/{}/Load", self_id),       Value::Real( 64.00),       1,       11.1452903947972,       11.1452903947972),
            (25,    format!("/{}/Load", self_id),       Value::Real(128.00),       1,       25.7521290954476,       25.7521290954476),
            (26,    format!("/{}/Load", self_id),       Value::Real(120.00),       1,       37.5331129585166,       37.5331129585166),
            (27,    format!("/{}/Load", self_id),       Value::Real(133.00),       1,       49.466473838702,       49.466473838702),
            (28,    format!("/{}/Load", self_id),       Value::Real(121.00),       1,       58.4081646088643,       58.4081646088643),
            (29,    format!("/{}/Load", self_id),       Value::Real(130.00),       1,       67.3571440327563,       67.3571440327563),
            (30,    format!("/{}/Load", self_id),       Value::Real(127.00),       1,       67.3571440327563,       74.8125010286617),
            (31,    format!("/{}/Load", self_id),       Value::Real(123.00),       1,       80.835938400079,       80.835938400079),
            (32,    format!("/{}/Load", self_id),       Value::Real(122.00),       1,       80.835938400079,       85.9814461000691),
            (33,    format!("/{}/Load", self_id),       Value::Real(120.00),       1,       90.2337653375605,       90.2337653375605),
            (34,    format!("/{}/Load", self_id),       Value::Real( 64.00),       1,       90.2337653375605,       86.9545446703654),
            (35,    format!("/{}/Load", self_id),       Value::Real( 32.00),       1,       80.0852265865698,       80.0852265865698),
            (36,    format!("/{}/Load", self_id),       Value::Real( 24.00),       1,       80.0852265865698,       73.0745732632485),
            (37,    format!("/{}/Load", self_id),       Value::Real( 12.00),       1,       65.4402516053425,       65.4402516053425),
            (38,    format!("/{}/Load", self_id),       Value::Real(  8.00),       1,       65.4402516053425,       58.2602201546747),
            (39,    format!("/{}/Load", self_id),       Value::Real( 17.00),       1,       53.1026926353403,       53.1026926353403),
            (40,    format!("/{}/Load", self_id),       Value::Real( 10.00),       1,       53.1026926353403,       47.7148560559228),
            (41,    format!("/{}/Load", self_id),       Value::Real(  7.00),       1,       42.6254990489324,       42.6254990489324),
            (42,    format!("/{}/Load", self_id),       Value::Real(  3.00),       1,       42.6254990489324,       37.6723116678159),
            (43,    format!("/{}/Load", self_id),       Value::Real(  6.00),       1,       33.7132727093389,       33.7132727093389),
            (44,    format!("/{}/Load", self_id),       Value::Real(  4.00),       1,       33.7132727093389,       29.9991136206715),
            (45,    format!("/{}/Load", self_id),       Value::Real(  2.00),       1,       33.7132727093389,       26.4992244180876),
            (46,    format!("/{}/Load", self_id),       Value::Real(  0.00),       1,       23.1868213658266,       23.1868213658266),
            (47,    format!("/{}/Load", self_id),       Value::Real(  4.00),       1,       23.1868213658266,       20.7884686950983),
            (48,    format!("/{}/Load", self_id),       Value::Real(  2.00),       1,       23.1868213658266,       18.439910108211),
            (49,    format!("/{}/Load", self_id),       Value::Real(  1.00),       1,       23.1868213658266,       16.2599213446847),
            (50,    format!("/{}/Load", self_id),       Value::Real(  3.00),       1,       14.6024311765991,       14.6024311765991),
            (51,    format!("/{}/Load", self_id),       Value::Real(  0.00),       1,       14.6024311765991,       12.7771272795242),
            (52,    format!("/{}/Load", self_id),       Value::Real(  2.00),       1,       14.6024311765991,       11.4299863695837),
            (53,    format!("/{}/Load", self_id),       Value::Real(  1.00),       1,       14.6024311765991,       10.1262380733857),
            (54,    format!("/{}/Load", self_id),       Value::Real(  0.70),       1,       14.6024311765991,       8.94795831421249),
            (55,    format!("/{}/Load", self_id),       Value::Real(  0.80),       1,       14.6024311765991,       7.92946352493593),
            (56,    format!("/{}/Load", self_id),       Value::Real(  0.40),       0,       6.98828058431894,       6.98828058431894),
            (57,    format!("/{}/Load", self_id),       Value::Real(  0.30),       0,       6.98828058431894,       6.15224551127907),
            (58,    format!("/{}/Load", self_id),       Value::Real(  0.20),       0,       6.98828058431894,       5.40821482236919),
            (59,    format!("/{}/Load", self_id),       Value::Real(  0.10),       0,       6.98828058431894,       4.74468796957304),
            (60,    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       6.98828058431894,       4.15160197337641),
            (61,    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       6.98828058431894,       3.63265172670436),
            (62,    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       6.98828058431894,       3.17857026086631),
            (63,    format!("/{}/Load", self_id),       Value::Real(  0.00),       0,       6.98828058431894,       2.78124897825802),
        ];
        let total_count = test_data.len();
        let target_thrd: Vec<(i32, f32)> = test_data.iter().filter_map(|(i, _name, _value, _op_cycle, thrd, _smooth)| {
            if *_name == format!("/{}/Load", self_id) {
                Some((*i, thrd.clone()))
            } else {
                None
            }
        }).collect();
        let target_smooth: Vec<(i32, f32)> = test_data.iter().filter_map(|(i, _name, _value, _op_cycle, _thrd, smooth)| {
            if *_name == format!("/{}/Load", self_id) {
                Some((*i, smooth.clone()))
            } else {
                None
            }
        }).collect();
        let target_op_cycle: Vec<(i32, i32)> = test_data.iter().filter_map(|(i, _name, _value, op_cycle, _thrd, _smooth)| {
            if *_name == format!("/{}/Load", self_id) {
                Some((*i, op_cycle.clone()))
            } else {
                None
            }
        }).collect();
        let target_thrd_count = target_thrd.len();
        let target_smooth_count = target_smooth.len();
        let receiver = Arc::new(RwLock::new(TaskTestReceiver::new(
            self_id,
            "",
            "in-queue",
            total_count * 2,
        )));
        services.wlock(self_id).insert(receiver.clone());
        let test_data: Vec<(String, Value)> = test_data.into_iter().map(|(_, name, value, _, _, _)| {
            (name, value)
        }).collect();
        let producer = Arc::new(RwLock::new(TaskTestProducer::new(
            self_id,
            &format!("/{}/MultiQueue.in-queue", self_id),
            Duration::ZERO,
            services.clone(),
            &test_data,
        )));
        services.wlock(self_id).insert(producer.clone());
        let services_handle = services.wlock(self_id).run().unwrap();
        let multi_queue_handle = multi_queue.write().unwrap().run().unwrap();
        let receiver_handle = receiver.write().unwrap().run().unwrap();
        info!("receiver runing - ok");
        let task_handle = task.write().unwrap().run().unwrap();
        info!("task runing - ok");
        thread::sleep(Duration::from_millis(100));
        let producer_handle = producer.write().unwrap().run().unwrap();
        info!("producer runing - ok");
        let time = Instant::now();
        receiver_handle.wait().unwrap();
        producer.read().unwrap().exit();
        task.read().unwrap().exit();
        task_handle.wait().unwrap();
        producer_handle.wait().unwrap();
        multi_queue.read().unwrap().exit();
        multi_queue_handle.wait().unwrap();
        services.rlock(self_id).exit();
        services_handle.wait().unwrap();
        let sent = producer.read().unwrap().sent().read().unwrap().len();
        let result = receiver.read().unwrap().received().read().unwrap().len();
        println!(" elapsed: {:?}", time.elapsed());
        println!("    sent: {:?}", sent);
        println!("received: {:?}", result);
        println!("target smooth   : {:?}", target_smooth_count);
        println!("target threshold: {:?}", target_thrd_count);

        for (i, result) in receiver.read().unwrap().received().read().unwrap().iter().enumerate() {
            println!("received: {}\t|\t{}\t|\t{:?}", i, result.name(), result.value());
            // assert!(result.name() == target_name, "step {} \nresult: {:?}\ntarget: {:?}", step, result.name(), target_name);
        };
        assert!(sent == total_count, "\nresult: {:?}\ntarget: {:?}", sent, total_count);
        assert!(result == total_count * 2, "\nresult: {:?}\ntarget: {:?}", result, total_count * 2);
        let smooth: Vec<Point> = receiver.read().unwrap().received().read().unwrap().iter().cloned().filter(|point| {
            point.name() == format!("/{}/RecorderTask/Smooth", self_id)
        }).collect();
        for (i, result) in smooth.iter().enumerate() {
            println!("smooth: {}\t|\t{}\t|\t{:?}", i, result.name(), result.value());
        };
        let thrd: Vec<Point> = receiver.read().unwrap().received().read().unwrap().iter().cloned().filter(|point| {
            point.name() == format!("/{}/RecorderTask/Threshold", self_id)
        }).collect();
        for (i, result) in thrd.iter().enumerate() {
            println!("threshold: {}\t|\t{}\t|\t{:?}", i, result.name(), result.value());
        };
        let op_cycle: Vec<Point> = receiver.read().unwrap().received().read().unwrap().iter().cloned().filter(|point| {
            point.name() == format!("/{}/RecorderTask/OpCycle", self_id)
        }).collect();
        for (i, result) in op_cycle.iter().enumerate() {
            println!("op cycle: {}\t|\t{}\t|\t{:?}", i, result.name(), result.value());
        };

        let target_name = "/App/RecorderTask/Smooth";
        for (i, result) in smooth.iter().enumerate() {
            let (step, target) = target_smooth[i].clone();
            assert!(result.value().as_real().aprox_eq(target, 3), "step {} \nresult smooth: {:?}\ntarget smooth: {:?}", step, result.value(), target);
            assert!(result.name() == target_name, "step {} \nresult: {:?}\ntarget: {:?}", step, result.name(), target_name);
        };
        let target_name = "/App/RecorderTask/Threshold";
        for (i, result) in thrd.iter().enumerate() {
            let (step, target) = target_thrd[i].clone();
            assert!(result.value().as_real().aprox_eq(target, 3), "step {} \nresult threshold: {:?}\ntarget threshold: {:?}", step, result.value(), target);
            assert!(result.name() == target_name, "step {} \nresult: {:?}\ntarget: {:?}", step, result.name(), target_name);
        };
        let target_name = "/App/RecorderTask/OpCycle";
        for (i, result) in op_cycle.iter().enumerate() {
            let (step, target) = target_op_cycle[i].clone();
            assert!(result.value().as_bool() == (target > 0), "step {} \nresult op cycle: {:?}\ntarget op cycle: {:?}", step, result.value(), target);
            assert!(result.name() == target_name, "step {} \nresult: {:?}\ntarget: {:?}", step, result.name(), target_name);
        };
        test_duration.exit();
    }
}

