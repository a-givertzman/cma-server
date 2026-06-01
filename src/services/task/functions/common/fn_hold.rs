use sal_sync::services::entity::Point;
use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::{FnOutRef},
    services::task::{FlowContext, FnFlow, functions::{
        FnKind, FnOut, FnResult
    }},
};
///
/// ### Function | Returns last good value, was `FnKeepValid`
/// - Удерживает последнее валидное значение оперативно
/// - Если источник возвращает `None` (отключен или молчит), отдает последнее сохраненное значение как `Old`.
/// - Если значение не изменилось, но источник активен, подавляет флаг и отдает `Old`.
#[derive(Debug)]
pub struct FnHold {
    id: String,
    kind: FnKind,
    input: FnOutRef,
    state: Option<Point>,
}
// 
impl FnHold {
    ///
    /// Creates new instance of the FnHold
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnOutRef) -> Self {
        Self { 
            id: format!("{}/FnHold{}", parent.into(), COUNT.fetch_add(1, Ordering::AcqRel)),
            kind: FnKind::Fn,
            input,
            state: None,
        }
    }    
}
// 
impl FnOut for FnHold { 
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
        self.input.borrow().inputs()
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let input = flow.map(self.input.borrow_mut().out())?;
        log::trace!("{}.out | input: {:?}", self.id, input);
        if let Some(point) = input {
            let is_changed = match &self.state {
                Some(prev) => prev.value() != point.value(),
                None => true,
            };
            self.state = Some(point.clone());
            if is_changed {
                return flow.wrap_new(point);
            } else {
                return flow.wrap_old(point);
            }
        }
        if let Some(point) = &self.state {
            return flow.wrap_old(point.clone());
        }
        Ok(None)
    }
    //
    fn reset(&mut self) {
        self.state = None;
        self.input.borrow_mut().reset();
    }
}
///
/// Global static counter of FnHold instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
