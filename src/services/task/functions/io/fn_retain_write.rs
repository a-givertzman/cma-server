use function_name::named;
use sal_core::error::Error;
use sal_sync::services::entity::{Name, {Point, PointTxId}};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{domain::{FnOutRef, Sender}, services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult, RetainEvent}};
///
/// ### Function | `FnRetainWrite`
/// 
/// Запись значения `Point` на диск.
/// Пишет по изменению значения, статуса или метки времени.
/// Для вычислений прозрачна, не вносит изменений в поток.
/// 
/// **Особенности работы:**
/// - `enable`: (Через `FnEnable`) При значении `false` (или 0) прерывает передачу данных (возвращает `None`).
/// - `default`: Запасной источник данных (вычисляется лениво),
///    если входа нет или он молчит (`Ok(None)`), то узел попытается вернуть и записать на диск `default`.
///
/// **Пример:**
/// ```yaml
/// fn Retain:            # Тут происходит запись
///     key: 'OperatingCycleId'
///     input fn Acc:
///         initial fn Retain:
///             default: const int 0
///             key: 'OperatingCycleId'
///         input: opCycleIsDone
/// ```

#[derive(Debug)]
pub struct FnRetainWrite {
    txid: usize,
    kind: FnKind,
    key: String,
    default: Option<FnOutRef>,
    input: Option<FnOutRef>,
    cache: Option<Point>,
    send: Sender<RetainEvent>,
    id: String,
}
//
impl FnRetainWrite {
    ///
    /// ### Returns `FnRetainWrite` new instance
    /// - `parent`: Идентификатор родительского узла
    /// - `path` Путь к retain файлу для значения (`assets/retain/App/RecorderTask/retain_value.json`)
    /// - `default`: Запасной источник данных (вычисляется лениво),
    ///    если входа нет или он молчит (`Ok(None)`), то узел попытается вернуть и записать на диск `default`.
    /// - `inputs`: Входной сигнал
    #[named]
    pub fn new(parent: &Name, send: Sender<RetainEvent>, key: impl Into<String>, default: Option<FnOutRef>, input: Option<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnRetainWrite{}", parent.join(), COUNT.fetch_add(1, Ordering::Relaxed));
        Ok(Self {
            txid: PointTxId::from_str(&id),
            kind: FnKind::Fn,
            key: key.into(),
            default,
            input,
            cache: None,
            send,
            id,
        })
    }
}
//
impl FnOut for FnRetainWrite {
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
        if let Some(default) = &self.default {
            inputs.append(&mut default.borrow().inputs());
        }
        if let Some(input) = &self.input {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let input = self.input.as_ref().map(|f| f.borrow_mut().out());
        if let Some(input) = input {
            if let Some(input) = flow.map(input)? {
                let name = input.name();
                if let Err(err) = self.send.send(RetainEvent::new(self.key.clone(), input.clone())) {
                    log::error!("{}.out | Can't store value '{}': {:?}", self.id, name, err);
                }
                return flow.wrap(input);
            }
        }
        if let Some(default) = self.default.as_ref() {
            if let Some(default) = flow.map(default.borrow_mut().out())? {
                let name = default.name();
                if let Err(err) = self.send.send(RetainEvent::new(self.key.clone(), default.clone())) {
                    log::error!("{}.out | Can't store value '{}': {:?}", self.id, name, err);
                }
                return flow.wrap(default);
            }
        }
        Ok(None)
    }
    //
    fn reset(&mut self) {
        if let Some(default) = &self.default {
            default.borrow_mut().reset();
        }
        if let Some(input) = &self.input {
            input.borrow_mut().reset();
        }
        self.cache = None;
    }
}
///
/// Global static counter of FnRetainWrite instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
