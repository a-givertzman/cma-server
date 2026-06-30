#[cfg(test)]

use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{Service, Services, conf::{ConfTree, ServicesConf}, entity::{Name, Object, Point, Status, ToPoint}}, sync::{Handles, Owner, channel::{self, Receiver, Sender}}};
use testing::entities::test_value::Value;
use std::{cell::RefCell, collections::HashMap, fmt::{Debug, Display}, rc::Rc, sync::{Arc, Once, atomic::{AtomicBool, AtomicUsize, Ordering}}, thread::{self}};
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::{domain::{RECV_TIMEOUT, RecvTimeoutError}, services::task::{FlowContext, FnKind, FnResult, TaskConf, TaskEvalNode, TaskNodes}};
///
///
static INIT: Once = Once::new();
///
/// once called initialisation
fn init_once() {
    INIT.call_once(|| {
        // implement your initialisation code to be called only once for current test file
    })
}
///
/// 
fn init_each() {}
///
/// Тестируем функции регистратора:
/// - Определение рабочего цикла
/// - Формирование ID рабочего цикла
/// - Расчет основных метрик
///     - Crane Average load per cycle
///     - Crane Max load per cycle
///     - Winch1 Average load per cycle
///     - Winch2 Average load per cycle
///     - Winch3 Average load per cycle
///     - Кран | Характеристическое число
///     - Winch1 | Характеристическое число
///     - Winch2 | Характеристическое число
///     - Winch3 | Характеристическое число
///     - ...
/// - Расчет диапазонов загрузки
///     - 0.05..0.15
///     - 0.15..0.25
///     - 0.25..0.35
///     - 0.35..0.45
///     - 0.45..0.55
///     - 0.55..0.65
///     - 0.65..0.75
///     - 0.75..0.85
///     - 0.85..0.95
///     - 0.95..1.05
///     - 1.05..1.15
///     - 1.15..1_25
///     - 1.25..
/// - Учет событий
///     - MopsEvent
///     - AopsEvent
///     - CraneLoadEvent
///     - Winch1LoadEvent
///     - Winch2LoadEvent
///     - Winch3LoadEvent
/// 
/// Тестируем функции регистратора: Расчет относительной нагрузки
/// - Расчет craneLoadRelative
/// - Расчет winch1LoadRelative
/// - Реакция на изменение номинальной нагрузки "на лету"
#[test]
fn calc_relative_load() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    init_each();
    let dbg = "calc_relative_load";
    log::debug!("{dbg}");
    let self_name = Name::new("", dbg);
    let mut task_nodes = TaskNodes::without_retain(dbg, 0);
    let conf = TaskConf::read(&self_name, "src/tests/unit/services/task/cma_recorder/cma-recorder.yaml").unwrap();
    let services = Arc::new(Services::new(dbg, ServicesConf::new(dbg, ConfTree::empty()), None).unwrap());
    task_nodes.build_nodes(&Name::from(dbg), &conf, services).unwrap();
    log::debug!("{dbg} | Nodes: {:#?}", task_nodes.get_inputs());
    // В конфиге кран и Winch1 слушают одни и те же адреса, поэтому ожидаем синхронного изменения
    let test_data = [
        // step  Входной сигнал (Событие)                     Значение            craneRelative         winch1Relative
        // 1. Устанавливаем номинальную нагрузку (Делитель = 100.0) -> Относительная = 0.0 (так как текущий вес 0.0)
        (1,      "/App/ied13/db905_visual_data_fast/Winch1.LoadR0", Value::Real(100.0), Err(()),            Err(())),
        // 2. Поднимаем 25 тонн -> Относительная = 25% (0.25)
        (2,      "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(25.0),  Ok(0.25),           Ok(0.25)),
        // 3. Поднимаем 125 тонн -> Относительная = 125% (1.25)
        (3,      "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(125.0), Ok(1.25),           Ok(1.25)),
        // 4. Динамически меняется номинал (вылет стрелы увеличился, номинал упал до 50.0). Вес висит тот же (125.0).
        // Относительная нагрузка должна мгновенно прыгнуть до 250% (2.5)
        (4,      "/App/ied13/db905_visual_data_fast/Winch1.LoadR0", Value::Real(50.0),  Ok(2.5),            Ok(2.5)),
    ];
    let flow = FlowContext::new();
    for (step, name, val, target_crane, target_winch1) in test_data {
        let point = val.to_point(0, name);
        task_nodes.eval(point);
        // Извлекаем целевые узлы напрямую из графа по именам переменных (let)
        // (Если метод vars() скрыт, сделай его pub(crate) или добавь getter get_var(&str) -> Option<FnInOutRef>)
        let crane_node = task_nodes.get_var("craneLoadRelative").expect("Variable 'craneLoadRelative' not found in DAG");
        let winch1_node = task_nodes.get_var("winch1LoadRelative").expect("Variable 'winch1LoadRelative' not found in DAG");
        // Проверяем Кран
        match flow.ignore(crane_node.borrow_mut().out()) {
            Ok(Some(result)) => {
                let actual = result.value().as_double();
                assert!((actual - target_crane.unwrap()).abs() < f64::EPSILON, "{dbg} | Step {step} | craneLoadRelative \n result: {actual} \n target: {:?}", target_crane);
            }
            Ok(None) => panic!("{dbg} | Step {step} | craneLoadRelative returned None"),
            Err(err) => if target_crane.is_err() {} else { panic!("{dbg} | Step {step} | craneLoadRelative returned err: {:?}", err) },
        }
        // Проверяем Лебедку 1
        match flow.ignore(winch1_node.borrow_mut().out()) {
            Ok(Some(result)) => {
                let actual = result.value().as_double();
                assert!((actual - target_winch1.unwrap()).abs() < f64::EPSILON, "{dbg} | Step {step} | winch1LoadRelative \n result: {actual} \n target: {:?}", target_winch1);
            }
            Ok(None) => panic!("{dbg} | Step {step} | winch1LoadRelative returned None"),
            Err(err) => if target_winch1.is_err() {} else { panic!("{dbg} | Step {step} | winch1LoadRelative returned err: {:?}", err) },
        }
    }
}
