#[cfg(test)]

use sal_sync::services::entity::PointConfAddress;
use sal_sync::services::entity::{Name, {PointConf, PointConfHistory, PointType}};
use std::{sync::Once, time::Duration};
use debugging::session::{DebugSession, LogLevel};
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
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
    init_once();
    init_each();
    let dbg = "ProfinetClientConf-test";
    let self_name = Name::from(dbg);
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(10));
    test_duration.run().unwrap();
    let path = "./src/tests/unit/conf/profinet_client_conf/profinet_client.yaml";
    let config = ProfinetClientConf::read(&self_name, path);
    type Hist = PointConfHistory;
    type Ptyp = PointType;
    fn point_conf(id: usize, parent: impl Into<String>, name: impl Into<String>, typ: PointType, hist: Hist, offset: u32, bit: Option<u8>) -> PointConf {
        PointConf { id, name: Name::new(parent, name).join(), type_: typ, history: hist, alarm: None, address: Some(PointConfAddress { offset: Some(offset), bit }), filters: None, comment: None }
    }
    let target_points = [
        // 222
        point_conf(0, &self_name, "/Ied01/db222/Drive.Speed", Ptyp::Real, Hist::None, 0, None),
        point_conf(0, &self_name, "/Ied01/db222/Drive.OutputVoltage", Ptyp::Real, Hist::None, 4, None),
        point_conf(0, &self_name, "/Ied01/db222/Drive.DCVoltage", Ptyp::Real, Hist::None, 8, None ),
        point_conf(0, &self_name, "/Ied01/db222/Drive.Current", Ptyp::Real, Hist::Read, 12, None),
        point_conf(0, &self_name, "/Ied01/db222/Drive.Torque", Ptyp::Real, Hist::None, 16, None),
        // 999
        point_conf(0, &self_name, "/Ied01/db999/Drive.positionFromMru", Ptyp::Real, Hist::None, 20, None),
        point_conf(0, &self_name, "/Ied01/db999/Drive.positionFromHoist", Ptyp::Real, Hist::None, 24, None),
        point_conf(0, &self_name, "/Ied01/db999/Capacitor.Capacity", Ptyp::Int, Hist::None, 28, None),
        point_conf(0, &self_name, "/Ied01/db999/ChargeIn.On", Ptyp::Bool, Hist::None, 30, Some(0)),
        point_conf(0, &self_name, "/Ied01/db999/ChargeOut.On", Ptyp::Bool, Hist::None, 32, Some(4)),
        // 888
        point_conf(0, &self_name, "/Ied01/db888/Sensor.Real", Ptyp::Real, Hist::None, 0, None),
        point_conf(0, &self_name, "/Ied01/db888/Sensor.Int", Ptyp::Int, Hist::None, 4, None),
    ];
    log::debug!("{dbg} | result config: {:?}", &config);
    log::debug!("{dbg} | result points:");
    let config_points = config.points();
    for point in &config_points {
        log::debug!("\t {:?}", point);
    }
    for target in &target_points {
        let result = config_points.iter().find(|point| {
            point.name == target.name
        });
        assert!(result.is_some(), "{dbg} | result points does not contains '{}'", target.name);
        let result = result.unwrap();
        assert!(result == target, "{dbg} | \nresult: {:?}\ntarget: {:?}", result, target);
    }
    let result = config.points().len();
    let target = target_points.len();
    assert!(result == target, "{dbg} | \nresult: {:?}\ntarget: {:?}", result, target);
    // Проверка суммарного адреса
    let target  = [16 + 4, 32 + 2, 4 + 2];
    for ((name, db), target) in config.dbs.iter().zip(target) {
        let result = db.size;
        assert!(result == target, "{dbg} | db '{name}': \nresult: {:?}\ntarget: {:?}", result, target);
    }
    test_duration.exit();
}
