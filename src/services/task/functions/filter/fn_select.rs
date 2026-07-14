use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use sal_sync::services::entity::PointType;
use crate::{
    domain::FnOutRef,
    services::task::{FnFlow, FnKind, FnOut, FnResult},
};
///
/// ### Function | `Select`
/// 
/// Классический мультиплексор (чистая функция).
/// - Возвращает значение `input`, если `select = true` или `> 0`.
/// - Возвращает значение `default`, если `select = false` или `<= 0`.
/// - Если `default` не задан, а `select = false`, возвращает `Ok(None)` (обрыв потока).
#[derive(Debug)]
pub struct FnSelect {
    id: String,
    kind: FnKind,
    default: Option<FnOutRef>,
    input: FnOutRef,
    select: FnOutRef,
}
//
impl FnSelect {
    ///
    /// ### Creates new instance of the FnSelect
    /// * `parent` - Идентификатор родительского узла.
    /// * `default` - Входной сигнал (select = 0). Опционален.
    /// * `input` - Входной сигнал (select = 1).
    /// * `select` - Управляющий логический или числовой сигнал.
    pub fn new(parent: impl Into<String>, default: Option<FnOutRef>, input: FnOutRef, select: FnOutRef) -> Self {
        let self_id = format!("{}/FnSelect{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        Self {
            id: self_id,
            kind: FnKind::Fn,
            default,
            input,
            select,
        }
    }
}
//
impl FnOut for FnSelect {
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
        inputs.append(&mut self.select.borrow().inputs());
        inputs.append(&mut self.input.borrow().inputs());
        if let Some(default) = &self.default {
            inputs.append(&mut default.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let input = self.input.borrow_mut().out();
        let default = self.default.as_mut().map(|f| f.borrow_mut().out());
        let select = self.select.borrow_mut().out();
        let Some(select) = select? else { return Ok(None) };
        let select = select.into_value();
        log::trace!("{}.out | select: {:?}", self.id, select);
        let is_selected = match select.typ() {
            PointType::Bool | PointType::Int | PointType::Real | PointType::Double => select.to_bool().as_bool().value.0,
            _ => return Err(concat_string!(self.id, ".out | Invalid select type '", select.typ().to_string(), "'")),
        };
        if is_selected {
            let Some(input) = input? else { return Ok(None) };
            log::trace!("{}.out | input value: {:?}", self.id, input);
            Ok(Some(input))
        } else {
            if let Some(default) = default {
                let Some(default) = default? else { return Ok(None) };
                log::trace!("{}.out | default value: {:?}", self.id, default);
                Ok(Some(default))
            } else {
                Ok(None)
            }
        }        
    }
    //
    fn hard_reset(&mut self) {
        if let Some(default) = &self.default {
            default.borrow_mut().hard_reset();
        }
        self.input.borrow_mut().hard_reset();
        self.select.borrow_mut().hard_reset();
    }
    //
    fn reset(&mut self) {}
}
///
/// Global static counter of FnSelect instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
#[cfg(test)]
mod tests {
    use sal_sync::services::entity::{Cot, Point, PointHlr, Status};

use super::*;
    use crate::{
        domain::FnOutRef,
        services::task::{FnFlow, FnKind, FnOut, FnResult},
    };
    use std::{cell::RefCell, rc::Rc}; // Замени на нужную обертку (Arc/Mutex), если твой FnOutRef устроен иначе
    ///
    /// ### Spy Node
    /// Узел-шпион для проверки ленивых вычислений (Lazy Evaluation).
    /// Считает количество вызовов метода `out()`.
    #[derive(Debug)]
    struct SpyNode {
        id: String,
        call_count: usize,
        mock_flow: Option<FnFlow>,
    }
    impl SpyNode {
        fn new(id: impl Into<String>, mock_flow: Option<FnFlow>) -> Rc<RefCell<Self>> {
            Rc::new(RefCell::new(Self {
                id: id.into(),
                call_count: 0,
                mock_flow,
            }))
        }
        fn calls(&self) -> usize {
            self.call_count
        }
    }
    impl FnOut for SpyNode {
        fn id(&self) -> String { self.id.clone() }
        fn kind(&self) -> FnKind { FnKind::Fn }
        fn inputs(&self) -> Vec<String> { vec![] }
        fn out(&mut self) -> FnResult<FnFlow, String> {
            self.call_count += 1;
            Ok(self.mock_flow.take()) // Забираем закэшированный флоу
        }
        fn hard_reset(&mut self) {}
        fn reset(&mut self) {}
    }
    // Вспомогательная функция для генерации фейковых FnFlow::New (замени на свой стандартный генератор из тестов)
    fn mock_point(val: f64) -> FnFlow {
        FnFlow::New(
            Point::Double(PointHlr::new(1, "test", val, Status::Ok, Cot::Inf, chrono::Utc::now()))
        )
    }
    //
    #[test]
    fn test_select_true_evaluates_input_only() {
        let select_node = SpyNode::new("select", Some(mock_point(1.0))); // select > 0 (true)
        let input_node = SpyNode::new("input", Some(mock_point(42.0)));
        let default_node = SpyNode::new("default", Some(mock_point(0.0)));
        let mut fn_select = FnSelect::new(
            "parent",
            Some(default_node.clone() as FnOutRef),
            input_node.clone() as FnOutRef,
            select_node.clone() as FnOutRef,
        );
        let result = fn_select.out().unwrap();
        assert!(result.is_some(), "Узел должен вернуть значение");
        assert_eq!(input_node.borrow().calls(), 1, "Активная ветка (input) должна быть опрошена 1 раз");
        assert_eq!(default_node.borrow().calls(), 1, "Спящая ветка (default) должна быть опрошена 1 раз");
    }
    //
    #[test]
    fn test_select_negative_value_evaluates_default_only() {
        let select_node = SpyNode::new("select", Some(mock_point(-5.0))); // select <= 0 (false)
        let input_node = SpyNode::new("input", Some(mock_point(42.0)));
        let default_node = SpyNode::new("default", Some(mock_point(99.0)));
        let mut fn_select = FnSelect::new(
            "parent",
            Some(default_node.clone() as FnOutRef),
            input_node.clone() as FnOutRef,
            select_node.clone() as FnOutRef,
        );
        let result = fn_select.out().unwrap();
        assert!(result.is_some(), "Узел должен вернуть значение default");
        assert_eq!(input_node.borrow().calls(), 1, "Спящая ветка (input) должна быть опрошена 1 раз");
        assert_eq!(default_node.borrow().calls(), 1, "Активная ветка (default) должна быть опрошена 1 раз");
    }
    //
    #[test]
    fn test_select_none_aborts_flow_immediately() {
        let select_node = SpyNode::new("select", None); // Обрыв управляющего сигнала
        let input_node = SpyNode::new("input", Some(mock_point(42.0)));
        let default_node = SpyNode::new("default", Some(mock_point(99.0)));
        let mut fn_select = FnSelect::new(
            "parent",
            Some(default_node.clone() as FnOutRef),
            input_node.clone() as FnOutRef,
            select_node.clone() as FnOutRef,
        );
        let result = fn_select.out().unwrap();
        assert!(result.is_none(), "При обрыве select узел должен вернуть обрыв (None)");
        assert_eq!(input_node.borrow().calls(), 1, "При обрыве должна быть опрошена 1 раз");
        assert_eq!(default_node.borrow().calls(), 1, "При обрыве должна быть опрошена 1 раз");
    }
    //
    #[test]
    fn test_select_false_without_default_returns_none() {
        let select_node = SpyNode::new("select", Some(mock_point(0.0))); // select = 0 (false)
        let input_node = SpyNode::new("input", Some(mock_point(42.0)));
        let mut fn_select = FnSelect::new(
            "parent",
            None, // default ветка отсутствует
            input_node.clone() as FnOutRef,
            select_node.clone() as FnOutRef,
        );
        let result = fn_select.out().unwrap();
        assert!(result.is_none(), "При select=false и отсутствии default узел должен вернуть обрыв потока (None)");
        assert_eq!(input_node.borrow().calls(), 1, "При переходе в несуществующую ветку input не должен опрашиваться");
    }
}
