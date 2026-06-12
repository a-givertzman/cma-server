use function_name::named;
use sal_core::error::Error;
use sal_sync::services::entity::{Name, {Point, PointTxId}};
use std::{fs, io::Write, path::{Path, PathBuf}, sync::atomic::{AtomicUsize, Ordering}};
use crate::{domain::FnOutRef, err, err_pass, services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult, functions::io::{RetainState, RetainValue}}};
///
/// ### Function | `FnRetainWrite`
/// 
/// Запись значения `Point` на диск.
/// Пишет по изменению значения, статуса или метки времени.
/// Для вычислений прозрачна, не вносит изменений в поток.
/// 
/// **Особенности работы:**
/// - `enable`: (Через `FnEnable`) При значении `false` (или 0) прерывает передачу данных (возвращает `None`).
/// - `default`: Запасной источник данных (вычисляется лениво),
///    если входа нет или он молчит (`Ok(None)`), то узел попытается вернуть и записать на диск `default`.
///
/// **Формат данных на диске**
/// ```json
/// { "value": "String value", "status": 0, "ts": "2026-06-11T09:06:45.123456789Z" }
/// { "value": 123, "status": 0", "ts": "2026-06-11T09:06:45.123456789Z" }
/// { "value": 12.3, "status": 10", "ts": "2026-06-11T09:06:45.123456789Z" }
/// { "value": false, "status": 0, "ts": "2026-06-11T09:06:45.123456789Z" }
/// ```
/// 
/// **Пример:**
/// ```yaml
/// fn Retain:            # Тут происходит запись
///     key: 'OperatingCycleId'
///     input fn Acc:
///         initial fn Retain:
///             default: const int 0
///             key: 'OperatingCycleId'
///         input: opCycleIsDone
/// ```

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
    /// - `default`: Запасной источник данных (вычисляется лениво),
    ///    если входа нет или он молчит (`Ok(None)`), то узел попытается вернуть и записать на диск `default`.
    /// - `inputs`: Входной сигнал
    #[named]
    pub fn new(parent: &Name, path: impl AsRef<Path>, default: Option<FnOutRef>, input: Option<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnRetainWrite{}", parent.join(), COUNT.fetch_add(1, Ordering::Relaxed));
        if !path.as_ref().is_file() {
            return Err(err!(id, "Rerain path is not a file: {}", path.as_ref().display()));
        }
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
    fn store(&self, state: &RetainState) -> Result<(), Error> {
        let json = serde_json::to_string(state)
            .map_err(|err| err_pass!(self.id, err, "Can't serialize JSON {:?}", state))?;
        let mut f = fs::OpenOptions::new().truncate(true).create(true).write(true).open(&self.tmp_path)
            .map_err(|err| err_pass!(self.id, err, "Can't open '{}'", self.tmp_path.display()))?;
        f.write_all(json.as_bytes())
            .map_err(|err| err_pass!(self.id, err, "Can't Write '{}'", self.tmp_path.display()))?;
        f.sync_data()
            .map_err(|err| err_pass!(self.id, err, "Can't Sync {}", self.tmp_path.display()))?;
        fs::rename(&self.tmp_path, &self.path)
            .map_err(|err| err_pass!(self.id, err, "Can't Rename '{}' -> '{}'", self.tmp_path.display(), self.path.display()))?;
        log::trace!("{}.store | Can't retain state to '{}'", self.id, self.path.display());
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
        let input = self.input.as_ref().map(|f| f.borrow_mut().out());
        if let Some(input) = input {
            if let Some(input) = flow.map(input)? {
                if let Err(err) = self.store(&RetainState::from(&input)) {
                    log::error!("{}.out | Can't store value '{}': {:?}", self.id, input.name(), err);
                }
                return flow.wrap(input);
            }
        }
        if let Some(default) = self.default.as_ref() {
            if let Some(default) = flow.map(default.borrow_mut().out())? {
                if let Err(err) = self.store(&RetainState::from(&default)) {
                    log::error!("{}.out | Can't store value '{}': {:?}", self.id, default.name(), err);
                }
                return flow.wrap(default);
            }
        }
        Ok(None)
    }
    //
    fn reset(&mut self) {
        if let Some(default) = &self.default {
            default.borrow_mut().reset();
        }
        if let Some(input) = &self.input {
            input.borrow_mut().reset();
        }
        self.cache = None;
    }
}
///
/// Global static counter of FnRetainWrite instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
