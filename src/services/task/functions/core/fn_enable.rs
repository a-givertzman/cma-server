use sal_sync::services::entity::Point;

use crate::{domain::FnOutRef, services::task::{FlowContext, FnFlow}};
use super::{FnOut, FnKind, FnResult};
///
/// ### Function | Enable Decorator
/// 
/// Декоратор для управления активностью вычислительного узла.
///
/// Оборачивает любой узел, реализующий `FnOut`, добавляя логику включения/выключения
/// на основе управляющего сигнала `enable`.
/// 
/// Режимы работы:
/// - `Cold`: При выключении внутренний узел сбрасывается (`reset()`). При включении начинает работу с чистого листа.
/// - `Warm`: Внутренний узел вычисляется всегда (поддерживая актуальное состояние), но наружу значение передается только при активном сигнале.

/// - **`Cold`**: При выключении (`enable = false`) внутренний узел сбрасывается (`reset()`), при включении начинает работу с чистого листа.
/// - **`Warm`**: Внутренний узел вычисляется всегда (поддерживая актуальное состояние), но наружу значение передается только при `enable = true`.
///
/// > **Если enable не привязан или молчит (`None`), по умолчанию используем `fals` или предыдущее значение**
#[derive(Debug, Clone)]
pub struct FnEnable<T: FnOut> {
    origin: T,
    mode: FnEnableMode,
    enable: FnOutRef,
    prev_en: bool,
    last_val: Option<Point>,
}
//
impl<T: FnOut> FnEnable<T> {
    ///
    /// Creates a new instance of the FnEnable decorator
    /// - `origin` - Оборачиваемая вычислительная функция (`FnXyz`).
    /// - `mode` - Режим работы при отключении сигнала (Cold / Warm).
    /// - `en` - Ссылка на `enable`, выдающий логический сигнал активности.
    pub fn new(origin: T, mode: FnEnableMode, en: FnOutRef) -> Self {
        Self {
            origin,
            mode,
            enable: en,
            prev_en: false,
            last_val: None,
        }
    }
}
//
impl<T: FnOut> FnOut for FnEnable<T> {
    //
    fn id(&self) -> String {
        self.origin.id()
    }
    //
    fn kind(&self) -> FnKind {
        self.origin.kind()
    }
    //
    fn inputs(&self) -> Vec<String> {
        let mut inputs = vec![];
        inputs.append(&mut self.enable.borrow().inputs());
        inputs.append(&mut self.origin.inputs());
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let en = match self.enable.borrow_mut().out()? {
            Some(en) => en.into_value().to_bool().as_bool().value.0,
            None => self.prev_en, // Если enable не приходит (None), то используем прежнее значение
        };
        // Детектируем передний фронт (false -> true)
        let rising_edge = !self.prev_en && en;
        // Детектируем задний фронт (true -> false)
        let falling_edge = self.prev_en && !en;
        self.prev_en = en;
        match self.mode {
            FnEnableMode::Cold => {
                if en {
                    if rising_edge {
                        self.origin.reset();
                    }
                    let mut flow = FlowContext::new();
                    let Some(val) = flow.map(self.origin.out())? else { return Ok(None) };
                    self.last_val = Some(val.clone());
                    if rising_edge {
                        return Ok(Some(FnFlow::New(val)));
                    }
                    flow.wrap(val)
                } else {
                    if falling_edge {
                        self.origin.reset();
                        self.last_val = None; // Очищаем стейт для чистоты
                    }
                    // Cold означает полное отсутствие сигнала при выключении
                    Ok(None)
                }
            }
            FnEnableMode::Warm => {
                let mut flow = FlowContext::new();
                // Всегда дергаем оригинал, чтобы кэш/математика внутри оставались актуальными
                let val = flow.map(self.origin.out())?;
                if !en {
                    return Ok(self.last_val.clone().map(FnFlow::Old));
                }
                if let Some(val) = val {
                    self.last_val = Some(val.clone());
                    if rising_edge {
                        return Ok(Some(FnFlow::New(val)));
                    }
                    flow.wrap(val)
                } else {
                    Ok(None)
                }
            }
        }
    }
    //
    fn hard_reset(&mut self) {
        self.prev_en = false;
        self.last_val = None;
        self.enable.borrow_mut().hard_reset();
        self.origin.hard_reset();
    }
    //
    fn reset(&mut self) {
        self.prev_en = false;
        self.last_val = None;
    }
}
///
/// ### Enable Strategy
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FnEnableMode {
    /// **Cold Standby**: При enable = false внутреннее состояние полностью уничтожается.
    Cold,
    /// **Warm Standby**: Математика работает всегда, но enable работает как заслонка.
    Warm,
}
///
/// Basic tests
#[cfg(test)]
mod tests {
    use super::*;
    use sal_sync::services::entity::{Point, ToPoint};
    use std::{cell::RefCell, rc::Rc};
    #[derive(Debug)]
    struct MockFn {
        id: String,
        calls: usize,
        hard_resets: usize,
        resets: usize,
        flow_to_return: FnFlow,
    }
    impl MockFn {
        fn new(flow: FnFlow) -> Self {
            Self {
                id: "MockFn".to_string(),
                calls: 0,
                hard_resets: 0,
                resets: 0,
                flow_to_return: flow,
            }
        }
    }
    impl FnOut for MockFn {
        fn id(&self) -> String { self.id.clone() }
        fn kind(&self) -> FnKind { FnKind::Fn }
        fn inputs(&self) -> Vec<String> { vec![] }
        fn out(&mut self) -> FnResult<FnFlow, String> {
            self.calls += 1;
            Ok(Some(self.flow_to_return.clone()))
        }
        fn hard_reset(&mut self) { self.hard_resets += 1; }
        fn reset(&mut self) { self.resets += 1; }
    }
    fn bool_point(val: bool) -> Point {
        val.to_point(1, "test_en")
    }
    #[test]
    fn test_cold_mode() {
        let enable_mock = Rc::new(RefCell::new(MockFn::new(FnFlow::New(bool_point(false)))));
        let origin_mock = MockFn::new(FnFlow::Old(bool_point(true)));
        let mut fn_enable = FnEnable::new(origin_mock, FnEnableMode::Cold, enable_mock.clone());
        // 1. Старт с en=false. Ожидаем Ok(None)
        let out = fn_enable.out().unwrap();
        assert!(out.is_none(), "Cold mode must return None when disabled");
        assert_eq!(fn_enable.origin.calls, 0, "Origin must not be called in Cold mode when disabled");
        // 2. Rising edge (en=true)
        enable_mock.borrow_mut().flow_to_return = FnFlow::New(bool_point(true));
        let out = fn_enable.out().unwrap().unwrap();
        assert!(out.is_new(), "Rising edge must force FnFlow::New");
        assert_eq!(fn_enable.origin.hard_resets, 0, "Origin must not be hard reset on rising edge");
        assert_eq!(fn_enable.origin.resets, 1, "Origin must be reset on rising edge");
        assert_eq!(fn_enable.origin.calls, 1, "Origin must be evaluated");
        // 3. Steady high
        enable_mock.borrow_mut().flow_to_return = FnFlow::Old(bool_point(true)); // Старый enable
        let out = fn_enable.out().unwrap().unwrap();
        assert!(!out.is_new(), "Steady high with Old origin must return Old");
        // 4. Falling edge (en=false)
        enable_mock.borrow_mut().flow_to_return = FnFlow::New(bool_point(false));
        let out = fn_enable.out().unwrap();
        assert!(out.is_none(), "Falling edge in Cold mode must return None");
        assert_eq!(fn_enable.origin.resets, 2, "Origin must be reset on falling edge");
    }
    #[test]
    fn test_warm_mode() {
        let enable_mock = Rc::new(RefCell::new(MockFn::new(FnFlow::New(bool_point(true)))));
        let origin_mock = MockFn::new(FnFlow::Old(bool_point(true)));
        let mut fn_enable = FnEnable::new(origin_mock, FnEnableMode::Warm, enable_mock.clone());
        // 1. Старт с en=true
        let out = fn_enable.out().unwrap().unwrap();
        assert!(out.is_new(), "Rising edge forces FnFlow::New");
        assert_eq!(fn_enable.origin.calls, 1);
        // 2. Переход в en=false (Warm)
        enable_mock.borrow_mut().flow_to_return = FnFlow::New(bool_point(false));
        let out = fn_enable.out().unwrap().unwrap();
        assert!(!out.is_new(), "Disabled Warm mode must return Old flow");
        assert_eq!(fn_enable.origin.calls, 2, "Origin MUST be evaluated in Warm mode even if disabled");
        // 3. Убедимся, что reset не вызывался
        assert_eq!(fn_enable.origin.resets, 0, "Warm mode must never reset origin on edges");
    }
    #[test]
    fn test_flow_isolation() {
        // Убеждаемся, что FnFlow::New на управляющем сигнале не делает полезные данные 'New'
        let enable_mock = Rc::new(RefCell::new(MockFn::new(FnFlow::New(bool_point(true))))); // enable сигналит "New"
        let origin_mock = MockFn::new(FnFlow::Old(bool_point(true))); // Данные старые
        let mut fn_enable = FnEnable::new(origin_mock, FnEnableMode::Warm, enable_mock.clone());
        // Первый проход (rising edge) всегда дает New, поэтому пропустим его
        let _ = fn_enable.out(); 
        // Второй проход. Enable все еще сыпет New(true), данные Old
        let out = fn_enable.out().unwrap().unwrap();
        assert!(!out.is_new(), "FnFlow::New from enable must not pollute the FlowContext of the origin");
    }
}