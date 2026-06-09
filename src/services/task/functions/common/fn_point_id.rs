use sal_sync::{collections::FxIndexMap, services::entity::{Point, PointConf, PointHlr, PointTxId}};
use std::{sync::atomic::{AtomicUsize, Ordering}};
use concat_string::concat_string;
use crate::{
    domain::FnOutRef, 
    services::task::{FlowContext, FnFlow, functions::{FnKind, FnOut, FnResult}},
};
///
/// ### Function | `FnPointId`
/// 
/// Возвращает ID входного сигнала по его имени из `RetainPointId`
/// - Является прозрачным узлом: сохраняет все метаданные входа (`Cot`, `Status`, `Timestamp`)
/// - Передает состояние потока (New/Old) без изменений.
/// 
/// **Example**
/// ```yaml
/// fn PointId:
///     input: point int /App/PointName
/// ```
#[derive(Debug)]
pub struct FnPointId {
    txid: usize,
    kind: FnKind,
    input: FnOutRef,
    points: FxIndexMap<String, usize>,
    id: String,
}
// 
impl FnPointId {
    ///
    /// Returns `FnPointId` new instance
    /// - `parent`: Идентификатор родительского узла
    /// - `input`: Входной сигнал (например `point any every`)
    /// - `points`: Список сигналов из `RetainPointId`
    pub fn new(parent: impl Into<String>, input: FnOutRef, points: Vec<PointConf>) -> Self {
        let id = format!("{}/FnPointId{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        Self { 
            txid: PointTxId::from_str(&id),
            kind: FnKind::Fn,
            input,
            points: points.into_iter().map(|p| (p.name, p.id)).collect(),
            id,
        }
    }    
    ///
    /// Возвращает `PointHlr` с обновленными `txid`, `name` и `value`
    #[inline]
    fn point_with<T>(txid: usize, name: impl Into<String>, p: &Point, value: T) -> PointHlr<T> {
        PointHlr::new(txid, name, value, p.status(), p.cot(), p.timestamp())
    }
}
// 
impl FnOut for FnPointId { 
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
        let Some(input) = flow.map(self.input.borrow_mut().out())? else { return Ok(None) };
        log::trace!("{}.out | input: {:?}", self.id, input);
        let Some(id) = self.points.get(&input.name()) else {
            return Err(concat_string!(self.id, ".out | Point '", input.name(), "' - not found in configured points"));
        };
        // log::debug!("{}.out | ID: {:?}", self.id, id);
        flow.wrap(Point::Int(Self::point_with(self.txid, &self.id, &input, *id as i64)))
    }
    //
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
    }
}
///
/// Global static counter of FnOut instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use sal_sync::services::entity::{Cot, PointConfHistory, PointType, Status};
    // Мок вышестоящего узла для управления выдачей данных в поток
    #[derive(Debug)]
    struct MockOut {
        pub next_value: Option<FnResult<FnFlow, String>>,
    }
    impl FnOut for MockOut {
        fn id(&self) -> String { "mock".to_string() }
        fn kind(&self) -> FnKind { FnKind::Fn }
        fn inputs(&self) -> Vec<String> { vec![] }
        fn out(&mut self) -> FnResult<FnFlow, String> {
            self.next_value.clone().unwrap_or(Ok(None))
        }
        fn reset(&mut self) {}
    }
    fn create_env(point_name: &str, setup_flow: FnFlow) -> (FnPointId, Rc<RefCell<MockOut>>) {
        let mock = Rc::new(RefCell::new(MockOut { next_value: Some(Ok(Some(setup_flow))) }));
        let config = vec![
            PointConf {id:42,
                name:point_name.to_string(),
                type_: PointType::Real,
                history: PointConfHistory::None,
                alarm: None,
                address: None,
                filters: None,
                comment: None,
            }
        ];
        let node = FnPointId::new("test_node", mock.clone(), config);
        (node, mock)
    }
    #[test]
    fn test_should_pass_new_flow_and_map_id() {
        let raw_point = Point::Real(PointHlr::new(1, "AI_01", 12.34, Status::Ok, Cot::Inf, chrono::Utc::now()));
        let (mut node, _) = create_env("AI_01", FnFlow::New(raw_point));
        let res = node.out().unwrap().unwrap();
        match res {
            FnFlow::New(Point::Int(hlr)) => {
                assert_eq!(hlr.value, 42);
                assert_eq!(hlr.status, Status::Ok);
            },
            _ => panic!("Expected FnFlow::New(Point::Int)"),
        }
    }
    #[test]
    fn test_should_preserve_old_flow_status() {
        let raw_point = Point::Real(PointHlr::new(1, "AI_01", 12.34, Status::Ok, Cot::Inf, chrono::Utc::now()));
        let (mut node, _) = create_env("AI_01", FnFlow::Old(raw_point));
        let res = node.out().unwrap().unwrap();
        match res {
            FnFlow::Old(Point::Int(hlr)) => {
                assert_eq!(hlr.value, 42);
            },
            _ => panic!("Expected FnFlow::Old(Point::Int)"),
        }
    }
    #[test]
    fn test_should_return_error_on_unknown_point() {
        let raw_point = Point::Real(PointHlr::new(1, "UNKNOWN_REG", 0.0, Status::Ok, Cot::Inf, chrono::Utc::now()));
        let (mut node, _) = create_env("AI_01", FnFlow::New(raw_point));
        let res = node.out();
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("not found in configured points"));
    }
    #[test]
    fn test_should_handle_none_flow_transparently() {
        let mock = Rc::new(RefCell::new(MockOut { next_value: Some(Ok(None)) }));
        let mut node = FnPointId::new("test_node", mock, vec![]);
        let res = node.out().unwrap();
        assert!(res.is_none());
    }
}
