use sal_sync::{services::entity::{Point, PointHlr}, sync::channel::Sender};
use std::sync::{atomic::{AtomicUsize, Ordering}};
use crate::{domain::FnOutRef, services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult}};
///
/// ### Function | `FnToApiQueue`
/// 
/// Экспортирует данные (SQL-запросы) из вычислительного графа в очередь API.
/// - Отправляет данные только при наличии статуса `New` в потоке.
/// - Автоматически экранирует кавычки и удаляет мусорные символы перед отправкой.
/// - Игнорирует пустые строки (не отправляет их в базу данных).
/// 
/// **Example**
/// ```yaml
/// fn ToApiQueue:
///     queue: /App/ApiClient.in-queue
///     input fn Sql:
///         sql: "insert into public.event (pid,value,status,timestamp) values ({input2.value},{input1.value},{input1.status},'{input1.timestamp}');"
///         input1 fn ToInt:
///             input: point any every      # point: every point of any type
///         input2 fn PointId:
///             input: point any every
/// ```
#[derive(Debug)]
pub struct FnToApiQueue {
    txid: usize,
    id: String,
    kind: FnKind,
    input: FnOutRef,
    tx: Sender<Point>,
}
// 
impl FnToApiQueue {
    ///
    /// Returns `FnToApiQueue` new instance
    /// - `parent`: Имя родительского узла.
    /// - `txid`: Идентификатор сервиса-отправителя (родительский `Task`).
    /// - `input`: Входящий поток данных (SQL-строки).
    /// - `send`: Канал связи с целевым сервисом (например, `ApiClient`).
    pub fn new(parent: impl Into<String>, txid: usize, input: FnOutRef, send: Sender<Point>) -> Self {
        Self {
            txid,
            kind: FnKind::Fn,
            input,
            tx: send,
            id: format!("{}/FnToApiQueue{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
        }
    }
    ///
    /// Возвращает `Point` с обновленными `txid`, `name` и `value`
    #[inline]
    fn point_with(&self, p: &Point, value: String) -> Point {
        Point::String(match p {
            Point::String(hlr) => hlr.clone().with_value(value).with_txid(self.txid),
            _ => PointHlr::new(self.txid, p.name(), value, p.status(), p.cot(), p.ts()),
        })
    }
    ///
    /// Выполняет отправку точки в канал целевого сервиса
    fn send(&self, point: Point) {
        if log::max_level() >= log::LevelFilter::Trace {
            match self.tx.send(point.clone()) {
                Ok(_) => log::trace!("{}.out | Point sent: {:#?}", self.id, point),
                Err(err) => log::error!("{}.out | Send error: {:?}", self.id, err),
            };
        } else {
            if let Err(err) = self.tx.send(point) {
                log::error!("{}.out | Send error: {:#?}", self.id, err);
            }
        }
    }
}
// 
impl FnOut for FnToApiQueue {
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
        self.input.borrow().inputs()
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let Some(input) = flow.map(self.input.borrow_mut().out())? else { return Ok(None) };
        log::trace!("{}.out | input: {:?}", self.id, input);
        if flow.is_new() {
            let sql = prepare_for_sql(&(&input).to_string().as_string().value);
            if !sql.is_empty() {
                self.send(self.point_with(&input, sql));
            }
        }
        flow.wrap(input)
    }
    //
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
    }
}
///
/// Global static counter of FnToApiQueue instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Подготавливает сырую строку для безопасной вставки в SQL-запрос.
/// - Удаляет пробелы по краям
/// - Вырезает нулевые байты (\0)
/// - Экранирует одинарные кавычки
pub fn prepare_for_sql(input: &str) -> String {
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
///
/// Basic Tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};
    use sal_sync::sync::channel;
    use sal_sync::services::entity::{PointHlr, Status, Cot};
    #[derive(Debug)]
    struct MockNode {
        id: String,
        flow: FnResult<FnFlow, String>,
        called: usize,
    }
    impl MockNode {
        fn new(id: &str, flow: FnResult<FnFlow, String>) -> Rc<RefCell<Self>> {
            Rc::new(RefCell::new(Self { id: id.to_string(), flow, called: 0 }))
        }
    }
    impl FnOut for MockNode {
        fn id(&self) -> String { self.id.clone() }
        fn kind(&self) -> FnKind { FnKind::Fn }
        fn inputs(&self) -> Vec<String> { vec![] }
        fn out(&mut self) -> FnResult<FnFlow, String> {
            self.called += 1;
            self.flow.clone()
        }
        fn reset(&mut self) {}
    }
    fn mock_point_string(name: &str, val: &str) -> Point {
        Point::String(PointHlr::new(0, name, val.to_string(), Status::Ok, Cot::Inf, chrono::Utc::now()))
    }
    #[test]
    fn test_prepare_for_sql() {
        assert_eq!(prepare_for_sql("hello"), "hello");
        assert_eq!(prepare_for_sql("  world  "), "world");
        assert_eq!(prepare_for_sql("O'Connor"), "O''Connor");
        assert_eq!(prepare_for_sql("'; DROP TABLE users; --"), "''; DROP TABLE users; --");
        assert_eq!(prepare_for_sql("''; DROP TABLE users; --"), "''''; DROP TABLE users; --");
        assert_eq!(prepare_for_sql("bad\0data"), "baddata");
        assert_eq!(prepare_for_sql("   "), "");
    }
    #[test]
    fn test_fntoapiqueue_sends_on_new() {
        let (tx, rx) = channel::bounded(10);
        let input_point = mock_point_string("sql_cmd", "INSERT INTO db");
        let input = MockNode::new("mock", Ok(Some(FnFlow::New(input_point.clone()))));
        let mut node = FnToApiQueue::new("test", 1, input.clone(), tx);
        let res = node.out().unwrap().unwrap();
        assert!(matches!(res, FnFlow::New(_)));
        let sent = rx.try_recv().expect("Message should be sent to queue").unwrap();
        assert_eq!(sent.as_string().value, "INSERT INTO db");
        assert_eq!(sent.txid(), 1);
    }
    #[test]
    fn test_fntoapiqueue_ignores_old_flow() {
        let (tx, rx) = channel::bounded(10);
        let input_point = mock_point_string("sql_cmd", "UPDATE db");
        let input = MockNode::new("mock", Ok(Some(FnFlow::Old(input_point))));
        let mut node = FnToApiQueue::new("test", 1, input.clone(), tx);
        let res = node.out().unwrap().unwrap();
        assert!(matches!(res, FnFlow::Old(_)));
        assert!(matches!(rx.try_recv(), Ok(None)), "Old flow should not be sent");
    }
    #[test]
    fn test_fntoapiqueue_ignores_empty_sql() {
        let (tx, rx) = channel::bounded(10);
        let input_point = mock_point_string("sql_cmd", "   \0  ");
        let input = MockNode::new("mock", Ok(Some(FnFlow::New(input_point))));
        let mut node = FnToApiQueue::new("test", 1, input.clone(), tx);
        let res = node.out().unwrap().unwrap();
        assert!(matches!(res, FnFlow::New(_)));
        assert!(matches!(rx.try_recv(), Ok(None)), "Empty SQL should not be sent");
    }
}