use function_name::named;
use sal_core::error::Error;
use sal_sync::{services::entity::{Point, PointHlr}, sync::channel::Sender};
use sqlparser::{dialect::PostgreSqlDialect, tokenizer::{Token, Tokenizer}};
use std::sync::{atomic::{AtomicUsize, Ordering}};
use crate::{domain::{FnOutRef, TryTo}, err_pass, services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult}};
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
    /// Подготавливает сырой SQL-запрос.
    /// - Удаляет пробелы по краям
    /// - Вырезает нулевые байты (\0)
    /// - Экранирует одинарные кавычки
    #[named]
    fn prepare_raw_sql(&self, sql: &str) -> Result<String, Error> {
        let dialect = PostgreSqlDialect {};
        let tokens = Tokenizer::new(&dialect, sql.trim())
            .tokenize()
            .map_err(|err| err_pass!(self.id, err, "Invalid SQL: {}", sql))?;
        let mut result = String::with_capacity(sql.len() + 16);
        for token in tokens {
            match token {
                // Если токен — это строка в одинарных кавычках 'value'
                Token::SingleQuotedString(ref text) => {
                    // Вырезаем нулевые байты для безопасности (как в вашей функции)
                    let clean_text: String = text.chars().filter(|&c| c != '\0').collect();
                    let escaped_text = clean_text.replace('\'', "''");
                    result.push('\'');
                    result.push_str(&escaped_text);
                    result.push('\'');
                }
                // Все остальные токены (INSERT, INTO, имена таблиц, скобки) пропускаем "как есть"
                _ => {
                    // Восстанавливаем исходное форматирование токена
                    result.push_str(&token.to_string());
                }
            }
        }
        Ok(result)
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
    #[named]
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let Some(input) = flow.map(self.input.borrow_mut().out())? else { return Ok(None) };
        log::trace!("{}.out | input: {:?}", self.id, input);
        if flow.is_new() {
            let sql: String = (&input).try_to().map_err(|err: sal_core::error::Error| err_pass!(self.id, err).to_string())?;
            let sql = self.prepare_raw_sql(&sql).map_err(|err: sal_core::error::Error| err_pass!(self.id, err).to_string())?;
            if !sql.is_empty() {
                self.send(self.point_with(&input, sql));
            }
        }
        flow.wrap(input)
    }
    //
    fn hard_reset(&mut self) {
        self.input.borrow_mut().hard_reset();
    }
    //
    fn reset(&mut self) {}
}
///
/// Global static counter of FnToApiQueue instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
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
        fn hard_reset(&mut self) {}
        fn reset(&mut self) {}
    }
    fn mock_point_string(name: &str, val: &str) -> Point {
        Point::String(PointHlr::new(0, name, val.to_string(), Status::Ok, Cot::Inf, chrono::Utc::now()))
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