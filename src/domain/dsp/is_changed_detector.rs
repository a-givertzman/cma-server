///
/// Трейт, который умеет клонировать себя в Box и сравнивать себя с dyn Any
pub trait DynamicEq: Any {
    fn eq_any(&self, other: &dyn Any) -> bool;
    fn clone_box(&self) -> Box<dyn DynamicEq>;
}
///
/// Реализуем его для всех типов, которые уже умеют сравниваться и клонироваться
impl<T: PartialEq + Clone + Any> DynamicEq for T {
    fn eq_any(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<T>().map_or(false, |v| v == self)
    }
    fn clone_box(&self) -> Box<dyn DynamicEq> {
        Box::new(self.clone())
    }
}
///
/// ### Детектор изменений
/// - Накапливает значения любых типов через метод `add()`
/// - Фиксирует изменение значения или его типа
pub struct IsChangedDetector {
    init: Option<Box<dyn DynamicEq>>,
    prev: Option<Box<dyn DynamicEq>>,
    is_changed: bool
}

impl IsChangedDetector {
    ///
    /// Создает `IsChangedDetector` с неопределенным исходным состоянием
    pub fn new() -> Self {
        Self {
            init: None,
            prev: None,
            is_changed: false,
        }
    }
    ///
    /// Создает `IsChangedDetector` с заданным исходным состоянием
    pub fn with<T: PartialEq + Clone + Any>(init: Option<T>) -> Self {
        match init {
            Some(val) => {
                Self {
                    init: Some(val.clone_box()),
                    prev: Some(Box::new(val) as Box<dyn DynamicEq>),
                    is_changed: false,
                }
            }
            None => Self::new(),
        }
    }
    ///
    /// Геттер для чтения статуса измененности извне
    pub fn is_changed(&self) -> bool {
        self.is_changed
    }
    ///
    /// Добавляет новое значение, возвращает `true`, если оно отличается от предыдущего
    pub fn add<T: PartialEq + Clone + Any>(&mut self, v: &T) -> bool {
        let is_changed = match &self.prev {
            Some(prev) => !prev.eq_any(v),
            None => true, // Если prev не было, это первое значение — оно изменилось
        };
        if is_changed {
            self.prev = Some(v.clone_box());
        }
        self.is_changed = is_changed;
        is_changed
    }
    ///
    /// Сброс в исходное состояние
    pub fn reset(&mut self) {
        self.prev = self.init.as_ref().map(|v| v.clone_box());
        self.is_changed = false;
    }
}
//
impl Debug for IsChangedDetector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IsChangedDetector")
            // .field("init", &self.init)
            // .field("prev", &self.prev)
            .field("is_changed", &self.is_changed)
            .finish()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    ///
    #[test]
    fn test_initial_state_with_value() {
        // Создаем детектор с начальным значением 42
        let detector = IsChangedDetector::with(Some(42));
        // Сразу после создания статус измененности должен быть false
        assert!(!detector.is_changed());
    }
    ///
    #[test]
    fn test_initial_state_empty() {
        // Создаем пустой детектор
        let detector = IsChangedDetector::new();
        assert!(!detector.is_changed());
    }
    ///
    #[test]
    fn test_add_same_value_does_not_trigger() {
        let mut detector = IsChangedDetector::with(Some(100));
        // Добавляем то же самое число
        let res = detector.add(&100);
        assert!(!res, "Метод add() должен вернуть false для одинаковых значений");
        assert!(!detector.is_changed(), "Флаг is_changed должен остаться false");
    }
    ///
    #[test]
    fn test_add_different_value_triggers() {
        let mut detector = IsChangedDetector::with(Some("hello".to_string()));
        // Изменяем строку
        let res1 = detector.add(&"world".to_string());
        assert!(res1, "Первое изменение должно вернуть true");
        assert!(detector.is_changed(), "Флаг должен переключиться в true");
        // Повторяем новое значение
        let res2 = detector.add(&"world".to_string());
        assert!(!res2, "Повтор нового значения должен вернуть false");
        assert!(!detector.is_changed(), "Флаг должен сброситься в false, так как относительно предыдущего шага изменений нет");
    }
    ///
    #[test]
    fn test_dynamic_type_swapping() {
        // Наш детектор универсален, проверим смену типов данных на лету
        let mut detector = IsChangedDetector::with(Some(10i64));
        // Меняем i64 на String
        assert!(detector.add(&"строка".to_string()), "Смена типа должна зафиксировать изменение");
        assert!(detector.is_changed());
        // Меняем String на массив байт Vec<u8>
        assert!(detector.add(&vec![1, 2, 3]), "Смена типа на bytes должна зафиксировать изменение");
        assert!(detector.is_changed());
        // Повторяем тот же массив байт
        assert!(!detector.add(&vec![1, 2, 3]), "Повтор массива байт не должен считаться изменением");
        assert!(!detector.is_changed());
    }
    ///
    #[test]
    fn test_reset_behavior() {
        // Инициализируем числом 5
        let mut detector = IsChangedDetector::with(Some(5));
        // Изменяем стейт несколько раз
        detector.add(&10);
        detector.add(&20);
        assert!(detector.is_changed());
        // Сбрасываем стейт к исходному (к 5)
        detector.reset();
        assert!(!detector.is_changed(), "После reset флаг измененности должен быть false");
        // Проверяем, что текущим значением (prev) снова стала пятерка.
        // Если мы добавим 5, изменений быть не должно.
        assert!(!detector.add(&5), "После reset текущее состояние должно быть равно исходному (5)");
        // А если добавим 20, оно должно зафиксировать изменение относительно 5.
        assert!(detector.add(&20), "После reset изменение относительно начального стейта должно работать");
    }
    ///
    #[test]
    fn test_empty_detector_behavior() {
        let mut detector = IsChangedDetector::new();
        // Для пустого детектора первое добавление — это всегда изменение
        assert!(detector.add(&true));
        assert!(detector.is_changed());
        // Повтор
        assert!(!detector.add(&true));
        assert!(!detector.is_changed());
    }
}
