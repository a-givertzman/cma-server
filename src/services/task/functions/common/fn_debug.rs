use std::sync::atomic::{AtomicUsize, Ordering};
use function_name::named;

use crate::{
    domain::FnOutRef, err_pass, services::task::{
        FlowContext, FnFlow, FnKind, FnOut, FnResult
    }
};
///
/// ### Function | `Debug`
/// 
/// Log values coming from inputs
///
/// Узел для отладки потока данных (Taint Tracking) в графе вычислений.
/// Перехватывает вызовы `out()` своих зависимостей, логирует актуальные значения 
/// и их статус (New/Old)
/// 
/// Если вход один, то возвращает его как есть, если больше одного, то возвращая `Ok(None)`.
#[derive(Debug)]
pub struct FnDebug {
    id: String,
    kind: FnKind,
    inputs: Vec<(String, FnOutRef)>,
}
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
    #[named]
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        if self.inputs.len() > 1 {
            for (name, input) in &self.inputs {
                match flow.ignore(input.borrow_mut().out()) {
                    Ok(Some(v)) => {
                        log::debug!(
                            "{}.out | {name}: Value {} | {}:{}\n  └─ Val: {:?} | {:?} | {:?} | {}",
                            self.id, flow, v.txid(), v.name(), v.value(), v.status(), v.cot(), v.ts().format("%H:%M:%S%.3f")
                        );
                    }
                    Ok(None) => log::warn!("{}.out | '{name}': None", self.id),
                    Err(err) => log::error!("{}.out | '{name}': {:?}", self.id, err),
                }
            }
            return Ok(None);
        }
        if let Some((name, input)) = self.inputs.first() {
            match flow.map(input.borrow_mut().out()) {
                Ok(Some(v)) => {
                    log::debug!(
                        "{}.out | {name}: Value {} | {}:{}\n  └─ Val: {:?} | {:?} | {:?} | {}",
                        self.id, flow, v.txid(), v.name(), v.value(), v.status(), v.cot(), v.ts().format("%H:%M:%S%.3f")
                    );
                    return flow.wrap(v);
                }
                Ok(None) => {
                    log::debug!("{}.out | {name}: Value {} | ---:---\n  └─ Val: --- | Ok(None)", self.id, flow);
                    return Ok(None);
                }
                Err(err) => {
                    log::debug!("{}.out | {name}: Value {} | ---:---\n  └─ Val: --- | Err({})", self.id, flow, err);
                    return Err(err_pass!(self.id, err).to_string());
                }
            }
        }
        Ok(None)
    }
    //
    fn hard_reset(&mut self) {
        for (_, input) in &self.inputs {
            input.borrow_mut().hard_reset();
        }
    }
    //
    fn reset(&mut self) {}
}
///
/// Global static counter of FnDebug instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
