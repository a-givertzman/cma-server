#[cfg(test)]

mod queue_name {
    use std::{sync::Once, time::Duration};
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};

    use crate::services::queue_name::QueueName;
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
    /// Testing QueueName.split
    #[test]
    fn split() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "queue_name_split";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = [
            (00, "Service1.queue", Ok(("Service1", "queue"))),
            (01, "Service1.send-to", Ok(("Service1", "send-to"))),
            (02, "Service1.in-queue", Ok(("Service1", "in-queue"))),
            (03, "Service1.out-queue", Ok(("Service1", "out-queue"))),
            (04, "/app/Service1.queue", Ok(("/app/Service1", "queue"))),
            (05, "/App/Service/Service1.send-to", Ok(("/App/Service/Service1", "send-to"))),
            (06, "/App-1/Serv-1/Service1.in-queue", Ok(("/App-1/Serv-1/Service1", "in-queue"))),
            (07, "/A-1/Service1.out-queue", Ok(("/A-1/Service1", "out-queue"))),
            (08, "out-queue", Err(())),
            (09, "Service1", Err(())),
        ];
        for (step, input, target) in test_data {
            let queue_name = QueueName::new(input);
            let result = queue_name.split();
            match result {
                Ok((service, queue)) => {
                    let (target_service, target_queue) = target.unwrap();
                    assert!(service == target_service, "step: {}\nresult: {:?}\ntarget: {:?}", step, service, target_service);
                    assert!(queue == target_queue, "step: {}\nresult: {:?}\ntarget: {:?}", step, queue, target_queue);
                }
                Err(_) => assert!(target.is_err(), "step: {}\nresult: {:?}\ntarget: {:?}", step, result, target),
            }
            let result = queue_name.split();
            match result {
                Ok((service, queue)) => {
                    let (target_service, target_queue) = target.unwrap();
                    assert!(service == target_service, "step: {}\nresult: {:?}\ntarget: {:?}", step, service, target_service);
                    assert!(queue == target_queue, "step: {}\nresult: {:?}\ntarget: {:?}", step, queue, target_queue);
                }
                Err(_) => assert!(target.is_err(), "step: {}\nresult: {:?}\ntarget: {:?}", step, result, target),
            }
        }
        test_duration.exit();
    }
    ///
    /// Testing QueueName.service
    #[test]
    fn service() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "queue_name_service";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = [
            (00, "Service1.queue", Ok(("Service1", "queue"))),
            (01, "Service1.send-to", Ok(("Service1", "send-to"))),
            (02, "Service1.in-queue", Ok(("Service1", "in-queue"))),
            (03, "Service1.out-queue", Ok(("Service1", "out-queue"))),
            (04, "/app/Service1.queue", Ok(("/app/Service1", "queue"))),
            (05, "/App/Service/Service1.send-to", Ok(("/App/Service/Service1", "send-to"))),
            (06, "/App-1/Serv-1/Service1.in-queue", Ok(("/App-1/Serv-1/Service1", "in-queue"))),
            (07, "/A-1/Service1.out-queue", Ok(("/A-1/Service1", "out-queue"))),
            (08, "out-queue", Err(())),
            (09, "Service1", Err(())),
        ];
        for (step, input, target) in test_data {
            let queue_name = QueueName::new(input);
            let result = queue_name.service();
            match result {
                Ok(service) => {
                    let (target_service, _) = target.unwrap();
                    assert!(service == target_service, "step: {}\nresult: {:?}\ntarget: {:?}", step, service, target_service);
                }
                Err(_) => assert!(target.is_err(), "step: {}\nresult: {:?}\ntarget: {:?}", step, result, target),
            }
        }
        test_duration.exit();
    }
    ///
    /// Testing QueueName.queue
    #[test]
    fn queue() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "queue_name_queue";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = [
            (00, "Service1.queue", Ok(("Service1", "queue"))),
            (01, "Service1.send-to", Ok(("Service1", "send-to"))),
            (02, "Service1.in-queue", Ok(("Service1", "in-queue"))),
            (03, "Service1.out-queue", Ok(("Service1", "out-queue"))),
            (04, "/app/Service1.queue", Ok(("/app/Service1", "queue"))),
            (05, "/App/Service/Service1.send-to", Ok(("/App/Service/Service1", "send-to"))),
            (06, "/App-1/Serv-1/Service1.in-queue", Ok(("/App-1/Serv-1/Service1", "in-queue"))),
            (07, "/A-1/Service1.out-queue", Ok(("/A-1/Service1", "out-queue"))),
            (08, "out-queue", Err(())),
            (09, "Service1", Err(())),
        ];
        for (step, input, target) in test_data {
            let queue_name = QueueName::new(input);
            let result = queue_name.queue();
            match result {
                Ok(queue) => {
                    let (_, target_queue) = target.unwrap();
                    assert!(queue == target_queue, "step: {}\nresult: {:?}\ntarget: {:?}", step, queue, target_queue);
                }
                Err(_) => assert!(target.is_err(), "step: {}\nresult: {:?}\ntarget: {:?}", step, result, target),
            }
        }
        test_duration.exit();
    }
    ///
    /// Testing QueueName.validate
    #[test]
    fn validate() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "queue_name_validate";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = [
            (00, "Service1.queue", Ok(("Service1", "queue"))),
            (01, "Service1.send-to", Ok(("Service1", "send-to"))),
            (02, "Service1.in-queue", Ok(("Service1", "in-queue"))),
            (03, "Service1.out-queue", Ok(("Service1", "out-queue"))),
            (04, "/app/Service1.queue", Ok(("/app/Service1", "queue"))),
            (05, "/App/Service/Service1.send-to", Ok(("/App/Service/Service1", "send-to"))),
            (06, "/App-1/Serv-1/Service1.in-queue", Ok(("/App-1/Serv-1/Service1", "in-queue"))),
            (07, "/A-1/Service1.out-queue", Ok(("/A-1/Service1", "out-queue"))),
            (08, "out-queue", Err(())),
            (09, "Service1", Err(())),
        ];
        for (step, input, target) in test_data {
            if target.is_ok() {
                let qn = QueueName::new(input);
                let target = qn.split();
                let result = qn.validate().split();
                assert!(result == target, "step: {}\nresult: {:?}\ntarget: {:?}", step, result, target);
            }
        }
        test_duration.exit();
    }

}
