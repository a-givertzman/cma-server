use function_name::named;
use sal_core::error::Error;
use sal_sync::services::
    entity::{Name, {Point, PointTxId}}
;
use std::{env, fs, io::Write, path::{Path, PathBuf}, sync::atomic::{AtomicUsize, Ordering}};
use crate::{
    domain::FnOutRef, pass_err, services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult, functions::io::RetainState}
};
///
/// ### Function | FnRetainWrite
/// 
/// Набор инструментов для записи и чтения значений на диске
/// 
/// - **Формат данных на диске**
/// ```json
/// { "value": "String value", "status": 0, "ts": "2026-06-11T09:06:45.123456789Z" }
/// { "value": 123, "status": 0", "ts": "2026-06-11T09:06:45.123456789Z" }
/// { "value": 12.3, "status": 10", "ts": "2026-06-11T09:06:45.123456789Z" }
/// { "value": false, "status": 0, "ts": "2026-06-11T09:06:45.123456789Z" }
/// ```
/// - **FnRetainWrite**
///     - Запись на диск по изменению значения, статуса или метки времени
///     - Для вычислений прозрачна, не вносит изменений в поток
/// - **FnRetainRead**
///     - Чтение значений с диска (по умолчанию читает с диска только в первый цикл, дальше возвращает закэшированное значение)
///     - `default` на случай когда значений еще не записано
///     - `every-cycle` - значение будет читаться на каждом цикле вычислений (учитывай нагрузку на диск)
#[derive(Debug)]
pub struct FnRetainWrite {
    txid: usize,
    kind: FnKind,
    default: Option<FnOutRef>,
    input: Option<FnOutRef>,
    cache: Option<Point>,
    tmp_path: PathBuf,
    path: PathBuf,
    id: String,
}
//
impl FnRetainWrite {
    ///
    /// ### Returns `FnRetainWrite` new instance
    /// - `parent`: Идентификатор родительского узла
    /// - `path` Путь к retain файлу для значения (`assets/retain/App/RecorderTask/retain_value.json`)
    /// - `enable` - boolean (numeric) input enables the readinf/storing and pass through if true (> 0)
    /// - `every-cycle` - if true read will done in every computing cycle, else read will done only once
    /// - `key` - the key to store Point with (full path: ./assets/retain/App/TaskName/key.json)
    /// - `input` - incoming Point's
    #[named]
    pub fn new(parent: &Name, path: impl AsRef<Path>, default: Option<FnOutRef>, input: Option<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnRetainWrite{}", parent.join(), COUNT.fetch_add(1, Ordering::Relaxed));
        let tmp_path = path.as_ref().with_extension("json.tmp");
        Ok(Self {
            txid: PointTxId::from_str(&id),
            kind: FnKind::Fn,
            default,
            input,
            cache: None,
            tmp_path,
            path: path.as_ref().to_path_buf(),
            id,
        })
    }
    ///
    /// ### Физическая запись состояния на диск
    /// 
    /// `RetainState` пишется через атомарную подмену файлов
    #[named]
    fn store(&self, state: &RetainState) -> Result<(), String> {
        let json = serde_json::to_string(state)
            .map_err(|err| pass_err!(self.id, err, "Can't serialize JSON {:?}", state))?;
        let mut f = fs::OpenOptions::new().truncate(true).create(true).write(true).open(&self.tmp_path)
            .map_err(|err| pass_err!(self.id, err, "Can't open '{}'", self.tmp_path.display()))?;
        f.write_all(json.as_bytes())
            .map_err(|err| pass_err!(self.id, err, "Can't Write '{}'", self.tmp_path.display()))?;
        f.sync_data()
            .map_err(|err| pass_err!(self.id, err, "Can't Sync {}", self.tmp_path.display()))?;
        fs::rename(&self.tmp_path, &self.path)
            .map_err(|err| pass_err!(self.id, err, "Can't Rename '{}' -> '{}'", self.tmp_path.display(), self.path.display()))?;
        log::trace!("{}.store | Retained '{}' state synchronized: '{}'", self.id, self.key, self.path.display());
        Ok(())
    }
}
//
impl FnOut for FnRetainWrite {
    //
    fn id(&self) -> String {
        self.id.clone()
    }
    //
    fn kind(&self) -> FnKind {
        self.kind
    }
    //
    fn inputs(&self) -> Vec<String> {
        let mut inputs = vec![];
        if let Some(default) = &self.default {
            inputs.append(&mut default.borrow().inputs());
        }
        if let Some(input) = &self.input {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let enable = match &self.enable {
            Some(enable) => {
                let enable = enable.borrow_mut().out();
                log::trace!("{}.out | enable: {:?}", self.id, enable);
                match enable {
                    FnResult::Ok(enable) => enable.to_bool().as_bool().value.0,
                    FnResult::None => return FnResult::None,
                    FnResult::Err(err) => return FnResult::Err(err),
                }
            }
            None => true,
        };
        log::trace!("{}.out | enable: {:?}", self.id, enable);
        if enable {
            match &self.input {
                Some(input) => {
                    let input = input.borrow_mut().out();
                    log::trace!("{}.out | input: {:?}", self.id, input);
                    match input {
                        FnResult::Ok(input) => {
                            if let Err(err) = self.store(&input) {
                                log::error!("{}.out | Error: '{:?}'", self.id, err);
                            };
                            FnResult::Ok(input)
                        }
                        FnResult::None => FnResult::None,
                        FnResult::Err(err) => FnResult::Err(err),
                    }
                }
                None => {
                    let default = match &self.default {
                        Some(default) => {
                            let default = default.borrow_mut().out();
                            log::trace!("{}.out | default: {:?}", self.id, default);
                            match default {
                                FnResult::Ok(default) => default,
                                FnResult::None => return FnResult::None,
                                FnResult::Err(err) => return FnResult::Err(err),
                            }
                        }
                        None => panic!("{}.out | The [default] input is not specified", self.id),
                    };
                    if self.every_cycle {
                        let point = match self.load(default.type_()) {
                            Some(point) => point,
                            None => default,
                        };
                        log::trace!("{}.out | every cycle: {} \t loaded '{}': \n\t{:?}", self.id, self.every_cycle, self.key, point);
                        FnResult::Ok(point)
                    } else {
                        let point = match &self.cache {
                            Some(point) => point.clone(),
                            None => match self.load(default.type_()) {
                                Some(point) => {
                                    point
                                }
                                None => default,
                            }
                        };
                        self.cache = Some(point.clone());
                        log::trace!("{}.out | every cycle: {} \t loaded '{}': \n\t{:?}", self.id, self.every_cycle, self.key, point);
                        FnResult::Ok(point)
                    }
                }
            }
        } else {
            FnResult::None
        }
    }
    //
    fn reset(&mut self) {
        if let Some(default) = &self.default {
            default.borrow_mut().reset();
        }
        if let Some(input) = &self.input {
            input.borrow_mut().reset();
        }
    }
}
///
/// Global static counter of FnRetainWrite instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
