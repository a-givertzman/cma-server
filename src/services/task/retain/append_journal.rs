use std::{cell::Cell, collections::VecDeque, fs::File, io::{BufWriter, Write}};
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use serde::{Serialize, Serializer, ser::SerializeMap};
use crate::{err_pass, services::task::{RetainEvent, RetainMode, retain::RetainState}};
use super::{Eval, RetainCtx};

///
/// ### Пишет один пакет с кадрированием длины в конец файла
pub struct AppendJournal<Child> {
    txid: usize,
    mode: RetainMode,
    buffer: Cell<VecDeque<RetainEvent>>,
    child: Child,
    dbg: Dbg,
}
//
impl<Child> AppendJournal<Child> {
    /// Returns `AppendJournal` new instance
    pub fn new(parent: impl Into<String>, txid: usize, mode: RetainMode, child: Child) -> Self {
        let dbg = Dbg::new(parent, crate::domain::me::<Self>());
        Self {
            txid,
            mode,
            buffer: Cell::new(VecDeque::new()),
            child,
            dbg,
        }
    }
}
//
impl<Child> Eval<(Option<RetainEvent>, RetainCtx), RetainCtx> for AppendJournal<Child>
where
    Child: Eval<RetainCtx, RetainCtx>, {
    #[named]
    fn eval(&self, (event, mut ctx): (Option<RetainEvent>, RetainCtx)) -> RetainCtx {
        let mut buf = if ctx.compacted {
            ctx.compacted = false;
            VecDeque::new()
        } else {
            self.buffer.take()
        };
        if let Some(event) = event {
            ctx.cache.insert_sync(event.key.clone(), event.p.clone());
            buf.push_back(event);
        }
        if let Some(writer) = &mut ctx.writer {
            let mut not_retained = Vec::new();
            for (i, event) in buf.iter().enumerate() {
                let state = RetainState::from(&event.p);
                match append(&self.dbg, writer, &self.mode, &event.key, &state) {
                    Ok(bytes) => {
                        ctx.file_size_bytes = ctx.file_size_bytes + bytes as u64;
                    }
                    Err(err) => {
                        log::warn!("{}.run | Can't store retain '{}' to '{}', error: {:?}", self.dbg, event.key, ctx.path.display(), err);
                        not_retained.push(i);
                    }
                }
            }
            if not_retained.is_empty() {
                buf.clear();
            } else {
                let mut current_idx = 0;
                buf.retain(|_| {
                    let keep = not_retained.contains(&current_idx);
                    current_idx += 1;
                    keep
                });
            }
        }
        self.buffer.set(buf);
        let mut ctx = self.child.eval(ctx);
        ctx.error = ctx.error.map(|err| err_pass!(self.dbg, err));
        ctx
    }
}
///
/// ### Сериализует и пишет один пакет в буфер, возвращая количество записанных байт
#[named]
pub fn append<T: Serialize>(
    dbg: &Dbg,
    writer: &mut BufWriter<File>, 
    mode: &RetainMode,
    key: &String,
    state: &T,
) -> Result<usize, Error> {
    match mode {
        RetainMode::Debug => {
            let mut writer = CountingWriter::new(writer);
            let mut serializer = serde_json::Serializer::new(&mut writer);
            let mut map = serializer.serialize_map(Some(1)).map_err(|err| err_pass!(dbg, err))?;
            map.serialize_entry(key, state).map_err(|err| err_pass!(dbg, err))?;
            map.end().map_err(|err| err_pass!(dbg, err))?;
            writer.write_all(b"\n").map_err(|err| err_pass!(dbg, err))?;
            Ok(writer.bytes_written())
        }
        RetainMode::Release => {
            writer.write_all(&(key.len() as u32).to_le_bytes()).map_err(|err| err_pass!(dbg, err))?;
            writer.write_all(key.as_bytes()).map_err(|err| err_pass!(dbg, err))?;
            let bytes = postcard::to_allocvec(state).map_err(|err| err_pass!(dbg, err))?;
            writer.write_all(&(bytes.len() as u32).to_le_bytes()).map_err(|err| err_pass!(dbg, err))?;
            writer.write_all(&bytes).map_err(|err| err_pass!(dbg, err))?;
            Ok(4 + key.len() + 4 + bytes.len())
        }
    }
}
///
/// ### Обертка над Write для подсчета записанных байт
struct CountingWriter<W: Write> {
    inner: W,
    bytes_written: usize,
}
//
impl<W: Write> CountingWriter<W> {
    pub fn new(inner: W) -> Self {
        Self { inner, bytes_written: 0 }
    }
    ///
    /// Получить текущее значение счетчика
    pub fn bytes_written(&self) -> usize {
        self.bytes_written
    }
    pub fn into_inner(self) -> W {
        self.inner
    }
}
//
impl<W: Write> Write for CountingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let result = self.inner.write(buf);
        if let Ok(bytes) = result {
            self.bytes_written += bytes;
        }
        result
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}
///
/// Basic Tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::File, io::BufWriter, sync::Arc, path::PathBuf};
    use sal_sync::{services::entity::{Point, PointHlr, Status, Cot}, services::types::Bool};
    struct MockChild;
    impl Eval<RetainCtx, RetainCtx> for MockChild {
        fn eval(&self, ctx: RetainCtx) -> RetainCtx {
            ctx
        }
    }
    fn mock_ctx(writer: Option<BufWriter<File>>) -> RetainCtx {
        RetainCtx {
            txid: 1,
            cache: Arc::new(crate::domain::FxSccHashMap::default()),
            path: PathBuf::from("dummy.log"),
            writer,
            file_size_bytes: 0,
            compactation_trigger: crate::services::task::retain::compactate_journal::Trigger::default(),
            compacted: false,
            flush_trigger: crate::services::task::retain::compactate_journal::Trigger::default(),
            error: None,
        }
    }
    #[test]
    fn test_append_journal_clears_buffer_on_success() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_journal_success.log");
        let file = File::create(&file_path).unwrap();
        let ctx = mock_ctx(Some(BufWriter::new(file)));
        let journal = AppendJournal::new("test", 1, RetainMode::Release, MockChild);
        let point = Point::Bool(PointHlr::new(0, "test_point", Bool(true), Status::Ok, Cot::Inf, chrono::Utc::now()));
        let event = RetainEvent { key: "tag_1".to_string(), p: point };
        journal.eval((Some(event), ctx));
        let buf = journal.buffer.take();
        assert!(buf.is_empty(), "Buffer must be empty after successful append");
        let _ = std::fs::remove_file(file_path);
    }
    #[test]
    fn test_append_journal_accumulates_without_writer() {
        let ctx = mock_ctx(None);
        let journal = AppendJournal::new("test", 1, RetainMode::Release, MockChild);
        let point = Point::Bool(PointHlr::new(0, "test_point", Bool(true), Status::Ok, Cot::Inf, chrono::Utc::now()));
        let event = RetainEvent { key: "tag_1".to_string(), p: point };
        journal.eval((Some(event), ctx));
        let buf = journal.buffer.take();
        assert_eq!(buf.len(), 1, "Buffer must accumulate events if writer is missing");
    }
}
