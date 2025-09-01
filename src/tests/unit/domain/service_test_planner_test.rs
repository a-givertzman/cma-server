use std::sync::Arc;
#[cfg(test)]
use std::{sync::Once, time::Duration};
use sal_core::dbg::Dbg;
use sal_sync::{services::{conf::ConfTree, entity::{Point, ToPoint}}, sync::Owner};
use testing::{entities::test_value::Value, stuff::max_test_duration::TestDuration};
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
use crate::domain::testing::ServiceTestPlanner;
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
/// Testing such functionality / behavior
#[test]
fn run() {
    DebugSession::init(LogLevel::Debug, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("ServiceTestPlanner-test");
    log::debug!("\n{dbg}");
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(20));
    test_duration.run().unwrap();
    let events = vec![
        vec![   // SendService0
            ("Int0", Value::Int(0)),
            ("Int1", Value::Int(1)),
            ("Int2", Value::Int(2)),
            ("Int3", Value::Int(3)),
            ("Int4", Value::Int(4)),
            ("Int5", Value::Int(5)),
            ("Int6", Value::Int(6)),
        ],
    ];
    let recv_limit0 = events[0].len();
    let conf = ConfTree::new_root(
        serde_yaml::from_str(&format!(r"
            thread-pool: 12
            services:
                retain:
                    path: assets/testing/retain/
                    point:
                        path: point/id.json
                        # api:
                        #     table: public.tags
                        #     address: 0.0.0.0:8080
                        #     auth_token: 123!@#
                        #     database: crane_data_server

            service MultiQueue:
                in queue in-queue:
                    max-length: 10000
                send-to:
                    - /{dbg}/RecvService0.in-queue
                    - /{dbg}/RecvService1.in-queue

            service RecvService RecvService0:
                in queue in-queue:
                    max-length: 10000
                recv-limit: {recv_limit0}
            service RecvService RecvService1:
                in queue in-queue:
                    max-length: 10000
                recv-limit: {recv_limit0}

            service SendService SendService0:
                send-to: /{dbg}/MultiQueue.in-queue
        ")).unwrap(),
    );
    // for (step, val, target) in test_data {
    //     let result = val + 1;
    //     assert!(result == target, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    // }
    let builder_dbg = dbg.clone();
    let each_sent_dbg = dbg.clone();
    let each_received_dbg = dbg.clone();
    let all_received_dbg = dbg.clone();
    let planner: Arc<ServiceTestPlanner>;
    let planner_clone: Arc<Owner<Arc<ServiceTestPlanner>>> = Arc::new(Owner::empty());
    let planner_clone1 = planner_clone.clone();
    planner = Arc::new(ServiceTestPlanner::new(
        &dbg,
        conf,
        move |txid, ix, name: &str, event: &Value| {
            let dbg = builder_dbg.clone();
            log::debug!("{dbg} | test event {ix}: '{name}'");
            event.to_point(txid, name)
        },
        events.clone(),
        vec![
            move |event: &Point| {
                let dbg = each_sent_dbg.clone();
                log::debug!("{dbg} | Sent event: {:?}", event.name());
            },
        ],
        (0..=1).map(|ix| {
            let dbg = each_received_dbg.clone();
            let events = events.first().unwrap().clone();
            move |received: &Vec<Point>| {
                let result: Vec<(String, Value)> = received.iter().map(|p| (p.name(), p.value())).collect();
                log::debug!("{dbg} | Receiver{ix} result: {:?}", result);
                let target: Vec<(String, Value)> = events.iter().map(|(name, val)| (name.to_string(), val.to_owned())).collect();
                assert!(result == target, "{dbg} | Receiver{} \nresult: {:?}\ntarget: {:?}", ix, result, target);
            }
        }).collect(),
        move |received: Vec<Vec<Point>>| {
            let dbg = all_received_dbg.clone();
            log::debug!("{dbg} | All received");
            for (ix, recvd) in received.iter().enumerate() {
                log::debug!("{dbg} | Received[{ix}]: {:?}", recvd);
            }
            planner_clone.take().unwrap().exit();
        },
    ));
    planner_clone1.replace(planner.clone());
    planner.run().unwrap();
    planner.wait().unwrap();
    test_duration.exit();
}
