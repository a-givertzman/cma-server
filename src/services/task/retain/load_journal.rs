use std::{collections::HashMap, fs::File, io::{BufRead, BufReader, Read}, path::Path, sync::Arc};
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::services::{entity::{Cot, Point, PointHlr}, types::Bool};
use crate::{domain::FxSccHashMap, err, err_pass, services::task::retain::{RetainState, RetainValue}};
use super::{EvalResult, Eval, RetainCtx};

///
/// ### Локальный флаг состояния чтения из файла
enum IoState {
    Continue((String, RetainState)),
    Done,
}
///
/// ### Загрузка состояния журнала в оперативный кэш
pub struct LoadJournal<Child> {
    child: Child,
    dbg: Dbg,
}
//
impl<Child> LoadJournal<Child> {
    ///
    /// Returns `LoadJournal` new instance
    pub fn new(parent: impl Into<String>, child: Child) -> Self {
        let dbg = Dbg::new(parent, crate::domain::me::<Self>());
        Self {
            child,
            dbg,
        }
    }
    ///
    /// ### Создает `Point` из `RetainState`
    fn point(state: &RetainState, txid: usize, name: impl Into<String>) -> Point {
        match &state.value {
            RetainValue::Bool(v) => Point::Bool(PointHlr::new(txid, name, Bool(*v), state.status, Cot::Inf, state.ts)),
            RetainValue::Int(v) => Point::Int(PointHlr::new(txid, name, *v, state.status, Cot::Inf, state.ts)),
            RetainValue::Real(v) => Point::Real(PointHlr::new(txid, name, *v, state.status, Cot::Inf, state.ts)),
            RetainValue::Double(v) => Point::Double(PointHlr::new(txid, name, *v, state.status, Cot::Inf, state.ts)),
            RetainValue::String(v) => Point::String(PointHlr::new(txid, name, v.clone(), state.status, Cot::Inf, state.ts)),
            RetainValue::Bytes(v) => Point::Bytes(PointHlr::new(txid, name, v.clone(), state.status, Cot::Inf, state.ts)),
        }
    }
    ///
    /// ### Парсит одну запись из байтов `postcard` в `(String, RetainState)`
    #[named]
    fn decode_entry(dbg: &Dbg, reader: &mut BufReader<File>, len_buf: &mut [u8; 4], key_buf: &mut Vec<u8>, state_buf: &mut Vec<u8>) -> Result<IoState, Error> {
        if reader.read_exact(len_buf).is_err() { return Ok(IoState::Done); }
        let len = u32::from_le_bytes(*len_buf);
        if len > 1024 {
            return Err(err!(dbg, "Размер ключа retain-записи: {} - превышает 1KB, файл кэша поврежден", len));
        }
        key_buf.resize(len as usize, 0u8);
        reader.read_exact(key_buf).map_err(|err| err_pass!(dbg, err))?;
        let key = std::str::from_utf8(key_buf).map_err(|err| err_pass!(dbg, err))?;
        reader.read_exact(len_buf).map_err(|err| err_pass!(dbg, err))?;
        let len = u32::from_le_bytes(*len_buf);
        if len > 10 * 1024 * 1024 {
            return Err(err!(dbg, "Размер значения retain-записи: {} - превышает 100MB, файл кэша поврежден", len));
        }
        state_buf.resize(len as usize, 0u8);
        reader.read_exact(state_buf).map_err(|err| err_pass!(dbg, err))?;
        let state = postcard::from_bytes::<RetainState>(state_buf).map_err(|err| err_pass!(dbg, err))?;
        Ok(IoState::Continue((key.to_owned(), state)))
    }
    ///
    /// ### Чтение с диска и парсинг `RetainState`
    /// 
    /// - Максимальный размер ключа - 1 KB
    /// - Максимальный размер `RetainState` - 10 MB
    /// 
    /// Устойчив к повреждению хвоста файла. При обнаружении бинарного мусора
    /// или неожиданного конца файла чтение останавливается, а корректно загруженные данные сохраняются.
    fn load(&self, path: &Path, txid: usize, cache: &Arc<FxSccHashMap<String, Point>>) -> Result<(), Error> {
        let dat_path = path.with_extension("dat");
        // let file = File::open(&dat_path).map_err(|err| err_pass!(self.dbg, err, "Can't open file '{}'", dat_path.display()))?;
        match File::open(&dat_path) {
            Ok(file) => {
                let mut reader = BufReader::new(file);
                let mut len_buf = [0u8; 4];
                let mut key_buf = Vec::with_capacity(1024);
                let mut state_buf = Vec::with_capacity(4096);
                loop {
                    match Self::decode_entry(&self.dbg, &mut reader, &mut len_buf, &mut key_buf, &mut state_buf) {
                        Ok(IoState::Done) => break,
                        Ok(IoState::Continue((name, state))) => {
                            let val = Self::point(&state, txid, &name);
                            _ = cache.upsert_sync(name, val);
                        }
                        Err(err) => {
                            log::error!("{}.load | Retain файл журнала оборван или поврежден '{}'.\n\tОшибка: {:?}.\n\tТолько часть данных загружено: {:#?}.",
                                self.dbg, dat_path.display(), err, cache);
                            break;
                        }
                    }
                }
                return Ok(());
            }
            Err(err) => if err.kind() != std::io::ErrorKind::NotFound {
                log::warn!("{}.load | Can't read cache '{}': {:?}", self.dbg, dat_path.display(), err);
            }
        }
        let json_path = path.with_extension("json");
        match File::open(&json_path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let mut lines = reader.lines();
                while let Some(line) = lines.next() {
                    match line {
                        Ok(line) => {
                            match serde_json::from_str::<HashMap<String, RetainState>>(&line) {
                                Ok(parsed) => {
                                    if let Some((key, state)) = parsed.into_iter().next() {
                                        let val = Self::point(&state, txid, &key);
                                        if let Err(err) = cache.insert_sync(key, val) {
                                            log::warn!("{}.load | Can't extend cache: {:?}", self.dbg, err);
                                        }
                                    }
                                }
                                Err(err) => log::warn!("{}.load | Can't parse entry in {}, error: {:?}", self.dbg, json_path.display(), err),
                            }
                        }
                        Err(err) => log::warn!("{}.load | Can't read entry from {}, error: {:?}", self.dbg, json_path.display(), err),
                    }
                }
            }
            Err(err) => if err.kind() != std::io::ErrorKind::NotFound {
                log::warn!("{}.load | Can't read cache '{}': {:?}", self.dbg, json_path.display(), err);
            }
        }
        Ok(())
    }
}
//
impl<Child> Eval<RetainCtx, EvalResult> for LoadJournal<Child>
where
    Child: Eval<RetainCtx, EvalResult>, {
    #[named]
    fn eval(&self, ctx: RetainCtx) -> EvalResult {
        self.load(&ctx.path, ctx.txid, &ctx.cache).map_err(|err| err_pass!(self.dbg, err))?;
        self.child.eval(ctx).map_err(|err: Error| err_pass!(self.dbg, err))
    }
}
///
/// Basic Tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use sal_sync::services::entity::Status;
use tempfile::NamedTempFile;
    use crate::services::task::retain::RetainValue;
    #[test]
    fn test_load_journal_torn_write_recovery() {
        let mut file = NamedTempFile::new().unwrap();
        let state = RetainState { value: RetainValue::Bool(true), status: Status::Ok, ts: chrono::Utc::now() };
        let key = "Motor_1_Start";
        let key_len = (key.len() as u32).to_le_bytes();
        file.write_all(&key_len).unwrap();
        file.write_all(key.as_bytes()).unwrap();
        let bytes = postcard::to_allocvec(&state).unwrap();
        let val_len = (bytes.len() as u32).to_le_bytes();
        file.write_all(&val_len).unwrap();
        file.write_all(&bytes).unwrap();
        let key2 = "Motor_2_Start";
        let key_len2 = (key2.len() as u32).to_le_bytes();
        file.write_all(&key_len2).unwrap();
        file.write_all(key2.as_bytes()).unwrap();
        let val_len2 = (bytes.len() as u32).to_le_bytes();
        file.write_all(&val_len2).unwrap();
        file.write_all(&bytes[..bytes.len() / 2]).unwrap();
        file.flush().unwrap();
        let dbg = Dbg::new("Test", "decode_entry");
        let mut file = File::open(file.path()).unwrap();
        let mut reader = BufReader::new(file);
        let mut len_buf = [0u8; 4];
        let mut key_buf = Vec::new();
        let mut state_buf = Vec::new();
        let res1 = LoadJournal::<()>::decode_entry(&dbg, &mut reader, &mut len_buf, &mut key_buf, &mut state_buf);
        assert!(matches!(res1, Ok(IoState::Continue((k, _))) if k == "Motor_1_Start"));
        let res2 = LoadJournal::<()>::decode_entry(&dbg, &mut reader, &mut len_buf, &mut key_buf, &mut state_buf);
        assert!(res2.is_err(), "Должен поймать ошибку оборванной записи хвоста файла");
    }
}
