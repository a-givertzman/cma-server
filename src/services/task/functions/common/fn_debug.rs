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
    inputs: Vec<(String, FnOutRef)>,
}
//
// 
impl FnDebug {
    ///
    /// ### Creates new instance of the `FnDebug`
    /// - `parent` - Идентификатор родительского узла
    /// - `inputs` - Список ссылок на зависимости (`FnOutRef`), значения которых требуется отслеживать.
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: impl IntoIterator<Item = (String, FnOutRef)>) -> Self {
        Self { 
            id: format!("{}/FnDebug{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            inputs: inputs.into_iter().collect(),
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
        self.inputs.iter()
            .flat_map(|(_, input)| input.borrow().inputs())
            .collect()
    }
    //
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let flow = FlowContext::new();
        for (name, input) in &self.inputs {
            match flow.ignore(input.borrow_mut().out()) {
                Ok(Some(v)) => {
                    log::debug!(
                        "{}.out | Value {} | {}:{}\n  └─ Val: {:?} | {:?} | {:?} | {}",
                        self.id, flow, v.txid(), v.name(), v.value(), v.status(), v.cot(), v.ts().format("%H:%M:%S%.3f")
                    );
                }
                Ok(None) => log::error!("{}.out | None on input '{}'", self.id, name),
                Err(err) => log::error!("{}.out | Error on input '{}': {:?}", self.id, name, err),
            }
        }
        Ok(None)
    }
    //
    //
    fn reset(&mut self) {
        for (_, input) in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
///
/// Global static counter of FnDebug instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
