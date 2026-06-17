use std::sync::atomic::{AtomicUsize, Ordering};
use sal_core::error::Error;
use sal_sync::services::entity::{Point, PointType};
use crate::{
    domain::FnOutRef, services::task::{
        FlowContext, FnFlow, FnKind, FnOut, FnResult
    }
};
///
/// Function | EMA (Exponential Moving Average)
/// - Returns smoothed input:
/// - out = out + (input - prev) * factor
#[derive(Debug)]
pub struct FnSmooth {
    kind: FnKind,
    factor: FnOutRef,
    input: FnOutRef,
    value: Point,
    id: String,
}
//
// 
impl FnSmooth {
    ///
    /// Creates new instance of the FnSmooth
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, factor: FnOutRef, input: FnOutRef) -> Result<Self, Error> {
        let id = format!("{}/FnSmooth{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        return Err(Error::new(&id, "new").err("Isn't implemented yet"));
        Ok(Self { 
            kind: FnKind::Fn,
            factor,
            input,
            value: Point::new(0, "", 0.0),
            id,
        })
    }    
}
//
// 
impl FnOut for FnSmooth { 
    //
    fn id(&self) -> String {
        self.id.clone()
    }
    //
    fn kind(&self) -> FnKind {
        self.kind
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
    fn out(&mut self) -> FnResult<FnFlow, String> {
        unimplemented!();
        // let mut flow = FlowContext::new();
        // let factor = self.factor.borrow_mut().out();
        // log::trace!("{}.out | factor: {:?}", self.id, factor);
        // let factor = match factor {
        //     FnResult::Ok(factor) => factor.to_double().as_double(),
        //     FnResult::None => return FnResult::None,
        //     FnResult::Err(err) => return FnResult::Err(err),
        // };
        // let input = self.input.borrow_mut().out();
        // log::trace!("{}.out | input: {:?}", self.id, input);
        // match input {
        //     FnResult::Ok(input) => {
        //         let input_type = input.typ();
        //         log::trace!("{}.out | factor: {:?}", self.id, factor);
        //         let delta = input.to_double().as_double() - self.value.to_double().as_double();
        //         log::trace!("{}.out | delta: {:?}", self.id, delta);
        //         let value = self.value.to_double().as_double() + delta * factor;
        //         log::trace!("{}.out | value: {:?}", self.id, value);
        //         let value = Point::Double(value);
        //         self.value = match input_type {
        //             PointType::Int => value.to_int(),
        //             PointType::Real => value.to_real(),
        //             PointType::Double => value.to_double(),
        //             _ => panic!("{}.out | Illegal type of input {:?}", self.id, input_type),
        //         };
        //         log::trace!("{}.out | value: {:?}", self.id, self.value);
        //         FnResult::Ok(self.value.clone())
        //     }
        //     FnResult::None => FnResult::None,
        //     FnResult::Err(err) => FnResult::Err(err),
        // }
    }
    //
    //
    fn reset(&mut self) {
        self.factor.borrow_mut().reset();
        self.input.borrow_mut().reset();
        self.value = Point::new(0, "", 0.0);
    }
}
///
/// Global static counter of FnSmooth instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
