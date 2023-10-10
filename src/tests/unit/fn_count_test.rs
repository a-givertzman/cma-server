#![allow(non_snake_case)]
#[cfg(test)]
use log::{debug, info};
use std::{sync::Once, rc::Rc, cell::RefCell};

use crate::core_::{nested_function::{fn_count::FnCount, fn_in::FnIn, fn_::FnInput, fn_::FnOutput, fn_reset::FnReset}, debug::debug_session::{DebugSession, LogLevel}};

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
fn test_single() {
    DebugSession::init(LogLevel::Debug);
    initOnce();
    initEach();
    info!("test_single");
    // let (initial, switches) = initEach();
    let input = Rc::new(RefCell::new(FnIn::new(false)));
    let mut fnCount = FnCount::new(
        0, 
        input.clone(),
    );
    let testData = vec![
        (false, 0),
        (false, 0),
        (true, 1),
        (false, 1),
        (false, 1),
        (true, 2),
        (false, 2),
        (true, 3),
        (false, 3),
        (false, 3),
        (true, 4),
        (true, 4),
        (false, 4),
        (false, 4),
    ];
    for (value, targetState) in testData {
        input.borrow_mut().add(value);
        // debug!("input: {:?}", &input);
        let state = fnCount.out();
        // debug!("input: {:?}", &mut input);
        debug!("value: {:?}   |   state: {:?}", value, state);
        assert_eq!(state, targetState);
    }        
}


#[test]
fn test_multiple() {
    DebugSession::init(LogLevel::Debug);
    info!("test_multiple");
    // let (initial, switches) = initEach();
    let input = Rc::new(RefCell::new(FnIn::new(false)));
    let mut fnCount = FnCount::new(
        0, 
        input.clone(),
    );
    let testData = vec![
        (false, 0),
        (false, 0),
        (true, 1),
        (false, 1),
        (false, 1),
        (true, 2),
        (false, 2),
        (true, 3),
        (false, 3),
        (false, 3),
        (true, 4),
        (true, 4),
        (false, 4),
        (false, 4),
    ];
    for (value, targetState) in testData {
        input.borrow_mut().add(value);
        // debug!("input: {:?}", &input);
        let state = fnCount.out();
        // debug!("input: {:?}", &mut input);
        debug!("value: {:?}   |   state: {:?}", value, state);
        assert_eq!(state, targetState);
    }        
}

#[test]
fn test_multiple_reset() {
    DebugSession::init(LogLevel::Debug);
    info!("test_multiple_reset");
    // let (initial, switches) = initEach();
    let input = Rc::new(RefCell::new(FnIn::new(false)));
    let mut fnCount = FnCount::new(
        0, 
        input.clone(),
    );
    let testData = vec![
        (false, 0, false),
        (false, 0, false),
        (true, 1, false),
        (false, 1, false),
        (false, 1, false),
        (true, 2, false),
        (false, 0, true),
        (true, 1, false),
        (false, 1, false),
        (false, 1, false),
        (true, 2, false),
        (true, 2, false),
        (false, 0, true),
        (false, 0, false),
    ];
    for (value, targetState, reset) in testData {
        if reset {
            fnCount.reset();
        }
        input.borrow_mut().add(value);
        // debug!("input: {:?}", &input);
        let state = fnCount.out();
        // debug!("input: {:?}", &mut input);
        debug!("value: {:?}   |   state: {:?}", value, state);
        assert_eq!(state, targetState);
    }        
}
