use sal_sync::services::entity::{Point, Status};
use serde::{Deserialize, Serialize};

///
/// ### Key for retation value
#[derive(Clone)]
pub(crate) struct RetainEvent {
    pub key: String,
    pub p: Point,
}
//
impl RetainEvent {
    ///
    /// ### Returns `RetainEvent` new instance
    pub fn new(key: String, p: Point) -> Self {
        Self { key, p }
    }
}
///
/// ### `RetainValue` | Storage wrapper for `Point` retain
/// Легковесная обертка для сериализации и десериализации типов данных `Point` на диск.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(super) enum RetainValue {
    Bool(bool),
    Int(i64),
    Real(f32),
    Double(f64),
    String(String),
    Bytes(Vec<u8>),
}
impl From<sal_sync::services::entity::Point> for RetainValue {
    fn from(p: sal_sync::services::entity::Point) -> Self {
        match p {
            sal_sync::services::entity::Point::Bool(p) => Self::Bool(p.value.0),
            sal_sync::services::entity::Point::Int(p) => Self::Int(p.value),
            sal_sync::services::entity::Point::Real(p) => Self::Real(p.value),
            sal_sync::services::entity::Point::Double(p) => Self::Double(p.value),
            sal_sync::services::entity::Point::String(p) => Self::String(p.value),
            sal_sync::services::entity::Point::Bytes(p) => Self::Bytes(p.value),
        }
    }
}
impl From<&sal_sync::services::entity::Point> for RetainValue {
    fn from(p: &sal_sync::services::entity::Point) -> Self {
        match p {
            sal_sync::services::entity::Point::Bool(p) => Self::Bool(p.value.0),
            sal_sync::services::entity::Point::Int(p) => Self::Int(p.value),
            sal_sync::services::entity::Point::Real(p) => Self::Real(p.value),
            sal_sync::services::entity::Point::Double(p) => Self::Double(p.value),
            sal_sync::services::entity::Point::String(p) => Self::String(p.value.clone()),
            sal_sync::services::entity::Point::Bytes(p) => Self::Bytes(p.value.clone()),
        }
    }
}
///
/// ### Состояние `Point` для хранения на диске
/// Инкапсулирует полное физическое состояние точки данных на момент записи.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub(super) struct RetainState {
    pub value: RetainValue,
    pub status: Status,
    pub ts: chrono::DateTime<chrono::Utc>,
}
impl From<&sal_sync::services::entity::Point> for RetainState {
    fn from(p: &sal_sync::services::entity::Point) -> Self {
        Self { value: RetainValue::from(p), status: p.status(), ts: p.timestamp() }
    }
}
impl From<sal_sync::services::entity::Point> for RetainState {
    fn from(p: sal_sync::services::entity::Point) -> Self {
        Self { status: p.status(), ts: p.timestamp(), value: RetainValue::from(p) }
    }
}
///
/// Basic Tests
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use sal_sync::services::entity::Status;
    use std::path::PathBuf;
    #[test]
    fn test_retain_state_contract() {
        // Подготавливаем эталонный слепок памяти
        let state = RetainState {
            value: RetainValue::Double(12.345),
            status: Status::Ok,
            ts: chrono::Utc.with_ymd_and_hms(2026, 6, 11, 9, 6, 45).unwrap(),
        };
        let json = serde_json::to_string(&state).unwrap();
        // Проверяем, что сериализатор честно развернул enum в объектный формат
        assert!(json.contains(r#"{"Double":12.345}"#));
        // Вскрытие капота: проверяем десериализацию
        let deserialized: RetainState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deserialized);
    }
    #[test]
    fn test_retain_value_polymorphism() {
        // Проверяем способность контракта переваривать все поддерживаемые типы 
        let values = vec![
            RetainValue::Bool(true),
            RetainValue::Int(-42),
            RetainValue::Real(3.14),
            RetainValue::String("Motor_Start".to_string()),
        ];
        for original in values {
            let json = serde_json::to_string(&original).unwrap();
            let restored: RetainValue = serde_json::from_str(&json).unwrap();
            assert_eq!(original, restored);
        }
    }
    #[test]
    fn test_retain_path_building() {
        // Тест цементирует механику склеивания путей через trim_start_matches
        let retain_path = PathBuf::from("assets/retain");
        let parent_str = "/AppName/TaskName";
        let key = "retain_key";
        let dir = retain_path.join(parent_str.trim_start_matches('/'));
        let path = dir.join(key).with_extension("json");
        let expected = PathBuf::from("assets/retain/AppName/TaskName/retain_key.json");
        assert_eq!(path, expected);
    }
}
