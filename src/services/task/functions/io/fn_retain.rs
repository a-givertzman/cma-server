use function_name::named;
use sal_core::error::Error;
use sal_sync::services::{Services, entity::Name , task::functions::FnConfig};
use std::{cell::RefCell, rc::Rc, sync::Arc};
use crate::{domain::{FnOutRef}, err, err_pass, services::task::{FnBuilder, FnEnable, TaskNodes, functions::{FnRetainRead, FnRetainWrite}}};

///
/// ### Builder | `FnRetain`
/// 
/// Билдер для создания узлов `FnRetainRead` или `FnRetainWrite`.
/// 
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
            .map_err(|err| err_pass!(self_id, err, "Can't get 'enable'"))?;
        let default = FnBuilder::get_input_config(parent, "default", conf, nodes, services)
            .map_err(|err| err_pass!(self_id, err, "Can't get 'default'"))?;
        let input = FnBuilder::get_input_config(parent, "input", conf, nodes, services)
            .map_err(|err| err_pass!(self_id, err, "Can't get 'input'"))?;
        let every_cycle = conf.param("every-cycle").map_or(Ok(false), |param| {
            param.as_param().conf.as_bool().ok_or_else(|| err!(self_id, "'every-cycle' - wrong config"))
        })?;
        let Some(key) = conf.param("key").map(|v| v.as_param()) else {
            return Err(err!(self_id, "Parameter 'key' - missed in '{}'", conf.name));
        };
        let key = key.conf.as_str()
            .ok_or_else(|| err!(self_id, "Parameter 'key' must be a string in '{}'", conf.name))?;
        Ok(match input {
            None => {
                let read = FnRetainRead::new(parent, nodes.retain(), key, every_cycle, default).map_err(|err| err_pass!(self_id, err))?;
                match enable {
                    Some(en) => Rc::new(RefCell::new(FnEnable::new(read, nodes.enable_mode(), en))),
                    None => Rc::new(RefCell::new(read)),
                }
            }
            Some(input) => { 
                let write = FnRetainWrite::new(parent, nodes.retain().link(), key, default, input).map_err(|err| err_pass!(self_id, err))?;
                match enable {
                    Some(en) => Rc::new(RefCell::new(FnEnable::new(write, nodes.enable_mode(), en))),
                    None => Rc::new(RefCell::new(write)),
                }
            }
        })
    }
}
