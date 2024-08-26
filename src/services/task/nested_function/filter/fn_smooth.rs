use std::sync::atomic::{AtomicUsize, Ordering};
use log::trace;
use crate::{
    conf::point_config::point_config_type::PointConfigType, core_::{
        point::point::Point, 
        types::fn_in_out_ref::FnInOutRef,
    }, services::task::nested_function::{
        fn_::{FnIn, FnInOut, FnOut},
        fn_kind::FnKind, fn_result::FnResult,
    }
};
///
/// Function | Returns smoothed input:
/// out = out + (input - prev) * factor
#[derive(Debug)]
pub struct FnSmooth {
    id: String,
    kind: FnKind,
    factor: FnInOutRef,
    input: FnInOutRef,
    value: Point,
}
//
// 
impl FnSmooth {
    ///
    /// Creates new instance of the FnSmooth
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, factor: FnInOutRef, input: FnInOutRef) -> Self {
        Self { 
            id: format!("{}/FnSmooth{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            factor,
            input,
            value: Point::new(0, "", 0.0),
        }
    }    
}
//
// 
impl FnIn for FnSmooth {}
//
// 
impl FnOut for FnSmooth { 
    //
    fn id(&self) -> String {
        self.id.clone()
    }
    //
    fn kind(&self) -> &FnKind {
        &self.kind
    }
    //
    fn inputs(&self) -> Vec<String> {
        let mut inputs = vec![];
        inputs.append(&mut self.factor.borrow().inputs());
        inputs.append(&mut self.input.borrow().inputs());
        inputs
    }
    //
    //
    fn out(&mut self) -> FnResult<Point, String> {
        let factor = self.factor.borrow_mut().out();
        trace!("{}.out | factor: {:?}", self.id, factor);
        let factor = match factor {
            FnResult::Ok(factor) => factor.to_double().as_double(),
            FnResult::None => return FnResult::None,
            FnResult::Err(err) => return FnResult::Err(err),
        };
        let input = self.input.borrow_mut().out();
        trace!("{}.out | input: {:?}", self.id, input);
        match input {
            FnResult::Ok(input) => {
                let input_type = input.type_();
                trace!("{}.out | factor: {:?}", self.id, factor);
                let delta = input.to_double().as_double() - self.value.to_double().as_double();
                trace!("{}.out | delta: {:?}", self.id, delta);
                let value = self.value.to_double().as_double() + delta * factor;
                trace!("{}.out | value: {:?}", self.id, value);
                let value = Point::Double(value);
                self.value = match input_type {
                    PointConfigType::Int => value.to_int(),
                    PointConfigType::Real => value.to_real(),
                    PointConfigType::Double => value.to_double(),
                    _ => panic!("{}.out | Illegal type of input {:?}", self.id, input_type),
                };
                trace!("{}.out | value: {:?}", self.id, self.value);
                FnResult::Ok(self.value.clone())
            }
            FnResult::None => FnResult::None,
            FnResult::Err(err) => FnResult::Err(err),
        }

    }
    //
    //
    fn reset(&mut self) {
        self.factor.borrow_mut().reset();
        self.input.borrow_mut().reset();
        self.value = Point::new(0, "", 0.0);
    }
}
//
// 
impl FnInOut for FnSmooth {}
///
/// Global static counter of FnSmooth instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
