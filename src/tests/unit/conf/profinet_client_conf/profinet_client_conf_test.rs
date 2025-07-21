#[cfg(test)]

use sal_sync::services::entity::{Name, {PointConf, PointConfHistory, PointConfType}};
use std::{sync::Once, time::Duration};
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
use testing::stuff::max_test_duration::TestDuration;
use crate::conf::profinet_client_conf::profinet_client_conf::ProfinetClientConf;
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
fn basic() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    let self_id = "profinet_client_config_test";
    let self_name = Name::from(self_id);
    println!("\n{}", self_id);
    let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
    test_duration.run().unwrap();
    let path = "./src/tests/unit/conf/profinet_client_config/profinet_client.yaml";
    let config = ProfinetClientConf::read(&self_name, path);
    let target_points = [
        // 222
        PointConf { id: 0, name: Name::new(&self_name, "/Ied01/db222/Drive.Speed").join(), type_: PointConfType::Real, history: PointConfHistory::None, alarm: None, address: None, filters: None, comment: None },
        PointConf { id: 0, name: Name::new(&self_name, "/Ied01/db222/Drive.OutputVoltage").join(), type_: PointConfType::Real, history: PointConfHistory::None, alarm: None, address: None, filters: None, comment: None },
        PointConf { id: 0, name: Name::new(&self_name, "/Ied01/db222/Drive.DCVoltage").join(), type_: PointConfType::Real, history: PointConfHistory::None, alarm: None, address: None, filters: None, comment: None },
        PointConf { id: 0, name: Name::new(&self_name, "/Ied01/db222/Drive.Current").join(), type_: PointConfType::Real, history: PointConfHistory::Read, alarm: None, address: None, filters: None, comment: None },
        PointConf { id: 0, name: Name::new(&self_name, "/Ied01/db222/Drive.Torque").join(), type_: PointConfType::Real, history: PointConfHistory::None, alarm: None, address: None, filters: None, comment: None },
        // 999
        PointConf { id: 0, name: Name::new(&self_name, "/Ied01/db999/Drive.positionFromMru").join(), type_: PointConfType::Real, history: PointConfHistory::None, alarm: None, address: None, filters: None, comment: None },
        PointConf { id: 0, name: Name::new(&self_name, "/Ied01/db999/Drive.positionFromHoist").join(), type_: PointConfType::Real, history: PointConfHistory::None, alarm: None, address: None, filters: None, comment: None },
        PointConf { id: 0, name: Name::new(&self_name, "/Ied01/db999/Capacitor.Capacity").join(), type_: PointConfType::Int, history: PointConfHistory::None, alarm: None, address: None, filters: None, comment: None },
        PointConf { id: 0, name: Name::new(&self_name, "/Ied01/db999/ChargeIn.On").join(), type_: PointConfType::Bool, history: PointConfHistory::None, alarm: None, address: None, filters: None, comment: None },
        PointConf { id: 0, name: Name::new(&self_name, "/Ied01/db999/ChargeOut.On").join(), type_: PointConfType::Bool, history: PointConfHistory::None, alarm: None, address: None, filters: None, comment: None },
    ];
    log::debug!("result config: {:?}", &config);
    log::debug!("result points:");
    let config_points = config.points();
    for point in &config_points {
        println!("\t {:?}", point);
    }
    for target in &target_points {
        let result = config_points.iter().find(|point| {
            point.name == target.name
        });
        assert!(result.is_some(), "result points does not contains '{}'", target.name);
        let result = result.unwrap();
        assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
    }
    let result = config.points().len();
    let target = target_points.len();
    assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
    test_duration.exit();
}
