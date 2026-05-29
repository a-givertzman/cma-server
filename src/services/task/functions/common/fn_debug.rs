use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::FnOutRef,
    services::task::{
        FlowContext, FnFlow, FnKind, FnOut, FnResult
    },
};
///
/// ### Function | Debug values coming from inputs
///
/// Узел для отладки потока данных (Taint Tracking) в графе вычислений.
/// Перехватывает вызовы `out()` своих зависимостей, логирует актуальные значения 
/// и их статус (New/Old)
/// 
/// Возвращая `Ok(None)`.
#[derive(Debug)]
pub struct FnDebug {
    id: String,
    kind: FnKind,
    inputs: Vec<FnOutRef>,
}
//
// 
impl FnDebug {
    ///
    /// ### Creates new instance of the `FnDebug`
    /// - `parent` - Идентификатор родительского узла
    /// - `inputs` - Список ссылок на зависимости (`FnOutRef`), значения которых требуется отслеживать.
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Self {
        Self { 
            id: format!("{}/FnDebug{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            inputs,
        }
    }    
}
//
// 
impl FnOut for FnDebug { 
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
        self.inputs.iter().flat_map(|input| input.borrow().inputs()).collect()
    }
    //
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        for input in &self.inputs {
            let Some(v) = flow.map(input.borrow_mut().out())? else { return Ok(None) };
            log::debug!(
                "{}.out | Value {} | {}:{}\n  └─ Val: {:?} | {:?} | {:?} | {}",
                self.id, flow, v.txid(), v.name(), v.value(), v.status(), v.cot(), v.timestamp().format("%H:%M:%S%.3f")
            );
        }
        Ok(None)
    }
    //
    //
    fn reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
///
/// Global static counter of FnDebug instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
