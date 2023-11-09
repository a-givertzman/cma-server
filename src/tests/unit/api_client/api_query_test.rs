#![allow(non_snake_case)]
#[cfg(test)]

use log::{warn, info, debug};
use std::{sync::Once, time::{Duration, Instant}};
use crate::core_::debug::debug_session::{DebugSession, LogLevel, Backtrace}; 

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
fn test_api_query() {
    DebugSession::init(LogLevel::Debug, Backtrace::Short);
    initOnce();
    initEach();
    println!("");
    info!("test_api_query");
    {
        "auth_token": "123zxy456!@#",
        "id": "123",
        "keep-alive": true,
        "sql": {
            "database": "database name",
            "sql": "Some valid sql query"
        },
        "debug": false
    }    
    let testData = vec![
        (),
    ]
    assert!(result == target, "result: {:?}\ntarget: {:?}", result, target);
}