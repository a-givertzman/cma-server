use std::sync::atomic::{Ordering, AtomicUsize};
use sal_sync::services::entity::Point;
use crate::services::task::FnFlow;

use super::{FnOut, FnKind, FnResult};
///
/// Function | Constant value
#[derive(Debug, Clone)]
pub struct FnConst {
    id: String,
    kind: FnKind,
    point: Point,
}
//
// 
impl FnConst {
    ///
    /// Creates new instance of function [Const] value
    ///     - [parent] - name of the parent object
    ///     - [value] - PointType, contains point with constant value
    pub fn new(parent: &str, value: Point) -> Self {
        Self {
            id: format!("{}/FnConst{}", parent, COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Input,
            point: value
        }
    }
}
//
// 
impl FnOut for FnConst {
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
        vec![]
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        log::trace!("{}.out | value: {:?}", self.id, &self.point);
        Ok(Some(FnFlow::Old(self.point.clone())))
    }
    //
    fn hard_reset(&mut self) {}
    fn reset(&mut self) {}
}
///
/// Global static counter of FnConst instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
