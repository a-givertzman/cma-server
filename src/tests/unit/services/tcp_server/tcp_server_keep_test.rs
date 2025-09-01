#[cfg(test)]

mod tcp_server {
    use sal_sync::{services::{conf::{ConfTree, ServicesConf}, entity::Name, MultiQueue, MultiQueueConf, Service, Services}, thread_pool::ThreadPool};
    use std::{sync::{Arc, Once}, thread, time::Duration};
    use testing::{
        entities::test_value::Value,
        stuff::{max_test_duration::TestDuration, inc_test_values::IncTestValues},
        session::test_session::TestSession,
    };
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        services::{
            server::{TcpServerConf, TcpServer},
            task::{TaskTestProducer, TaskTestReceiver},
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
        let dbg = "tcp_server_keep_send";
        let self_name = Name::new("", dbg);
        println!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(20));
        test_duration.run().unwrap();
        let iterations = 100;
        let test_data = IncTestValues::new(
            dbg,
            0,
            iterations,
        );
        let test_data: Vec<Value> = test_data.collect();
        let total_count = test_data.len();
        let tcp_port = TestSession::free_tcp_port_str();
        let tcp_addr = format!("127.0.0.1:{}", tcp_port);
        let tp = ThreadPool::new(dbg, Some(8));
        let services = Arc::new(Services::new(dbg, ServicesConf::new(
            dbg, 
            ConfTree::new_root(serde_yaml::from_str(r#"
                retain:
            "#).unwrap()),
        ), Some(tp.scheduler())));
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
        let conf = TcpServerConf::from_yaml(&self_name, &conf);
        let tcp_server = Arc::new(TcpServer::new(conf, services.clone(), tp.scheduler()));
        services.insert(tcp_server.clone());
        let mq_conf = r#"
            service MultiQueue:
                in queue in-queue:
                    max-length: 10000
                send-to:
        "#;
        let mq_conf = serde_yaml::from_str(mq_conf).unwrap();
        let mq_conf = MultiQueueConf::from_yaml(&self_name, &mq_conf);
        let mq_service = Arc::new(MultiQueue::new(mq_conf, services.clone(), Some(tp.scheduler())));
        services.insert(mq_service.clone());
        let producer = Arc::new(TaskTestProducer::new(
            dbg,
            &Name::new(&self_name, "MultiQueue.in-queue").join(),
            Duration::from_millis(100),
            services.clone(),
            test_data.clone(),
        ));
        services.insert(producer.clone());
        let emulated_tcp_client_recv = Arc::new(EmulatedTcpClientRecv::new(
            dbg,
            &tcp_addr,
            Some(iterations),
            Some(test_data.last().unwrap().clone()),
            vec![25, 50, 75],
        ));
        services.run().unwrap();
        mq_service.run().unwrap();
        tcp_server.run().unwrap();
        thread::sleep(Duration::from_millis(100));
        emulated_tcp_client_recv.run().unwrap();
        thread::sleep(Duration::from_millis(100));
        producer.run().unwrap();
        emulated_tcp_client_recv.wait_marker_received();
        let received = emulated_tcp_client_recv.received();
        let received = received.read();
        let target = 0.75;
        let result = (received.len() as f32) / (total_count as f32);
        // println!("elapsed: {:?}", timer.elapsed());
        println!("total test events: {:?}", total_count);
        println!("sent events: {:?}", producer.sent().len());
        println!("recv events: {:?} ({}%)", received.len(), result * 100.0);
        assert!(result >= target, "\nresult: {:?}\ntarget: {:?}", result, target);
        emulated_tcp_client_recv.exit();
        producer.exit();
        tcp_server.exit();
        mq_service.exit();
        services.exit();
        emulated_tcp_client_recv.wait().unwrap();
        producer.wait().unwrap();
        tcp_server.wait().unwrap();
        mq_service.wait().unwrap();
        services.wait().unwrap();
        test_duration.exit();
    }
    ///
    ///
    #[test]
    fn keep_receive() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        let dbg = "tcp_server_keep_receive";
        let self_name = Name::new("", dbg);
        println!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(30));
        test_duration.run().unwrap();
        let iterations = 100;
        let test_data = IncTestValues::new(
            dbg,
            0,
            iterations,
        );
        let test_data: Vec<Value> = test_data.collect();
        let total_count = test_data.len();
        let tp = ThreadPool::new(dbg, Some(8));
        let services = Arc::new(Services::new(dbg, ServicesConf::new(
            dbg, 
            ConfTree::new_root(serde_yaml::from_str(r#"
                retain:
            "#).unwrap()),
        ), Some(tp.scheduler())));
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
        let conf = TcpServerConf::from_yaml(&self_name, &conf);
        let tcp_server = Arc::new(TcpServer::new(conf, services.clone(), tp.scheduler()));
        services.insert(tcp_server.clone());

        let mq_conf = format!(r#"
            service MultiQueue:
                in queue in-queue:
                    max-length: 10000
                send-to:
                    - {}/TaskTestReceiver.queue
        "#, self_name);
        let mq_conf = serde_yaml::from_str(&mq_conf).unwrap();
        let mq_conf = MultiQueueConf::from_yaml(self_name, &mq_conf);
        let mq_service = Arc::new(MultiQueue::new(mq_conf, services.clone(), Some(tp.scheduler())));
        services.insert(mq_service.clone());        // "MultiQueue",
        let receiver = Arc::new(TaskTestReceiver::new(
            dbg,
            "",
            "queue",
            iterations,
        ));
        services.insert(receiver.clone());
        let emulated_tcp_client = Arc::new(EmulatedTcpClientSend::new(
            dbg,
            "/test/Jds/",
            &tcp_addr,
            test_data.clone(),
            vec![25, 50, 75],
            true,
        ));
        services.run().unwrap();
        mq_service.run().unwrap();
        tcp_server.run().unwrap();
        thread::sleep(Duration::from_millis(100));
        emulated_tcp_client.run().unwrap();
        thread::sleep(Duration::from_millis(100));
        receiver.run().unwrap();
        receiver.wait().unwrap();
        emulated_tcp_client.exit();
        emulated_tcp_client.wait().unwrap();
        let mut received = receiver.received();
        let target = total_count;
        let result = received.len();
        assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        for value in test_data {
            let result = received.remove(0).as_int().value;
            let target = value.as_int();
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        tcp_server.exit();
        mq_service.exit();
        services.exit();
        tcp_server.wait().unwrap();
        mq_service.wait().unwrap();
        services.wait().unwrap();
        test_duration.exit();
    }
}