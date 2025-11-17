#![allow(non_snake_case)]
#[cfg(test)]
mod tests;
mod core_;
mod task;
use std::{env, time::Duration, thread};

use core_::{debug::debug_session::DebugSession, conf::ConfTree};

use crate::{core_::{conf::task_config::TaskConfig, debug::debug_session::LogLevel}, task::task::Task};




fn main() {
    DebugSession::new().filter(LogLevel::Debug).init();
    log::info!("test_task");
    
    // let (initial, switches) = init_each();
    log::trace!("dir: {:?}", env::current_dir());
    let path = "./src/tests/unit/task/task_config_test.yaml";
    let config = TaskConfig::read(path);
    log::trace!("config: {:?}", &config);
    let mut task = Task::new(config);
    log::trace!("task tuning...");
    task.run();
    log::trace!("task tuning - ok");
    thread::sleep(Duration::from_secs_f32(0.5));
    log::trace!("task stopping...");
    task.exit();
    log::trace!("task stopping - ok");
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
    log::debug!("map: {:?}", &map);
    let removed = map.remove_entry("input2");
    log::debug!("removed: {:?}", &removed);
    log::debug!("map: {:?}", &map);
}
