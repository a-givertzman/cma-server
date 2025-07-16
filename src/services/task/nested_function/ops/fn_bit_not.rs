use sal_sync::services::entity::point::point::Point;
use std::sync::atomic::{AtomicUsize, Ordering};
use log::trace;
use crate::{
    core_::types::fn_in_out_ref::FnInOutRef,
    services::task::nested_function::{
        fn_::{FnIn, FnInOut, FnOut}, fn_kind::FnKind, fn_result::FnResult,
    },

};
///
/// Function | Returns bitwise NOT of all inputs
/// 
/// Example
/// 
/// ```yaml
/// fn BitNot:
///     input: point int '/App/Service/Point.Name1'
/// fn BitNot:
///     input: point bool '/App/Service/Point.Name1'
/// ```
#[derive(Debug)]
pub struct FnBitNot {
    id: String,
    kind: FnKind,
    input: FnInOutRef,
}
//
// 
impl FnBitNot {
    ///
    /// Creates new instance of the FnBitNot
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnInOutRef) -> Self {
        Self { 
            id: format!("{}/FnBitNot{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind:FnKind::Fn,
            input,
        }
    }
}
//
// 
impl FnIn for FnBitNot {}
//
// 
impl FnOut for FnBitNot {
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
        self.input.borrow().inputs()
    }
    //
    fn out(&mut self) -> FnResult<Point, String> {
        let input = self.input.borrow_mut().out();
        trace!("{}.out | input: {:#?}", self.id, input);
        match input {
            FnResult::Ok(input) => {
                let value = match input {
                    Point::Bool(mut val) => {
                        val.value = ! val.value;
                        Point::Bool(val)
                    }
                    Point::Int(mut val) => {
                        val.value = ! val.value;
                        Point::Int(val)
                    }
                    Point::Real(_) => {
                        panic!("{}.out | Not implemented for Real", self.id);
                    }
                    Point::Double(_) => {
                        panic!("{}.out | Not implemented for Double", self.id);
                    }
                    Point::String(_) => {
                        panic!("{}.out | Not implemented for String", self.id);
                    }
                };
                // trace!("{}.out | value: {:#?}", self.id, value);
                FnResult::Ok(value)
            }
            FnResult::None => FnResult::None,
            FnResult::Err(err) => FnResult::Err(err),
        }
    }
    //
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
    }
}
//
// 
impl FnInOut for FnBitNot {}
///
/// Global static counter of FnBitNot instances
pub static COUNT: AtomicUsize = AtomicUsize::new(1);
