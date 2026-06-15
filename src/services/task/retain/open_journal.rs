use std::{cell::RefCell, fs::OpenOptions, io::{BufWriter, Write}};
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use crate::{err, err_pass, services::task::{FlushConf, RetainEvent}};
use super::{Eval, RetainCtx};

///
/// ### Открывает новый файл
/// 
/// Возвращает в контекст
/// - BufWriter
/// - Размер файла в байтах
pub struct OpenJournal<Child> {
    conf: FlushConf,
    ctx: RefCell<Option<RetainCtx>>,
    child: Child,
    dbg: Dbg,
}
//
impl<Child> OpenJournal<Child> {
    pub fn new(parent: impl Into<String>, conf: &FlushConf, ctx: RetainCtx, child: Child) -> Self {
        let dbg = Dbg::new(parent, crate::domain::me::<Self>());
        Self {
            conf: conf.clone(),
            ctx: RefCell::new(Some(ctx)),
            child,
            dbg,
        }
    }
    ///
    /// ### Закрывает файл если был открыт.
    /// Сохраняет все остатки из буфера на диск.
    #[named]
    pub fn close(&self) -> Result<(), Error> {
        if let Some(ctx) = self.ctx.borrow_mut().as_mut() {
            if let Some(w) = &mut ctx.writer {
                w.flush().map_err(|err| err_pass!(self.dbg, err))?;
                w.get_ref().sync_all().map_err(|err| err_pass!(self.dbg, err))?;
            }
        }
        Ok(())
    }
}
//
impl<Child> Eval<Option<RetainEvent>, Result<(), Error>> for OpenJournal<Child>
where
    Child: Eval<(Option<RetainEvent>, RetainCtx), RetainCtx>, {
    #[named]
    fn eval(&self, event: Option<RetainEvent>) -> Result<(), Error> {
        match self.ctx.borrow_mut().as_mut() {
            Some(ctx) => {
                let path = ctx.path.clone();
                if ctx.writer.is_none() {
                    if let Some(parent) = std::path::Path::new(&path).parent() {
                        std::fs::create_dir_all(parent).map_err(|err| err_pass!(self.dbg, err))?;
                    }
                    let file = OpenOptions::new().append(true).create(true).open(&path).map_err(|err| err_pass!(self.dbg, err))?;
                    ctx.file_size_bytes = file.metadata()
                        .map_err(|err| err_pass!(self.dbg, err))?
                        .len();
                    ctx.writer = Some(BufWriter::with_capacity(self.conf.bytes_limit, file));
                }
            }
            None => return Err(err!(self.dbg, "Context not found")),
        }
        match self.ctx.replace(None) {
            Some(ctx) => {
                let mut ctx = self.child.eval((event, ctx));
                let result = ctx.error.take();
                self.ctx.replace(Some(ctx));
                match result {
                    Some(err) => Err(err_pass!(self.dbg, err)),
                    None => Ok(()),
                }
            }
            None => Err(err!(self.dbg, "Context not found")),
        }
    }
}
