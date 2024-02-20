#![allow(non_snake_case)]
#[cfg(test)]

mod tests {
    use log::debug;
    use std::{sync::Once, time::Duration};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use testing::stuff::max_test_duration::TestDuration;
    use crate::conf::{point_config::{point_config::PointConfig, point_config_history::PointConfigHistory, point_config_type::PointConfigType}, profinet_client_config::profinet_client_config::ProfinetClientConfig}; 
    
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    // use super::*;
    
    static INIT: Once = Once::new();
    
    ///
    /// once called initialisation
    fn initOnce() {
        INIT.call_once(|| {
                // implement your initialisation code to be called only once for current test file
            }
        )
    }
    
    
    ///
    /// returns:
    ///  - ...
    fn initEach() -> () {
    
    }
    
    #[test]
    fn profinet_clientConfig() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        initOnce();
        initEach();
        println!("");
        let selfId = "test ProfinetClientConfig";
        println!("{}", selfId);
        let testDuration = TestDuration::new(selfId, Duration::from_secs(10));
        testDuration.run().unwrap();
        let path = "./src/tests/unit/conf/profinet_client_config/profinet_client.yaml";
        let config = ProfinetClientConfig::read(path);
        debug!("config: {:?}", &config);
        debug!("config points:");
        let targetPoints = [
            PointConfig { name: format!("{}/db899/Drive.Speed", selfId), _type: PointConfigType::Float, history: PointConfigHistory::None, alarm: None, address: None, filters: None, comment: None },
            PointConfig { name: format!("{}/db899/Drive.OutputVoltage", selfId), _type: PointConfigType::Float, history: PointConfigHistory::None, alarm: None, address: None, filters: None, comment: None },
            PointConfig { name: format!("{}/db899/Drive.DCVoltage", selfId), _type: PointConfigType::Float, history: PointConfigHistory::None, alarm: None, address: None, filters: None, comment: None },
            PointConfig { name: format!("{}/db899/Drive.Current", selfId), _type: PointConfigType::Float, history: PointConfigHistory::Read, alarm: None, address: None, filters: None, comment: None },
            PointConfig { name: format!("{}/db899/Drive.Torque", selfId), _type: PointConfigType::Float, history: PointConfigHistory::None, alarm: None, address: None, filters: None, comment: None },
            PointConfig { name: format!("{}/db899/Drive.positionFromMru", selfId), _type: PointConfigType::Float, history: PointConfigHistory::None, alarm: None, address: None, filters: None, comment: None },
            PointConfig { name: format!("{}/db899/Drive.positionFromHoist", selfId), _type: PointConfigType::Float, history: PointConfigHistory::None, alarm: None, address: None, filters: None, comment: None },
            PointConfig { name: format!("{}/db899/Capacitor.Capacity", selfId), _type: PointConfigType::Int, history: PointConfigHistory::None, alarm: None, address: None, filters: None, comment: None },
            PointConfig { name: format!("{}/db899/ChargeIn.On", selfId), _type: PointConfigType::Bool, history: PointConfigHistory::None, alarm: None, address: None, filters: None, comment: None },
            PointConfig { name: format!("{}/db899/ChargeOut.On", selfId), _type: PointConfigType::Bool, history: PointConfigHistory::None, alarm: None, address: None, filters: None, comment: None },
        ];
        let configPoints = config.points();
        for point in &configPoints {
            println!("\t {:?}", point);
        }
        for target in &targetPoints {
            let result = configPoints.iter().find(|point| {
                point.name == target.name
            });
            assert!(result.is_some(), "result points does not contains '{}'", target.name);
            let result = result.unwrap();
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        let result = config.points().len();
        let target = targetPoints.len();
        assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        testDuration.exit();
    }
}
