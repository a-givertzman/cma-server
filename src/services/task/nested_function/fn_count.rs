use std::sync::atomic::{AtomicUsize, Ordering};
use log::trace;
use sal_sync::services::entity::point::{point::Point, point_hlr::PointHlr};
use crate::core_::types::fn_in_out_ref::FnInOutRef;
use super::{fn_::{FnIn, FnInOut, FnOut}, fn_kind::FnKind, fn_result::FnResult};
///
/// Counts number of raised fronts of boolean input
#[derive(Debug)]
pub struct FnCount {
    id: String,
    kind: FnKind,
    input: FnInOutRef,
    prev: bool,
    count: Option<i64>,
    initial: Option<FnInOutRef>,
}
//
// 
impl FnCount {
    ///
    /// Creates new instance of the FnCount
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, initial: Option<FnInOutRef>, input: FnInOutRef) -> Self {
        Self { 
            id: format!("{}/FnCount{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind:FnKind::Fn,
            input,
            prev: false,
            count: None,
            initial,
        }
    }
}
//
// 
impl FnIn for FnCount {}
//
// 
impl FnOut for FnCount {
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
        inputs.append(&mut self.input.borrow().inputs());
        if let Some(initial) = &self.initial {
            inputs.append(&mut initial.borrow().inputs());
        }
        inputs
    }
    ///
    fn out(&mut self) -> FnResult<Point, String> {
        let mut count = match self.count {
            Some(count) => count,
            None => {
                match &mut self.initial {
                    Some(initial) => {
                        let initial = match initial.borrow_mut().out() {
                            FnResult::Ok(initial) => initial,
                            FnResult::None => return FnResult::None,
                            FnResult::Err(err) => return FnResult::Err(err),
                        };
                        initial
                            .try_as_int()
                            .unwrap_or_else(|_| {
                                panic!("{}.out | Initial must be of type 'Bool', but found '{:?}'", self.id, initial.type_())
                            })
                            .value
                    },
                    None => 0,
                }
            }
        };
        let input = self.input.borrow_mut().out();
        // trace!("{}.out | input: {:?}", self.id, self.input.print());
        match input {
            FnResult::Ok(input) => {
                let input_val = input.to_bool().as_bool().value.0;
                if !self.prev && input_val {
                    count += 1;
                    self.count = Some(count);
                }
                self.prev = input_val;
                trace!("{}.out | value: {:?}", self.id, count);
                FnResult::Ok(Point::Int(
                    PointHlr::new(
                        input.tx_id(),
                        &format!("{}.out", self.id),
                        count,
                        input.status(),
                        input.cot(),
                        input.timestamp(),
                    )
                ))
            }
            FnResult::None => FnResult::None,
            FnResult::Err(err) => FnResult::Err(err),
        }
    }
    fn reset(&mut self) {
        self.count = None;
        self.input.borrow_mut().reset();
    }
}
//
// 
impl FnInOut for FnCount {}
///
/// Global static counter of FnCount instances
pub static COUNT: AtomicUsize = AtomicUsize::new(1);
