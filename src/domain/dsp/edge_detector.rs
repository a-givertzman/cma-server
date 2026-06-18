///
/// ### Детектор фронтов (Edge Detector)
/// - Накапливает значения `bool` или числовые типы.
/// - Фиксирует переход 0 -> 1 как передний фронт (Rising).
/// - Фиксирует переход 1 -> 0 как задний фронт (Falling).
#[derive(Debug, Clone, Default)]
pub struct EdgeDetector {
    init: Option<bool>,
    prev: Option<bool>,
    edge: Option<Edge>,
    is_high: bool,
}
impl EdgeDetector {
    ///
    /// Создает `EdgeDetection` с неопределенным исходным состоянием
    pub fn new() -> Self {
        Self::default()
    }
    /// 
    /// Создает `EdgeDetection` с заданным исходным состоянием
    pub fn with(init: Option<impl ToBool>) -> Self {
        let init = init.map(|v| v.to_bool());
        Self {
            init,
            prev: init,
            edge: None,
            is_high: false,
        }
    }
    ///
    /// Добавляет новое значение, обновляет состояние и возвращает зафиксированный фронт.
    pub fn add(&mut self, val: impl ToBool) -> Option<Edge> {
        let val = val.to_bool();
        self.edge = match (self.prev, val) {
            (Some(false), true) => Some(Edge::Rising),
            (Some(true), false) => Some(Edge::Falling),
            _ => None,
        };
        self.is_high = match self.edge {
            Some(Edge::Rising) => true,
            Some(Edge::Falling) => false,
            None => self.is_high,
        };
        self.prev = Some(val);
        self.edge
    }
    ///
    /// Возвращает текущий фронт если зафиксирован или `None`
    pub fn get(&self) -> Option<Edge> {
        self.edge
    }
    ///
    /// Возвращает `true`, если последним зафиксирован передний фронт.
    pub fn is_rising(&self) -> bool {
        matches!(self.edge, Some(Edge::Rising))
    }
    ///
    /// Возвращает `true`, если последним зафиксирован задний фронт.
    pub fn is_falling(&self) -> bool {
        matches!(self.edge, Some(Edge::Falling))
    }
    ///
    /// Возвращает `true` если был `Rising`, период между `Rising` и `Falling` - взведенное состояние
    pub fn is_high(&self) -> bool {
        self.is_high
    }
    ///
    /// Возвращает `true` если не было `Rising`, сброшенное состояние
    pub fn is_low(&self) -> bool {
        !self.is_high
    }
    ///
    /// Сброс в исходное состояние
    pub fn reset(&mut self) {
        self.prev = self.init;
        self.edge = None;
    }
}
///
/// ### Фронт сигнала
/// - `Rising`: Передний фронт (переход 0 -> 1).
/// - `Falling`: Задний фронт (переход 1 -> 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Rising,
    Falling,
}
impl Edge {
    ///
    /// Возвращает `true`, если последним зафиксирован передний фронт.
    pub fn is_rising(&self) -> bool {
        *self == Edge::Rising
    }
    ///
    /// Возвращает `true`, если последним зафиксирован задний фронт.
    pub fn is_falling(&self) -> bool {
        *self == Edge::Falling
    }
}
pub trait ToBool {
    fn to_bool(&self) -> bool;
}
impl ToBool for bool { fn to_bool(&self) -> bool { *self } }
impl ToBool for i8 { fn to_bool(&self) -> bool { *self != 0 } }
impl ToBool for i16 { fn to_bool(&self) -> bool { *self != 0 } }
impl ToBool for i32 { fn to_bool(&self) -> bool { *self != 0 } }
impl ToBool for i64 { fn to_bool(&self) -> bool { *self != 0 } }
impl ToBool for i128 { fn to_bool(&self) -> bool { *self != 0 } }
impl ToBool for isize { fn to_bool(&self) -> bool { *self != 0 } }
impl ToBool for u8 { fn to_bool(&self) -> bool { *self != 0 } }
impl ToBool for u16 { fn to_bool(&self) -> bool { *self != 0 } }
impl ToBool for u32 { fn to_bool(&self) -> bool { *self != 0 } }
impl ToBool for u64 { fn to_bool(&self) -> bool { *self != 0 } }
impl ToBool for u128 { fn to_bool(&self) -> bool { *self != 0 } }
impl ToBool for usize { fn to_bool(&self) -> bool { *self != 0 } }
impl ToBool for f32 { fn to_bool(&self) -> bool { self.is_finite() && *self != 0.0 } }
impl ToBool for f64 { fn to_bool(&self) -> bool { self.is_finite() && *self != 0.0 } }
///
/// Basic tests
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_undefined_initial_state() {
        let mut detector = EdgeDetector::new();
        assert_eq!(detector.add(true), None); // Неизвестно, был ли переход
        assert_eq!(detector.add(false), Some(Edge::Falling)); // Теперь четкий задний фронт
        assert_eq!(detector.add(true), Some(Edge::Rising)); // Передний фронт
    }
    #[test]
    fn test_defined_initial_state() {
        let mut detector = EdgeDetector::with(Some(false));
        assert_eq!(detector.add(true), Some(Edge::Rising)); // Сразу фиксируем фронт
        assert_eq!(detector.add(true), None); // Состояние не изменилось
    }
    #[test]
    fn test_reset_behavior() {
        let mut detector = EdgeDetector::with(Some(true));
        detector.add(false);
        assert_eq!(detector.get(), Some(Edge::Falling));
        detector.reset();
        assert_eq!(detector.get(), None);
        // После сброса начальное состояние снова Some(true)
        assert_eq!(detector.add(false), Some(Edge::Falling));
    }
}
