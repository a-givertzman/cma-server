use std::fmt::{Debug, Display};

use sal_sync::services::entity::Point;

use crate::services::task::FnResult;

///
/// ### Контракт потока данных
/// 
/// - Оборачивает `Point` в вычислениях `Task`.
/// - Определяет статус входных и выходных значений.
///     - `New` - Точка была обновлена в текущем цикле вычислений (пришел эвент)
///     - `Old` - Точка взята из кэша, в этом цикле эвента для нее не было
#[derive(Debug, Clone)]
pub enum FnFlow {
    /// Точка была обновлена в текущем цикле вычислений (пришел эвент)
    New(Point),
    /// Точка взята из кэша, в этом цикле эвента для нее не было
    Old(Point),
}

impl FnFlow {
    ///
    /// Возвращает ссылку на `Point`
    pub fn value(&self) -> &Point {
        match self {
            FnFlow::New(p) => p,
            FnFlow::Old(p) => p,
        }
    }
    ///
    /// Возвращает `Point`
    pub fn into_value(self) -> Point {
        match self {
            FnFlow::New(p) => p,
            FnFlow::Old(p) => p,
        }
    }
    ///
    /// Возвращает `true` если `FnFlow::New`
    pub fn is_new(&self) -> bool {
        match self {
            FnFlow::New(_) => true,
            FnFlow::Old(_) => false,
        }
    }
}

///
///
/// ### Контекст вычисления узла.
/// Аккумулирует признак активности данных (is_new) при обходе входов,
/// позволяя финальному узлу корректно транслировать статус обновления.
pub struct FlowContext {
    is_new: bool,
}
//
impl FlowContext {
    ///
    /// Returns `FlowContext` new instance
    pub fn new() -> Self {
        Self {
            is_new: false,
        }
    }
    ///
    /// ### Пропускает `FnResult<FnFlow>`
    /// - Игнорирует `FnFlow`
    /// - Извлекает чистый `Point` для дальнейшей бизнес-логики.
    pub fn ignore(&self, v: FnResult<FnFlow, String>) -> FnResult<Point, String> {
        let Some(flow) = v? else { return Ok(None) };
        Ok(Some(flow.into_value()))
    }
    ///
    /// ### Пропускает через себя `FnResult<FnFlow>`
    /// - Фиксирует `FnFlow::New`
    /// - Извлекает чистый `Point` для дальнейшей бизнес-логики.
    pub fn map(&mut self, v: FnResult<FnFlow, String>) -> FnResult<Point, String> {
        let Some(flow) = v? else { return Ok(None) };
        self.is_new |= flow.is_new();
        Ok(Some(flow.into_value()))
    }
    ///
    /// ### Оборачивает итоговый `Point` обратно в `FnFlow`, 
    /// - Учитывая историю опроса всех входов в текущем контексте.
    /// - Возвращает `Ok(Some(FnFlow(p)))`
    pub fn wrap(&self, p: Point) -> FnResult<FnFlow, String> {
        Ok(Some(match self.is_new {
            true => FnFlow::New(p),
            false => FnFlow::Old(p)
        }))
    }
    ///
    /// ### Принудительно оборачивает итоговый `Point` в `FnFlow::New`, 
    /// - Учитывая историю опроса всех входов в текущем контексте.
    /// - Возвращает `Ok(Some(FnFlow(p)))`
    pub fn wrap_new(&self, p: Point) -> FnResult<FnFlow, String> {
        Ok(Some(FnFlow::New(p)))
    }
    ///
    /// ### Принудительно оборачивает итоговый `Point` в `FnFlow::Old`, 
    /// - Возвращает `Ok(Some(FnFlow(p)))`
    pub fn wrap_old(&self, p: Point) -> FnResult<FnFlow, String> {
        Ok(Some(FnFlow::Old(p)))
    }
    ///
    /// ### Возвращает `true` если `FnFlow::New` зарегистрирован
    /// - Это означае, что один из входов вернул новое значение
    pub fn is_new(&self) -> bool {
        self.is_new
    }
    ///
    /// ### Возвращает `true` если `FnFlow::New` не было
    /// - Все входы вернули устаревшее значение
    pub fn is_old(&self) -> bool {
        !self.is_new
    }
}
//
impl Display for FlowContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_new {
            write!(f, "Flow::New")
        } else {
            write!(f, "Flow::Old")
        }
    }
}
//
impl Debug for FlowContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
