#[cfg(test)]

mod tcp_server {
    use sal_sync::services::{entity::name::Name, retain::retain_conf::RetainConf, service::service::Service};
    use std::{sync::{Arc, Once, RwLock}, thread, time::Duration};
    use testing::{
        entities::test_value::Value,
        stuff::{max_test_duration::TestDuration, inc_test_values::IncTestValues, wait::WaitTread},
        session::test_session::TestSession,
    };
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::{multi_queue_config::MultiQueueConfig, tcp_server_config::TcpServerConfig},
        services::{
            multi_queue::multi_queue::MultiQueue, safe_lock::rwlock::SafeLock, server::tcp_server::TcpServer,
            services::Services, task::{task_test_producer::TaskTestProducer, task_test_receiver::TaskTestReceiver},
        },
        tests::unit::services::tcp_server::{emulated_tcp_client_recv::EmulatedTcpClientRecv, emulated_tcp_client_send::EmulatedTcpClientSend}
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
    fn keep_send() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "tcp_server_keep_send";
        let self_name = Name::new("", self_id);
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(20));
        test_duration.run().unwrap();
        let iterations = 100;
        let test_data = IncTestValues::new(
            self_id,
            0,
            iterations,
        );
        let test_data: Vec<Value> = test_data.collect();
        let total_count = test_data.len();
        let tcp_port = TestSession::free_tcp_port_str();
        let tcp_addr = format!("127.0.0.1:{}", tcp_port);
        let services = Arc::new(RwLock::new(Services::new(self_id, RetainConf::new(None::<&str>, None))));
        let conf = format!(r#"
            service TcpServer:
                cycle: 10 ms
                reconnect: 1 s  # default 3 s
                address: {}
                auth: none      # auth: none / auth-secret: pass: ... / auth-ssh: path: ...
                in queue link:
                    max-length: 10000
                send-to: {}/MultiQueue.in-queue
        "#, tcp_addr, self_name);
        let conf = serde_yaml::from_str(&conf).unwrap();
        let conf = TcpServerConfig::from_yaml(&self_name, &conf);
        let tcp_server = Arc::new(RwLock::new(TcpServer::new(conf, services.clone())));
        services.wlock(self_id).insert(tcp_server.clone());
        let mq_conf = r#"
            service MultiQueue:
                in queue in-queue:
                    max-length: 10000
                send-to:
        "#;
        let mq_conf = serde_yaml::from_str(mq_conf).unwrap();
        let mq_conf = MultiQueueConfig::from_yaml(&self_name, &mq_conf);
        let mq_service = Arc::new(RwLock::new(MultiQueue::new(mq_conf, services.clone())));
        services.wlock(self_id).insert(mq_service.clone());
        let producer = Arc::new(RwLock::new(TaskTestProducer::new(
            self_id,
            &Name::new(&self_name, "MultiQueue.in-queue").join(),
            Duration::from_millis(100),
            services.clone(),
            test_data.clone(),
        )));
        services.wlock(self_id).insert(producer.clone());
        let emulated_tcp_client_recv = Arc::new(RwLock::new(EmulatedTcpClientRecv::new(
            self_id,
            &tcp_addr,
            Some(iterations),
            Some(test_data.last().unwrap().clone()),
            vec![25, 50, 75],
        )));
        let services_handle = services.wlock(self_id).run().unwrap();
        let mq_service_handle = mq_service.write().unwrap().run().unwrap();
        let tcp_server_handle = tcp_server.write().unwrap().run().unwrap();
        thread::sleep(Duration::from_millis(100));
        let emulated_tcp_client_recv_handle = emulated_tcp_client_recv.write().unwrap().run().unwrap();
        thread::sleep(Duration::from_millis(100));
        let producer_handle = producer.write().unwrap().run().unwrap();
        emulated_tcp_client_recv.read().unwrap().wait_marker_received();
        let received = emulated_tcp_client_recv.read().unwrap().received();
        let received = received.read().unwrap();
        let target = 0.75;
        let result = (received.len() as f32) / (total_count as f32);
        // println!("elapsed: {:?}", timer.elapsed());
        println!("total test events: {:?}", total_count);
        println!("sent events: {:?}", producer.read().unwrap().sent().read().unwrap().len());
        println!("recv events: {:?} ({}%)", received.len(), result * 100.0);
        assert!(result >= target, "\nresult: {:?}\ntarget: {:?}", result, target);
        emulated_tcp_client_recv.read().unwrap().exit();
        producer.read().unwrap().exit();
        tcp_server.read().unwrap().exit();
        mq_service.read().unwrap().exit();
        services.rlock(self_id).exit();
        emulated_tcp_client_recv_handle.wait().unwrap();
        producer_handle.wait().unwrap();
        tcp_server_handle.wait().unwrap();
        mq_service_handle.wait().unwrap();
        services_handle.wait().unwrap();
        test_duration.exit();
    }
    ///
    ///
    #[test]
    fn keep_receive() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "tcp_server_keep_receive";
        let self_name = Name::new("", self_id);
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(30));
        test_duration.run().unwrap();
        let iterations = 100;
        let test_data = IncTestValues::new(
            self_id,
            0,
            iterations,
        );
        let test_data: Vec<Value> = test_data.collect();
        let total_count = test_data.len();
        let services = Arc::new(RwLock::new(Services::new(self_id, RetainConf::new(None::<&str>, None))));
        let tcp_port = TestSession::free_tcp_port_str();
        let tcp_addr = format!("127.0.0.1:{}", tcp_port);
        let conf = format!(r#"
            service TcpServer:
                cycle: 1 ms
                reconnect: 1 s  # default 3 s
                address: {}
                auth: none      # auth: none / auth-secret: pass: ... / auth-ssh: path: ...
                in queue link:
                    max-length: 10000
                send-to: {}/MultiQueue.in-queue
        "#, tcp_addr, self_name);
        let conf = serde_yaml::from_str(&conf).unwrap();
        let conf = TcpServerConfig::from_yaml(&self_name, &conf);
        let tcp_server = Arc::new(RwLock::new(TcpServer::new(conf, services.clone())));
        services.wlock(self_id).insert(tcp_server.clone());

        let mq_conf = format!(r#"
            service MultiQueue:
                in queue in-queue:
                    max-length: 10000
                send-to:
                    - {}/TaskTestReceiver.queue
        "#, self_name);
        let mq_conf = serde_yaml::from_str(&mq_conf).unwrap();
        let mq_conf = MultiQueueConfig::from_yaml(self_name, &mq_conf);
        let mq_service = Arc::new(RwLock::new(MultiQueue::new(mq_conf, services.clone())));
        services.wlock(self_id).insert(mq_service.clone());        // "MultiQueue",
        let receiver = Arc::new(RwLock::new(TaskTestReceiver::new(
            self_id,
            "",
            "queue",
            iterations,
        )));
        services.wlock(self_id).insert(receiver.clone());
        let emulated_tcp_client = Arc::new(RwLock::new(EmulatedTcpClientSend::new(
            self_id,
            "/test/Jds/",
            &tcp_addr,
            test_data.clone(),
            vec![25, 50, 75],
            true,
        )));
        let services_handle = services.wlock(self_id).run().unwrap();
        let mq_service_handle = mq_service.write().unwrap().run().unwrap();
        let tcp_server_handle = tcp_server.write().unwrap().run().unwrap();
        thread::sleep(Duration::from_millis(100));
        let emulated_tcp_client_handle = emulated_tcp_client.write().unwrap().run().unwrap();
        thread::sleep(Duration::from_millis(100));
        let receiver_handle = receiver.write().unwrap().run().unwrap();
        receiver_handle.wait().unwrap();
        emulated_tcp_client.read().unwrap().exit();
        emulated_tcp_client_handle.wait().unwrap();
        let received = receiver.read().unwrap().received();
        let mut received = received.write().unwrap();
        let target = total_count;
        let result = received.len();
        assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        for value in test_data {
            let result = received.remove(0).as_int().value;
            let target = value.as_int();
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        tcp_server.read().unwrap().exit();
        mq_service.read().unwrap().exit();
        services.rlock(self_id).exit();
        tcp_server_handle.wait().unwrap();
        mq_service_handle.wait().unwrap();
        services_handle.wait().unwrap();
        test_duration.exit();
    }
}