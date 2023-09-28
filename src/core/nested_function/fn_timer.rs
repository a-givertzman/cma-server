#![allow(non_snake_case)]

use log::debug;
use std::{cell::RefCell, rc::Rc, time::Instant};

use crate::core::state::switch_state::{SwitchState, Switch, SwitchCondition};

use super::fn_::FnOutput;


#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum FnTimerState {
    Off,
    Start,
    Progress,
    Stop,
}
///
/// Counts number of raised fronts of boolean input
pub struct FnTimer {
    input: Rc<RefCell<dyn FnOutput<bool>>>,
    state: SwitchState<FnTimerState, bool>,
    total: f64,
    start: Instant,
}

impl FnTimer {
    pub fn new(initial: impl Into<f64>, input: Rc<RefCell<dyn FnOutput<bool>>>) -> Self {
        let switches = vec![
            Switch{
                state: FnTimerState::Off,
                conditions: vec![
                    SwitchCondition {
                        condition: Box::new(|value| {value}),
                        target: FnTimerState::Start,
                    },
                ],
            },
            Switch{
                state: FnTimerState::Start,
                conditions: vec![
                    SwitchCondition {
                        condition: Box::new(|value| {value}),
                        target: FnTimerState::Start,
                    },
                    SwitchCondition {
                        condition: Box::new(|value| {!value}),
                        target: FnTimerState::Stop,
                    },
                ],
            },
            Switch{
                state: FnTimerState::Progress,
                conditions: vec![
                    // SwitchCondition {
                    //     condition: Box::new(|value| {value}),
                    //     target: FnTimerState::Progress,
                    // },
                    SwitchCondition {
                        condition: Box::new(|value| {!value}),
                        target: FnTimerState::Stop,
                    },
                ],
            },
            Switch{
                state: FnTimerState::Stop,
                conditions: vec![
                    SwitchCondition {
                        condition: Box::new(|value| {value}),
                        target: FnTimerState::Start,
                    },
                    SwitchCondition {
                        condition: Box::new(|value| {!value}),
                        target: FnTimerState::Off,
                    },
                ],
            },
        ];
        Self { 
            input,
            state: SwitchState::new(FnTimerState::Off, switches),
            total: initial.into(),
            start: Instant::now(),
        }
    }
}

impl FnOutput<f64> for FnTimer {
    ///
    fn out(&mut self) -> f64 {
        // debug!("FnTimer.out | input: {:?}", self.input.print());
        let value = self.input.borrow_mut().out();
        self.state.add(value);
        let state = self.state.state();
        debug!("FnTimer.out | input.out: {:?}   |   state: {:?}", &value, &state);
        match state {
            FnTimerState::Off => {},
            FnTimerState::Start => {
                self.start = Instant::now();
            },
            FnTimerState::Progress => {
                self.total = self.start.elapsed().as_secs_f64();
            },
            FnTimerState::Stop => {
                self.total = self.start.elapsed().as_secs_f64();
            },
        };
        self.total
    }
}
