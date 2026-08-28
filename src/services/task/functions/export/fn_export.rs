use function_name::named;
use sal_core::error::Error;
use sal_sync::{services::{entity::{Point, PointConf, PointType, PointHlr}, types::Bool}, sync::channel::Sender};
use std::sync::{atomic::{AtomicUsize, Ordering}};
use crate::{
    domain::{FnOutRef, TryTo}, err, err_pass, services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult}
};
///
/// ### Function | FnExport
///  
/// Узел экспорта сигнала `Point` другому сервису
/// 
/// - Сигнал отправляется в очередь только при выполнении условий:
///     - Входящий поток имеет статус `New` (изменение значения).
///     - Разрешающий сигнал `enable` активен (через `FnEnable`).
///     - Указан маршрут `send-to`.
/// - Если указан `conf`, то сигнал будет конвертирован в указанный тип и отправлен с указанным именем
/// 
/// - Функция абсолютно прозрачна, не разрывает граф, всегда возвращает оригинал со входа.
/// 
/// **Example**
/// 
/// ```yaml
/// fn Export:
///     enable: const bool true         # optional, default true
///     send-to: /AppTest/MultiQueue.in-queue
///     conf point Point.Name:          # full name will be: /App/Task/Point.Name
///         type: 'Bool'
///     input: point string /AppTest/Exit
/// ```
#[derive(Debug)]
pub struct FnExport {
    txid: usize,
    kind: FnKind,
    conf: Option<PointConf>,
    input: FnOutRef,
    tx: Option<Sender<Point>>,
    id: String,
}
//
impl FnExport {
    ///
    /// Creates new instance of FnExport
    /// - `parent` - Name of the parent node
    /// - `txid`: Идентификатор сервиса отправителя (родительский `Task` в данном случае)
    /// - `conf` - Configuration of the Point to be produced. If None, input Point is sent
    /// - `input`: Incoming input events
    /// - `send`: Link of the destination service
    pub fn new(parent: impl Into<String>, txid: usize, conf: Option<PointConf>, input: FnOutRef, send: Option<Sender<Point>>) -> Self {
        let id = format!("{}/FnExport{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        Self {
            txid,
            kind: FnKind::Fn,
            conf,
            input,
            tx: send,
            id,
        }
    }
    ///
    /// Конвертирует оригинальный сигнал в целевой тип с обновлением txid и имени
    #[named]
    fn convert_to(&self, name: &str, p: &Point, target: &PointType) -> Result<Point, Error> {
        match target {
            PointType::Bool => Ok(Point::Bool(PointHlr::new(self.txid, name, Bool(p.try_to()?), p.status(), p.cot(), p.ts()))),
            PointType::Int => Ok(Point::Int(PointHlr::new(self.txid, name, p.try_to()?, p.status(), p.cot(), p.ts()))),
            PointType::Real => Ok(Point::Real(PointHlr::new(self.txid, name, p.try_to()?, p.status(), p.cot(), p.ts()))),
            PointType::Double => Ok(Point::Double(PointHlr::new(self.txid, name, p.try_to()?, p.status(), p.cot(), p.ts()))),
            PointType::String => Ok(p.to_string().with_txid(self.txid).with_name(name)),
            PointType::Bytes => match p {
                Point::Bytes(hlr) => Ok(Point::Bytes(hlr.clone()).with_txid(self.txid).with_name(name)),
                _ => Err(err!(self.id, "Invalid input type {:?} for converting into 'Bytes'", p.typ())),
            },
            PointType::Json => match p {
                Point::Bytes(hlr) => Ok(Point::Bytes(hlr.clone()).with_txid(self.txid).with_name(name)),
                _ => Err(err!(self.id, "Invalid input type {:?} for converting into 'Json'", p.typ())),
            },
        }
    }
    ///
    /// Выполняет отправку точки в канал целевого сервиса
    fn send(&self, point: Point) {
        if let Some(tx) = &self.tx {
            if log::max_level() >= log::LevelFilter::Trace {
                match tx.send(point.clone()) {
                    Ok(_) => log::trace!("{}.out | Point sent: {:#?}", self.id, point),
                    Err(err) => log::error!("{}.out | Send error: {:?}", self.id, err),
                };
            } else {
                if let Err(err) = tx.send(point) {
                    log::error!("{}.out | Send error: {:#?}", self.id, err);
                }
            }
        }
    }
}
//
impl FnOut for FnExport {
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
        // log::debug!("{}.out | input: {:?}", self.id, input);
        if flow.is_new() {
            let point = match &self.conf {
                Some(conf) => self.convert_to(&conf.name, &input, &conf.type_)
                    .map_err(|err| err_pass!(self.id, err).to_string())?,
                None => input.clone().with_txid(self.txid),
            };
            self.send(point);
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
/// Global static counter of FnExport instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::unbounded;
    use debugging::session::{DebugSession, LogLevel};
    use sal_sync::services::entity::{Point, PointHlr, Status, Cot};
    use std::{cell::RefCell, rc::Rc};
    // Простой Mock-источник данных
    #[derive(Debug)]
    struct MockNode {
        flow: Option<FnFlow>,
        inputs: Vec<String>,
    }
    impl FnOut for MockNode {
        fn id(&self) -> String { "MockNode".to_string() }
        fn kind(&self) -> FnKind { FnKind::Fn }
        fn inputs(&self) -> Vec<String> { self.inputs.clone() }
        fn out(&mut self) -> FnResult<FnFlow, String> { Ok(self.flow.clone()) }
        fn hard_reset(&mut self) {}
        fn reset(&mut self) {}
    }
    fn mock_int_point(val: i64) -> Point {
        Point::Int(PointHlr::new(0, "SourcePoint", val, Status::Ok, Cot::Inf, chrono::Utc::now()))
    }
    #[test]
    fn test_export_sends_only_on_new() {
        DebugSession::new().filter(LogLevel::Debug).init().unwrap();
        let (tx, rx) = unbounded();
        let source_point = mock_int_point(42);
        let input = Rc::new(RefCell::new(MockNode { 
            flow: Some(FnFlow::New(source_point.clone())), 
            inputs: vec![] 
        }));
        let mut node = FnExport::new("Test", 99, None, input.clone(), Some(tx));
        // Такт 1: Статус New -> Должны отправить
        let res1 = node.out().unwrap().unwrap();
        assert!(matches!(res1, FnFlow::New(_)));
        let sent_point = rx.try_recv().expect("Point must be sent on New flow").unwrap();
        assert_eq!(sent_point.txid(), 99, "TxId должен быть перезаписан для защиты от петель");
        // Такт 2: Статус Old -> Отправки быть не должно
        input.borrow_mut().flow = Some(FnFlow::Old(source_point));
        let res2 = node.out().unwrap().unwrap();
        assert!(matches!(res2, FnFlow::Old(_)), "\ntarget: FnFlow::Old(_) \nresult: {:?}", res2);
        let event = rx.try_recv();
        assert!(event == Ok(None), "Point MUST NOT be sent on Old flow \ntarget: Ok(None) \nresult: {:?}", event);
    }
    #[test]
    fn test_export_transparent_tap() {
        let source_point = mock_int_point(100);
        let input = Rc::new(RefCell::new(MockNode { 
            flow: Some(FnFlow::New(source_point)), 
            inputs: vec![] 
        }));
        // Создаем ноду без канала отправки (нет send-to)
        let mut node = FnExport::new("Test", 99, None, input, None);
        let res = node.out().unwrap().unwrap();
        // Убеждаемся, что исходные данные и статус потока не повреждены
        assert!(matches!(res, FnFlow::New(_)));
        assert_eq!(res.value().as_int().value, 100);
    }
}
