#![allow(non_snake_case)]
#[cfg(test)]
mod tests;
mod core_;
mod task;
use std::{env, time::Duration, thread};

use log::{debug, info, trace};
use core_::{debug::debug_session::DebugSession, conf::conf_tree::ConfTree};

use crate::{core_::{conf::task_config::TaskConfig, debug::debug_session::LogLevel}, task::task::Task};




fn main() {
    DebugSession::init(LogLevel::Debug, Backtrace::Short);
    info!("test_task");
    
    // let (initial, switches) = init_each();
    trace!("dir: {:?}", env::current_dir());
    let path = "./src/tests/unit/task/task_config_test.yaml";
    let config = TaskConfig::read(path);
    trace!("config: {:?}", &config);
    let mut task = Task::new(config);
    trace!("task tuning...");
    task.run();
    trace!("task tuning - ok");
    thread::sleep(Duration::from_secs_f32(0.5));
    trace!("task stopping...");
    task.exit();
    trace!("task stopping - ok");
}

fn main1() {
    DebugSession::init(core_::debug::debug_session::LogLevel::Debug);
    let test_data = [
        r#"
            input1: const 177.3
            input2: point '/Path/Point.Name/'
            input3:
                fn Count:
                    inputConst1: const '13.5'
                    inputConst2: newVar1
        "#,
    ];
    let mut conf: serde_yaml::Value = serde_yaml::from_str(test_data[0]).unwrap();
    let map = conf.as_mapping_mut().unwrap();
    debug!("map: {:?}", &map);
    let removed = map.remove_entry("input2");
    debug!("removed: {:?}", &removed);
    debug!("map: {:?}", &map);
}
