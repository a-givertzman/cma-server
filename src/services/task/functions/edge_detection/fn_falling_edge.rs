use concat_string::concat_string;
use sal_core::error::Error;
use sal_sync::services::{entity::{Point, PointHlr}, types::Bool};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::{Edge, EdgeDetector, FnOutRef},
    services::task::{
        FlowContext, FnChange, FnFlow, FnKind, FnOut, FnResult, TryTo
    },
};
///
/// ### Function | `FnFallingEdge`
/// 
/// Детектор отричательного (заднего) фронта
/// 
/// - `input`: Последовательность `false -> true` - активирует выход на один такт
#[derive(Debug)]
pub struct FnFallingEdge {
    id: String,
    kind: FnKind,
    input: FnChange,
    edge: EdgeDetector,
    value: EdgeDetector,
}
//
impl FnFallingEdge {
    ///
    /// Creates new instance of the FnFallingEdge
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnOutRef) -> Self {
        Self { 
            id: format!("{}/FnFallingEdge{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            input: FnChange::new(input),
            edge: EdgeDetector::new(),
            value: EdgeDetector::new(),
        }
    }    
    ///
    /// Возвращает `PointHlr` с обновленными `name` и `value`
    #[inline]
    fn point_with<T>(p: &Point, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(p.txid(), name, value, p.status(), p.cot(), p.timestamp())
    }
}
//
// 
impl FnOut for FnFallingEdge { 
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
        self.input.inputs()
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let input = self.input.out();
        let mut flow = FlowContext::new();
        let Some(input) = flow.ignore(input)? else {
            self.edge.reset();
            return Ok(None);
        };
        let val: bool = (&input).try_to().map_err(|err: Error| concat_string!(self.id, ".out | Invalid input ", err.to_string()))?;
        let value = match self.edge.add(val) {
            Some(Edge::Falling) => true,
            _ => false,
        };
        let is_changed = self.value.add(value).is_some();
        let point = Point::Bool(Self::point_with(&input, &self.id, Bool(value)));
        // log::trace!("{}.out | value: {:#?}", self.id, point);
        if is_changed {
            flow.wrap_new(point)
        } else {
            flow.wrap_old(point)
        }
    }
    //
    fn reset(&mut self) {
        self.edge.reset();
        self.value.reset();
        self.input.reset();
    }
}
///
/// Global static counter of FnFallingEdge instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
