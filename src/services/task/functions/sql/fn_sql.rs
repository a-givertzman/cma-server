use function_name::named;
use sal_core::error::Error;
use sal_sync::{collections::FxIndexMap, services::{Services, entity::{Name, Point, PointHlr}, task::functions::FnConfig}};
use std::{sync::{atomic::{AtomicUsize, Ordering}, Arc}};
use crate::{
    domain::{
        FnOutRef, PointMeta, format::{FormatPoint, Sufix}
    }, err, err_pass, services::task::{
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
    #[named]
    pub fn new(parent: impl Into<String>, conf: &FnConfig, nodes: &mut TaskNodes, services: Arc<Services>) -> Result<FnSql, Error> {
        let self_name = Name::new(parent, format!("FnSql{}", COUNT.fetch_add(1, Ordering::Relaxed)));
        let id = self_name.join();
        let txid = nodes.txid();
        let sql = conf.param("sql")
            .ok_or_else(|| err!(id, "Can't find 'sql'"))?
            .as_param();
        let sql = sql.conf.as_str()
            .ok_or_else(|| err!(id, "Wrong conf in 'sql': {:?}", sql.conf))?;
        let sql = FormatPoint::new(sql).map_err(|err| err_pass!(id, err))?;
        let markers = sql.markers();
        let mut inputs = FxIndexMap::default();
        for (marker, (name, sufix)) in markers {
            log::trace!("{}.new | input name: {:?}", id, name);
            let input_conf = conf.input_conf(&name)
                .map_err(|err| err_pass!(id, err, "Can't get input '{name}' conf"))?;
            inputs.insert(
                marker, 
                (
                    FnBuilder::new(&self_name, input_conf, nodes, services.clone())
                        .map_err(|err| err_pass!(id, err, "Can't build input '{name}'"))?,
                    name,
                    sufix,
                )
            );
        }
        Ok(FnSql {
            txid,
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
///
/// Basic Tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};
    use sal_sync::services::entity::{Cot, Status};
    #[derive(Debug)]
    struct MockNode {
        id: String,
        flow: FnResult<FnFlow, String>,
    }
    impl MockNode {
        fn new(id: &str, flow: FnResult<FnFlow, String>) -> Self {
            Self { id: id.to_string(), flow }
        }
    }
    impl FnOut for MockNode {
        fn id(&self) -> String { self.id.clone() }
        fn kind(&self) -> FnKind { FnKind::Var }
        fn inputs(&self) -> Vec<String> { vec![] }
        fn out(&mut self) -> FnResult<FnFlow, String> { self.flow.clone() }
        fn reset(&mut self) {}
    }
    fn mock_point_int(name: &str, val: i64) -> Point {
        Point::Int(PointHlr::new(0, name, val, Status::Ok, Cot::Inf, chrono::Utc::now()))
    }
    fn mock_point_string(name: &str, val: &str) -> Point {
        Point::String(PointHlr::new(0, name, val.to_string(), Status::Ok, Cot::Inf, chrono::Utc::now()))
    }
    #[test]
    fn test_sql_metric_evaluation_new() {
        let template = "UPDATE test SET status = '{st.value}', val = {val.value} WHERE id = '{id.name}';";
        let sql = FormatPoint::new(template).unwrap();
        let mut inputs = FxIndexMap::default();
        let st_node = Rc::new(RefCell::new(MockNode::new("st", Ok(Some(FnFlow::New(mock_point_string("st", "Valid")))))));
        let val_node = Rc::new(RefCell::new(MockNode::new("val", Ok(Some(FnFlow::New(mock_point_int("val", 42)))))));
        let id_node = Rc::new(RefCell::new(MockNode::new("id", Ok(Some(FnFlow::New(mock_point_int("pump_1", 100)))))));
        inputs.insert("st.value".to_string(), (st_node.clone() as FnOutRef, "st".to_string(), Sufix::Value));
        inputs.insert("val.value".to_string(), (val_node.clone() as FnOutRef, "val".to_string(), Sufix::Value));
        inputs.insert("id.name".to_string(), (id_node.clone() as FnOutRef, "id".to_string(), Sufix::Name));
        let mut metric = FnSql {
            txid: 0,
            kind: FnKind::Fn,
            inputs,
            sql,
            id: "parent/SqlMetric_test".to_string(),
        };
        let res = metric.out().unwrap().unwrap();
        assert!(matches!(res, FnFlow::New(_)), "Ожидаем статус New при новых данных");
        assert_eq!(res.value().to_string().as_string().value, "UPDATE test SET status = 'Valid', val = 42 WHERE id = 'pump_1';");
    }
    #[test]
    fn test_sql_metric_taint_tracking_old() {
        let template = "INSERT INTO log (msg) VALUES ('{msg.value}');";
        let sql = FormatPoint::new(template).unwrap();
        let mut inputs = FxIndexMap::default();
        let msg_node = Rc::new(RefCell::new(MockNode::new("msg", Ok(Some(FnFlow::Old(mock_point_string("msg", "System OK")))))));
        inputs.insert("msg.value".to_string(), (msg_node.clone() as FnOutRef, "msg".to_string(), Sufix::Value));
        let mut metric = FnSql {
            txid: 0,
            kind: FnKind::Fn,
            inputs,
            sql,
            id: "parent/SqlMetric_test".to_string(),
        };
        let res = metric.out().unwrap().unwrap();
        assert!(matches!(res, FnFlow::Old(_)), "Если источник отдал Old, узел обязан пробросить Old");
        assert_eq!(res.value().to_string().as_string().value, "INSERT INTO log (msg) VALUES ('System OK');");
    }
    #[test]
    fn test_sql_metric_cold_mode() {
        let template = "SELECT * FROM {table.value};";
        let sql = FormatPoint::new(template).unwrap();
        let mut inputs = FxIndexMap::default();
        let table_node = Rc::new(RefCell::new(MockNode::new("table", Ok(None))));
        inputs.insert("table.value".to_string(), (table_node.clone() as FnOutRef, "table".to_string(), Sufix::Value));
        let mut metric = FnSql {
            txid: 0,
            kind: FnKind::Fn,
            inputs,
            sql,
            id: "parent/SqlMetric_test".to_string(),
        };
        let res = metric.out().unwrap();
        assert!(res.is_none(), "При отсутствии сигнала (Ok(None)) узел должен молча уснуть");
    }
}
