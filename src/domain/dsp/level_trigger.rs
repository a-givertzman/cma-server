use crate::domain::ToBool;
///
/// ### Направление изменения уровеня сигнала
/// - `Up`: Перешел в 1 (переход None / 0 -> 1).
/// - `Down`: Перешел в 0 (переход None / 1 -> 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Up,
    Down,
}
impl Level {
    ///
    /// Возвращает `true`, если последним зафиксирован переход в `1`.
    pub fn is_up(&self) -> bool {
        *self == Level::Up
    }
    ///
    /// Возвращает `true`, если последним зафиксирован переход в `0`.
    pub fn is_down(&self) -> bool {
        *self == Level::Down
    }
}
///
/// ### Детектор изменения (Level-Triggered)
/// - Накапливает значения `bool` или числовые типы.
/// - Фиксирует переходы:
/// - Переход `None -> 1` (Холодный старт) => `Some(Level::Up)`
/// - Переход `None -> 0` (Холодный старт) => `Some(Level::Down)`
/// - Переход `0 -> 1` => `Some(Level::Up)`
/// - Переход `1 -> 0` => `Some(Level::Down)`
/// - Нет перехода => `None`
#[derive(Debug, Clone, Default)]
pub struct LevelTrigger {
    init: Option<bool>,
    prev: Option<bool>,
}
impl LevelTrigger {
    ///
    /// Создает `LevelTrigger` с неопределенным исходным состоянием
    pub fn new() -> Self {
        Self::default()
    }
    /// 
    /// Создает `LevelTrigger` с заданным исходным состоянием
    pub fn with(init: impl ToBool) -> Self {
        let init = Some(init.to_bool());
        Self {
            init,
            prev: init,
        }
    }
    ///
    /// Добавляет новое значение, обновляет состояние и возвращает наличие изменения.
    /// - Переход `None -> 1` (Холодный старт) => `Some(Level::Up)`
    /// - Переход `None -> 0` (Холодный старт) => `Some(Level::Down)`
    /// - Переход `0 -> 1` => `Some(Level::Up)`
    /// - Переход `1 -> 0` => `Some(Level::Down)`
    /// - Нет перехода => `None`
    pub fn add(&mut self, val: impl ToBool) -> Option<Level> {
        let val = val.to_bool();
        let is_changed = match (self.prev, val) {
            (None, true) => Some(Level::Up),
            (None, false) => Some(Level::Down),
            (Some(false), true) => Some(Level::Up),
            (Some(true), false) => Some(Level::Down),
            (Some(false), false) => None,
            (Some(true), true) => None,
        };
        self.prev = Some(val);
        is_changed
    }
    // ///
    // /// Возвращает текущий состояние зафиксированного изменения
    // pub fn get(&self) -> Option<Level> {
    //     self.is_changed
    // }
    // ///
    // /// Возвращает `true`, если последним зафиксирован передний фронт.
    // pub fn is_rising(&self) -> bool {
    //     matches!(self.Level, Some(Level::Rising))
    // }
    // ///
    // /// Возвращает `true`, если последним зафиксирован задний фронт.
    // pub fn is_falling(&self) -> bool {
    //     matches!(self.Level, Some(Level::Falling))
    // }
    // ///
    // /// Возвращает `true` если был `Rising`, период между `Rising` и `Falling` - взведенное состояние
    // pub fn is_high(&self) -> bool {
    //     self.is_high
    // }
    // ///
    // /// Возвращает `true` если не было `Rising`, сброшенное состояние
    // pub fn is_low(&self) -> bool {
    //     !self.is_high
    // }
    ///
    /// Сброс в исходное состояние
    pub fn reset(&mut self) {
        self.prev = self.init;
    }
}
///
/// Basic tests
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cold_true() {
        let mut trigger = LevelTrigger::new();
        assert_eq!(trigger.add(true), Some(Level::Up));   // Холодный старт, всегда изменение
        assert_eq!(trigger.add(true), None);   // Без изменений
        assert_eq!(trigger.add(false), Some(Level::Down));  // Теперь задний фронт
        assert_eq!(trigger.add(false), None); // Без изменений
        assert_eq!(trigger.add(true), Some(Level::Up));   // Передний фронт
        assert_eq!(trigger.add(true), None);   // Без изменений
    }
    #[test]
    fn test_cold_false() {
        let mut trigger = LevelTrigger::new();
        assert_eq!(trigger.add(false), Some(Level::Down));  // Холодный старт, всегда изменение
        assert_eq!(trigger.add(false), None); // Состояние не изменилось
        assert_eq!(trigger.add(true), Some(Level::Up));
    }
    #[test]
    fn test_defined_initial_state() {
        let mut trigger = LevelTrigger::with(true);
        assert_eq!(trigger.add(true), None); // Состояние не изменилось
        assert_eq!(trigger.add(false), Some(Level::Down)); // Фиксируем фронт
    }
    #[test]
    fn test_reset_behavior() {
        let mut trigger = LevelTrigger::with(true);
        assert_eq!(trigger.add(false), Some(Level::Down));
        trigger.reset();
        // После сброса начальное состояние снова true
        assert_eq!(trigger.add(true), None);
    }
}
