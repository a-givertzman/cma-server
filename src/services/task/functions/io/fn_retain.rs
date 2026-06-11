use function_name::named;
use sal_core::error::Error;
use sal_sync::services::{Services, entity::{Name, Status, }, task::functions::FnConfig};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc, sync::Arc};
use crate::{domain::{FnOutRef}, new_err, pass_err, services::task::{FnBuilder, FnEnable, TaskNodes, functions::{FnRetainRead, FnRetainWrite}}};
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
///
/// ### Состояние `Point` для хранения на диске
/// Инкапсулирует полное физическое состояние точки данных на момент записи.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub(super) struct RetainState {
    pub value: RetainValue,
    pub status: Status,
    pub ts: chrono::DateTime<chrono::Utc>,
}
///
/// ### Builder | `FnRetain`
/// 
/// Билдер для создания узлов `FnRetainRead` или `FnRetainWrite`.
/// 
/// - **Формат данных на диске**
/// ```json
/// { "value": {"Bool": false}, "status": 0, "ts": "2026-06-11T09:06:45.123456789Z" }
/// { "value": {"Int": 123}, "status": 0, "ts": "2026-06-11T09:06:45.123456789Z" }
/// { "value": {"Real": 12.3}, "status": 0, "ts": "2026-06-11T09:06:45.123456789Z" }
/// { "value": {"Double": 12.3}, "status": 0, "ts": "2026-06-11T09:06:45.123456789Z" }
/// { "value": {"String": "String value"}, "status": 0, "ts": "2026-06-11T09:06:45.123456789Z" }
/// ```
/// - **`FnRetainRead`** (Если `input` отсутствует)
///     - Чтение значений с диска (по умолчанию читает с диска только в первый цикл, дальше возвращает закэшированное значение).
///     - `default` на случай, когда значений еще не записано.
///     - `every-cycle` - значение будет читаться на каждом цикле вычислений (учитывай нагрузку на диск)
/// - **`FnRetainWrite`** (Если `input` задан)
///     - Запись на диск при изменении значения, статуса или метки времени.
///     - Для графа прозрачна, пропускает `FnFlow` сквозь себя без модификаций.
#[derive(Debug)]
pub struct FnRetain {}
//
impl FnRetain {
    ///
    /// ### Returns `FnRetainRead`  or `FnRetainWrite` new instance
    /// - `parent`: Идентификатор родительского узла
    /// - `conf`: Конфигурация узла
    #[named]
    pub fn new(parent: &Name, conf: &FnConfig, nodes: &mut TaskNodes, services: &Arc<Services>) -> Result<FnOutRef, Error> {
        let self_id = format!("{parent}/FnRetain");
        let enable = FnBuilder::get_input_config(parent, "enable", conf, nodes, services)
            .map_err(|err| pass_err!(self_id, err, "Can't get 'enable'"))?;
        let default = FnBuilder::get_input_config(parent, "default", conf, nodes, services)
            .map_err(|err| pass_err!(self_id, err, "Can't get 'default'"))?;
        let input = FnBuilder::get_input_config(parent, "input", conf, nodes, services)
            .map_err(|err| pass_err!(self_id, err, "Can't get 'input'"))?;
        let every_cycle = conf.param("every-cycle").map_or(Ok(false), |param| {
            param.as_param().conf.as_bool().ok_or_else(|| new_err!(self_id, "'every-cycle' - wrong config"))
        })?;
        let Some(key) = conf.param("key").map(|v| v.as_param()) else {
            return Err(new_err!(self_id, "Parameter 'key' - missed in '{}'", conf.name));
        };
        let key = key.conf.as_str()
            .ok_or_else(|| new_err!(self_id, "Parameter 'key' must be a string in '{}'", conf.name))?;
        let Some(retain_path) = services.retain().path else {
            return Err(new_err!(self_id, "Retain: path - missed in Application config"));
        };
        let cw_dir = std::env::current_dir().map_err(|err| pass_err!(self_id, err))?;
        let dir = cw_dir.join(retain_path).join(parent.join().trim_start_matches("/"));
        std::fs::create_dir_all(&dir).map_err(|err| pass_err!(self_id, err, "Error creating dir: '{}'", dir.display()))?;
        let path = dir.join(key).with_extension("json");
        Ok(if input.is_none() {
            let read = FnRetainRead::new(parent, path, every_cycle, default).map_err(|err| pass_err!(self_id, err))?;
            match enable {
                Some(en) => Rc::new(RefCell::new(FnEnable::new(read, nodes.enable_mode(), en))),
                None => Rc::new(RefCell::new(read)),
            }
        } else { 
            let write = FnRetainWrite::new(parent, path, default, input).map_err(|err| pass_err!(self_id, err))?;
            match enable {
                Some(en) => Rc::new(RefCell::new(FnEnable::new(write, nodes.enable_mode(), en))),
                None => Rc::new(RefCell::new(write)),
            }
        })
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
