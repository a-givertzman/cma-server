#[cfg(test)]

mod cma_recorder {
    use sal_sync::{services::{conf::{ConfTree, ServicesConf}, entity::Name, MultiQueue, MultiQueueConf, Service, Services}, thread_pool::ThreadPool};
    use std::{env, sync::{Arc, Once}, thread, time::{Duration, Instant}};
    use testing::{entities::test_value::Value, stuff::max_test_duration::TestDuration};
    use debugging::session::debug_session::{DebugSession, LogLevel};
    use crate::{
        services::task::{Task, TaskConf, TaskTestReceiver},
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
    /// Testing Task FnFilter for Real's
    #[test]
    fn filter() {
        DebugSession::new().filter(LogLevel::Info).init();
        init_once();
        init_each();
        let dbg = "App";
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
        let config = TaskConf::from_yaml(
            &self_name,
            &serde_yaml::from_str(r"
                service Task RecorderTask:
                    cycle: 1 ms
                    in queue recv-queue:
                        max-length: 10000
                    subscribe:
                        /App/MultiQueue:                    # - multicast subscription to the MultiQueue
                            {cot: Inf}: []                      #   - on all points having Cot::Inf
                    fn Debug debug01:
                        input fn Export:
                            send-to: /App/MultiQueue.in-queue
                            conf point Load001:
                                type: 'Real'
                            input fn Filter:
                                pass fn Ge:
                                    input1: point real '/App/Load'
                                    input2: const real 1.5
                                input: point real '/App/Load'
                    fn Debug debug02:
                        input point Load002:
                            type: 'Real'
                            input: point real '/App/RecorderTask/Load001'
                            send-to: /App/TaskTestReceiver.in-queue
            ").unwrap(),
        );
        log::trace!("config: {:?}", config);
        log::debug!("Task config points: {:#?}", config.points());
        let task = Arc::new(Task::new(config, services.clone(), tp.scheduler()));
        log::debug!("Task points: {:#?}", task.points());
        services.insert(task.clone());
        let conf = MultiQueueConf::from_yaml(
            dbg,
            &serde_yaml::from_str(r"service MultiQueue:
                in queue in-queue:
                    max-length: 10000
            ").unwrap(),
        );
        let multi_queue = Arc::new(MultiQueue::new(conf, services.clone(), Some(tp.scheduler())));
        services.insert(multi_queue.clone());
        let test_data = vec![
            (format!("/{}/Load", dbg), Value::Real(-7.035),  None),
            (format!("/{}/Load", dbg), Value::Real(-2.5),    None),
            (format!("/{}/Load", dbg), Value::Real(-5.5),    None),
            (format!("/{}/Load", dbg), Value::Real(-1.5),    None),
            (format!("/{}/Load", dbg), Value::Real(-1.0),    None),
            (format!("/{}/Load", dbg), Value::Real(-0.1),    None),
            (format!("/{}/Load", dbg), Value::Real(0.1),     None),
            (format!("/{}/Load", dbg), Value::Real(1.0),     None),
            (format!("/{}/Load", dbg), Value::Real(1.5),     Some(1.5)),
            (format!("/{}/Load", dbg), Value::Real(5.5),     Some(5.5)),
            (format!("/{}/Load", dbg), Value::Real(2.5),     Some(2.5)),
            (format!("/{}/Load", dbg), Value::Real(7.035),   Some(7.035)),
        ];
        let mut target_data = test_data.iter().filter(|(_, _, target)| target.is_some());
        let total_count = test_data.len();
        let target_count = target_data.clone().count();
        let receiver = Arc::new(TaskTestReceiver::new(
            dbg,
            "",
            "in-queue",
            target_count,
        ));
        services.insert(receiver.clone());
        let producer = Arc::new(TaskTestProducer::new(
            dbg,
            &format!("/{}/MultiQueue.in-queue", dbg),
            Duration::from_millis(10),
            services.clone(),
            &test_data.iter().cloned().map(|(name, value, _)| (name, value)).collect::<Vec<(String, Value)>>(),
        ));
        services.insert(producer.clone());
        services.run().unwrap();
        multi_queue.run().unwrap();
        receiver.run().unwrap();
        log::info!("receiver runing - ok");
        thread::sleep(Duration::from_millis(100));
        task.run().unwrap();
        log::info!("task runing - ok");
        thread::sleep(Duration::from_millis(100));
        producer.run().unwrap();
        log::info!("producer runing - ok");
        let time = Instant::now();
        receiver.wait().unwrap();
        producer.exit();
        task.exit();
        task.wait().unwrap();
        producer.wait().unwrap();
        multi_queue.exit();
        multi_queue.wait().unwrap();
        services.exit();
        services.wait().unwrap();
        let sent = producer.sent().len();
        let result = receiver.received().len();
        println!(" elapsed: {:?}", time.elapsed());
        println!("    sent: {:?}", sent);
        println!("received: {:?}", result);
        println!("target  : {:?}", target_count);
        assert!(sent == total_count, "\nresult: {:?}\ntarget: {:?}", sent, total_count);
        assert!(result == target_count, "\nresult: {:?}\ntarget: {:?}", result, target_count);
        let target_name = "/App/RecorderTask/Load002";
        for result in receiver.received().iter() {
            let (_, _, target) = target_data.next().unwrap();
            assert!(result.value().as_real() == target.unwrap(), "\nresult: {:?}\ntarget: {:?}", result.value(), target);
            assert!(result.name() == target_name, "\nresult: {:?}\ntarget: {:?}", result.name(), target_name);
        };
        test_duration.exit();
    }
}

