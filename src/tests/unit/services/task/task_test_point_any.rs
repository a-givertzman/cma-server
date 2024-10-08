#[cfg(test)]

mod task {
    use log::{trace, info};
    use sal_sync::services::{entity::name::Name, retain::retain_conf::RetainConf, service::service::Service};
    use std::{sync::{Arc, Once, RwLock}, thread, time::{Duration, Instant}};
    use testing::{entities::test_value::Value, stuff::{max_test_duration::TestDuration, random_test_values::RandomTestValues, wait::WaitTread}};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::task_config::TaskConfig,
        services::{safe_lock::rwlock::SafeLock, services::Services, task::{task::Task, task_test_producer::TaskTestProducer, task_test_receiver::TaskTestReceiver}},
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
    ///
    #[test]
    fn point_any_structure() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "task_test_point_any";
        let self_name = Name::new("", self_id);
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(3));
        test_duration.run().unwrap();
        //
        // can be changed
        let iterations = 10;
        let conf = serde_yaml::from_str(&format!(r#"
            service Task TaskAny:
                cycle: 1 ms
                in queue in-queue:
                    max-length: 10000
                fn ToApiQueue:
                    queue: {}/TaskTestReceiver.in-queue
                    input fn SqlMetric:
                        initial: 0.123      # начальное значение
                        table: table_name
                        sql: "insert into {{table}} (id, value, timestamp) values ({{id}}, {{input1.value}}, {{input1.value}});"
                        input1: point any every
        "#, self_name)).unwrap();
        let config = TaskConfig::from_yaml(&self_name, &conf);
        trace!("config: {:?}", &config);
        let services = Arc::new(RwLock::new(Services::new(&self_name, RetainConf::new(None::<&str>, None))));
        let receiver = Arc::new(RwLock::new(TaskTestReceiver::new(
            &self_name.join(),
            "",
            "in-queue",
            iterations,
        )));
        services.wlock(self_id).insert(receiver.clone());
        let test_data = RandomTestValues::new(
            self_id,
            vec![
                Value::Real(-7.035),
                Value::Real(-2.5),
                Value::Real(-5.5),
                Value::Real(-1.5),
                Value::Real(-1.0),
                Value::Real(-0.1),
                Value::Real(0.1),
                Value::Real(1.0),
                Value::Real(1.5),
                Value::Real(5.5),
                Value::Real(2.5),
                Value::Real(7.035),
            ],
            iterations,
        );
        let test_data: Vec<Value> = test_data.collect();
        let total_count = test_data.len();
        assert!(total_count == iterations, "\nresult: {:?}\ntarget: {:?}", total_count, iterations);
        let producer = Arc::new(RwLock::new(TaskTestProducer::new(
            &self_name.join(),
            &Name::new(self_name, "TaskAny.in-queue").join(),
            Duration::ZERO,
            services.clone(),
            test_data,
        )));
        let task = Arc::new(RwLock::new(Task::new(config, services.clone())));
        services.wlock(self_id).insert(task.clone());
        let services_handle = services.wlock(self_id).run().unwrap();
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
        services.rlock(self_id).exit();
        task_handle.wait().unwrap();
        producer_handle.wait().unwrap();
        services_handle.wait().unwrap();
        let sent = producer.read().unwrap().sent().read().unwrap().len();
        let result = receiver.read().unwrap().received().read().unwrap().len();
        println!(" elapsed: {:?}", time.elapsed());
        println!("    sent: {:?}", sent);
        println!("received: {:?}", result);
        assert!(sent == iterations, "\nresult: {:?}\ntarget: {:?}", sent, iterations);
        assert!(result == iterations, "\nresult: {:?}\ntarget: {:?}", result, iterations);
        test_duration.exit();
    }
}

