use std::{marker::PhantomData, sync::Arc};
use sal_core::dbg::Dbg;
use vibro_core::Eval;

type ApiClient = crate::infra::ApiClient;
pub struct SqlExport<F, OnErr, Ctx, Child> {
    ///  Rkbtyn для отправки SQL запросов в БД
    api_client: Arc<ApiClient>,
    /// Замыкание в котором проверяем наличие ошибок в контексте
    on_err: OnErr,
    /// Замыкание в котором формируются SQL запросы.
    builder: F,
    /// Предыдущий узел конвейера вычислений (например, угловой ресемплер или оконный фильтр).
    child: Child,
    _ctx: PhantomData<Ctx>, 
    /// Полное имя узла для отладки
    dbg: Dbg,
}
//
impl<F, OnErr, Ctx, Child> SqlExport<F, OnErr, Ctx, Child>
where
    F: Fn(&Ctx) -> Vec<String>,
    OnErr: Fn(Ctx) -> Result<Ctx, Ctx>,
    Child: Eval<Ctx, Ctx> + Send + 'static {
    ///
    /// ### Returns `SqlExport` new instance
    /// - `parent` - Идентификатор родительской сущности (для отладки).
    /// - `on_err` - Замыкание в котором проверяем наличие ошибок в контексте.
    /// - `builder` - Замыкание в котором формируются SQL запросы.
    /// - `child` - Дочерний (предыдущий) расчетный шаг
    pub fn new(parent: &Dbg, api_client: Arc<ApiClient>, on_err: OnErr, builder: F, child: Child) -> Self {
        let dbg = Dbg::new(parent, crate::me::<Self>());
        Self {
            api_client,
            on_err,
            builder,
            child,
            _ctx: PhantomData,
            dbg,
        }
    }
}
//
impl<F, OnErr, Ctx, Child> Eval<Ctx, Ctx> for SqlExport<F, OnErr, Ctx, Child>
where
    F: Fn(&Ctx) -> Vec<String>,
    OnErr: Fn(Ctx) -> Result<Ctx, Ctx>,
    Child: Eval<Ctx, Ctx> + Send + 'static {
    //
    #[inline]
    fn eval(&self, ctx: Ctx) -> Ctx {
        let mut ctx = self.child.eval(ctx);
        let ctx = match (self.on_err)(ctx) {
            Err(ctx) => return ctx,
            Ok(ctx) => ctx,
        };
        let sqls = (self.builder)(&ctx);
        let mut results = Vec::with_capacity(sqls.len());
        for sql in sqls {
            let r = self.api_client.fetch(sql);
            results.push(r);
        }
        for r in results.into_iter().map(|r| r.wait()).flatten() {
            if let Err(err) = r {
                log::warn!("{}.eval | {:?}", self.dbg, err);
                break;
            }
        }
        ctx
    }
    //
    fn exit(&self) {
        self.child.exit();
    }
}
/// Подготавливает сырую строку для безопасной вставки в SQL-запрос.
/// - Удаляет пробелы по краям
/// - Вырезает нулевые байты (\0)
/// - Экранирует одинарные кавычки
pub(super) fn escape(input: &str) -> String {
    let trimmed = input.trim();
    // +8 байт — запас под несколько кавычек
    let mut result = String::with_capacity(trimmed.len() + 8);
    for c in trimmed.chars() {
        match c {
            '\0' => continue,
            '\'' => result.push_str("''"),
            _ => result.push(c),
        }
    }
    result
}
