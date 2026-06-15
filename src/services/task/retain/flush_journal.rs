use std::io::Write;
use function_name::named;
use sal_core::dbg::Dbg;
use crate::err_pass;
use super::{Eval, RetainCtx};

///
/// Выполняет сброс буфера, гарантируя, что накопленные данные будут переданы OS.
/// Физическую запись на диск OS выполнит по своему усмотрению.
pub struct FlushJournal<Child> {
    child: Child,
    dbg: Dbg,
}
//
impl<Child> FlushJournal<Child> {
    pub fn new(parent: impl Into<String>, child: Child) -> Self {
        let dbg = Dbg::new(parent, crate::domain::me::<Self>());
        Self {
            child,
            dbg,
        }
    }
}
//
impl<Child> Eval<RetainCtx, RetainCtx> for FlushJournal<Child>
where
    Child: Eval<RetainCtx, RetainCtx>,
{
    #[named]
    fn eval(&self, mut ctx: RetainCtx) -> RetainCtx {
        if ctx.flush_trigger.is_exceeded(0u64) {
            ctx.flush_trigger.start();
            if let Some(writer) = &mut ctx.writer {
                if let Err(err) = writer.flush() {
                    log::warn!("{}.run | Can't flush to '{:?}', error: {:?}", self.dbg, ctx.path.display(), err);
                }
            }
        }
        let mut ctx = self.child.eval(ctx);
        ctx.error = ctx.error.map(|err| err_pass!(self.dbg, err));
        ctx
    }
}
