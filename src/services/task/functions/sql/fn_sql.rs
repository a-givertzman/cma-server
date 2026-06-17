use sal_core::error::Error;
use sal_sync::{collections::FxIndexMap, services::{Services, entity::{Name, Point, PointHlr}, task::functions::FnConfig}};
use std::{sync::{atomic::{AtomicUsize, Ordering}, Arc}};
use crate::{
    domain::{
        FnOutRef, PointMeta, format::{FormatPoint, Sufix}
    },
    services::task::{
        FlowContext, FnFlow, functions::{FnBuilder, FnKind, FnOut, FnResult}, task_nodes::TaskNodes
    }
};
///
/// ### Function | FnSql
/// 
/// Строит SQL-запрос, подставляя актуальные значения входов вместо маркеров {xyz}.
///
/// Является чистой функцией (Stateless), не хранит внутренний кэш и пересобирает строку при каждом такте.
/// 
/// **Example 1**
/// - `input1.value = 'Valid'`
/// - `input2.value = 10`
/// - `inpur1.timestamp = '20'`
/// - `input1.status = Ok`
/// "UPDATE table SET kind = '{input1}' WHERE id = '{input2}';"    =>  UPDATE table SET kind = 'Valid' WHERE id = '10';
/// 
/// **Example 2**
/// ```yaml
/// fn Sql:
///     sql: "UPDATE table_name SET value = '{input1}' WHERE id = '{input2}';"
///     input1: point int '/path/Point.Name'
///     input2: const int 11
/// ```
#[derive(Debug)]
pub struct FnSql {
    txid: usize,
    name: Name,
    kind: FnKind,
    /// `Map<marker, (input, name, sufix)>`
    inputs: FxIndexMap<String, (FnOutRef, String, Sufix)>,
    sql: FormatPoint,
    id: String,
}
//
// 
impl FnSql {
    ///
    /// Returns `FnSql` new instance
    /// - `parent`: Идентификатор родительского узла
    /// - `inputs`: Вектор входных сигналов, должен содержать не менее одного входа
    /// - `nodes`: Граф `TaskNodes`
    /// - `services`: Ссылка на контейнер всех сервисов
    pub fn new(parent: impl Into<String>, conf: &FnConfig, nodes: &mut TaskNodes, services: Arc<Services>) -> Result<FnSql, Error> {
        let self_name = Name::new(parent, format!("FnSql{}", COUNT.fetch_add(1, Ordering::Relaxed)));
        let id = self_name.join();
        let error = Error::new(&id, "new");
        let txid = nodes.txid();
        let sql = conf.param("sql")
            .ok_or_else(|| error.err(format!("Can't find 'sql'")))?
            .as_param();
        let sql = sql.conf.as_str()
            .ok_or_else(|| error.err(format!("Wrong conf in 'sql': {:?}", sql.conf)))?;
        let sql = FormatPoint::new(sql).map_err(|err| error.pass(err))?;
        let markers = sql.markers();
        let mut inputs = FxIndexMap::default();
        for (marker, (name, sufix)) in markers {
            log::trace!("{}.new | input name: {:?}", id, name);
            let input_conf = conf.input_conf(&name).unwrap();
            inputs.insert(
                marker, 
                (
                    FnBuilder::new(&self_name, input_conf, nodes, services.clone())
                        .map_err(|err| error.pass_with(format!("Can't build input '{name}'"), err))?,
                    name,
                    sufix,
                )
            );
        }
        Ok(FnSql {
            txid,
            name: self_name,
            kind: FnKind::Fn,
            inputs,
            sql,
            id,
        })
    }
    ///
    /// Возвращает `PointHlr` с обновленными `txid`, `name` и `value`
    #[inline]
    fn point_with<T>(txid: usize, meta: &PointMeta, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)
    }
}
// 
impl FnOut for FnSql {
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
        for (_, (input, _, _)) in &self.inputs {
            inputs.extend(input.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let inputs: FxIndexMap<&String, (FnResult<FnFlow, String>, &String, &Sufix)> = self.inputs.iter().map(|(marker, (input, name, sufix))| {
            (marker, (input.borrow_mut().out(), name, sufix))
        }).collect();
        let mut flow = FlowContext::new();
        let mut meta = PointMeta::default();
        for (marker, (input, name, _sufix)) in inputs {
            // log::trace!("{}.out | name: {:?}, sufix: {:?}", self_id, name, sufix);
            log::trace!("{}.out | input: {:?} - found", self.id, name);
            let Some(input) = flow.map(input)? else { return Ok(None) };
            meta = meta.update_latest(&input).update_status(&input);
            self.sql.insert(marker, input);
        }
        let value = self.sql.out();
        log::trace!("{}.out | sql: {:?}", self.id, self.sql.out());
        flow.wrap(Point::String(Self::point_with(self.txid, &meta, &self.id, value)))
    }
    //
    fn reset(&mut self) {
        for (_, (input, _, _)) in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
///
/// Global static counter of FnSql instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
