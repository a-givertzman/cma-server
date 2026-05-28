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
//
impl<T: FnOut> FnEnable<T> {
    ///
    /// Creates a new instance of the FnEnable decorator
    /// - `origin` - Оборачиваемая вычислительная функция (`FnXyz`).
    /// - `mode` - Режим работы при отключении сигнала (Cold / Warm).
    /// - `enable` - Ссылка на `enable`, выдающий логический сигнал активности.
    pub fn new(origin: T, mode: FnEnableMode, enable: FnOutRef) -> Self {
        Self {
            origin,
            mode,
            enable,
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
        let mut flow = FlowContext::new();
        let en = match flow.map(self.enable.borrow_mut().out())? {
            Some(en) => en.to_bool().as_bool().value.0,
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
                    let Some(val) = flow.map(self.origin.out())? else { return Ok(None) };
                    self.last_val = Some(val.clone());
                    if rising_edge {
                        return Ok(Some(FnFlow::New(val)));
                    }
                    flow.wrap(val)
                } else {
                    if falling_edge {
                        self.origin.reset();
                    }
                    Ok(self.last_val.clone().map(FnFlow::Old))
                }
            }
            FnEnableMode::Warm => {
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
    fn reset(&mut self) {
        self.prev_en = false;
        self.last_val = None;
        self.enable.borrow_mut().reset();
        self.origin.reset();
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
    use std::{cell::{Cell, RefCell}, rc::Rc};
    use sal_sync::services::entity::{Point, ToPoint};
    use crate::services::task::{FnFlow, FnKind, FnResult};
    #[derive(Debug)]
    struct FakeEnable {
        val: bool,
    }
    impl FakeEnable {
        fn new(val: bool) -> Rc<RefCell<Self>> {
            Rc::new(RefCell::new(Self { val }))
        }
    }
    impl FnOut for FakeEnable {
        fn id(&self) -> String { "fake_en".into() }
        fn kind(&self) -> FnKind { FnKind::Var }
        fn inputs(&self) -> Vec<String> { vec![] }
        fn out(&mut self) -> FnResult<FnFlow, String> {
            Ok(Some(FnFlow::New(self.val.to_point(0, "en"))))
        }
        fn reset(&mut self) {}
    }
    #[derive(Debug)]
    struct FakeOrigin {
        val: Option<FnFlow>,
        reset_count: Rc<Cell<usize>>,
    }
    impl FakeOrigin {
        fn new(reset_count: Rc<Cell<usize>>) -> Self {
            Self { val: None, reset_count }
        }
    }
    impl FnOut for FakeOrigin {
        fn id(&self) -> String { "fake_origin".into() }
        fn kind(&self) -> FnKind { FnKind::Fn }
        fn inputs(&self) -> Vec<String> { vec![] }
        fn out(&mut self) -> FnResult<FnFlow, String> { Ok(self.val.clone()) }
        fn reset(&mut self) {
            self.reset_count.set(self.reset_count.get() + 1);
        }
    }
    #[test]
    fn test_fn_enable_cold_mode() {
        let enable = FakeEnable::new(false);
        let resets = Rc::new(Cell::new(0));
        let origin = FakeOrigin::new(resets.clone());
        let mut fn_enable = FnEnable::new(origin, FnEnableMode::Cold, enable.clone());
        let out = fn_enable.out().unwrap();
        assert!(out.is_none(), "Должен вернуть None, так как сигнал изначально опущен");
        assert_eq!(resets.get(), 0);
        enable.borrow_mut().val = true;
        fn_enable.origin.val = Some(FnFlow::New(42_i64.to_point(0, "val")));
        let out = fn_enable.out().unwrap().unwrap();
        assert!(out.is_new(), "Low -> High: должно вернуться New");
        if let Point::Int(p) = out.value() { assert_eq!(p.value, 42); }
        assert_eq!(resets.get(), 1, "При rising_edge в Cold режиме должен быть вызван reset");
        fn_enable.origin.val = Some(FnFlow::Old(43_i64.to_point(0, "val")));
        let out = fn_enable.out().unwrap().unwrap();
        assert!(!out.is_new(), "Сквозной проброс Old");
        assert_eq!(resets.get(), 1, "При стабильном сигнале reset не вызывается");
        enable.borrow_mut().val = false;
        let out = fn_enable.out().unwrap().unwrap();
        assert!(!out.is_new(), "High -> Low: должно вернуться Old");
        if let Point::Int(p) = out.value() { assert_eq!(p.value, 43, "Должно сохраниться последнее валидное значение"); }
        assert_eq!(resets.get(), 2, "При falling_edge в Cold режиме должен быть вызван reset");
    }
    #[test]
    fn test_fn_enable_warm_mode() {
        let enable = FakeEnable::new(false);
        let resets = Rc::new(Cell::new(0));
        let origin = FakeOrigin::new(resets.clone());
        let mut fn_enable = FnEnable::new(origin, FnEnableMode::Warm, enable.clone());
        fn_enable.origin.val = Some(FnFlow::New(10_i64.to_point(0, "val")));
        let out = fn_enable.out().unwrap();
        assert!(out.is_none(), "В Warm режиме при опущенном сигнале отдаем last_val (который None)");
        assert_eq!(resets.get(), 0);
        enable.borrow_mut().val = true;
        fn_enable.origin.val = Some(FnFlow::New(20_i64.to_point(0, "val")));
        let out = fn_enable.out().unwrap().unwrap();
        assert!(out.is_new());
        if let Point::Int(p) = out.value() { assert_eq!(p.value, 20); }
        assert_eq!(resets.get(), 0, "В Warm режиме reset не вызывается никогда");
        enable.borrow_mut().val = false;
        fn_enable.origin.val = Some(FnFlow::New(30_i64.to_point(0, "val")));
        let out = fn_enable.out().unwrap().unwrap();
        assert!(!out.is_new());
        if let Point::Int(p) = out.value() { assert_eq!(p.value, 20, "High -> Low: отдаем last_val, игнорируя внутренние вычисления"); }
        assert_eq!(resets.get(), 0);
    }
    #[test]
    fn test_fn_enable_warm_mode_none_handling() {
        let enable = FakeEnable::new(true);
        let resets = Rc::new(Cell::new(0));
        let origin = FakeOrigin::new(resets.clone());
        let mut fn_enable = FnEnable::new(origin, FnEnableMode::Warm, enable.clone());
        fn_enable.origin.val = Some(FnFlow::New(100_i64.to_point(0, "val")));
        let out = fn_enable.out().unwrap().unwrap();
        assert!(out.is_new());
        fn_enable.origin.val = None;
        let out = fn_enable.out().unwrap();
        assert!(out.is_none(), "Если оригинальная функция возвращает None, мы также возвращаем None, не трогая last_val");
    }
}