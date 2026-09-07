use serde::Deserialize;

/// Tables used for storing diagnostic risults into database
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Tables {
    /// Таблица | Справочник оборудования.
    pub equipment: String,
    /// Таблица | Тренды вибрации.
    pub trends: String,
    /// Таблица | Состояния оборудования на основании вибрации.
    pub faults: String,
}
