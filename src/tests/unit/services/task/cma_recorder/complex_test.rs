#[cfg(test)]

use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{Service, Services, conf::{ConfTree, ServicesConf}, entity::{Cot, Name, Object, Point, PointHlr, Status, ToPoint}, types::Bool}, sync::{Handles, Owner, channel::{self, Receiver, Sender}}};
use testing::entities::test_value::Value;
use std::{cell::RefCell, collections::HashMap, fmt::{Debug, Display}, rc::Rc, sync::{Arc, Once, atomic::{AtomicBool, AtomicUsize, Ordering}}, thread::{self}, time::Duration};
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::{domain::{RECV_TIMEOUT, RecvTimeoutError}, services::task::{FlowContext, FnKind, FnResult, TaskConf, TaskEvalNode, TaskNodes}, short_type_name};
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
/// Расчет относительной нагрузки
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
    // log::debug!("{dbg} | Nodes: {:#?}", task_nodes.get_inputs());
    // В конфиге кран и Winch1 слушают одни и те же адреса, поэтому ожидаем синхронного изменения
    let test_data: &[(i32, &str, Value, Result<Option<f32>, ()>, Result<Option<f32>, ()>, Result<Option<f32>, ()>, Result<Option<f32>, ()>)] = &[
        // step  Входной сигнал (Событие)                           Значение            crane                   winch1                  winch2                  winch3
        // - Crane
        // Устанавливаем номинальную нагрузку (Делитель = 100.0) -> Относительная = 0.0 (так как текущий вес 0.0)
        (01,      "/App/ied13/db905_visual_data_fast/Winch1.LoadR0", Value::Real(100.0), Ok(None),              Ok(None),               Ok(None),               Ok(None)),
        // Поднимаем 25 тонн -> Относительная = 25% (0.25)
        (02,      "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(25.0),  Ok(Some(0.25)),        Ok(Some(0.25)),         Ok(None),               Ok(None)),
        // Поднимаем 125 тонн -> Относительная = 125% (1.25)
        (03,      "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(125.0), Ok(Some(1.25)),        Ok(Some(1.25)),         Ok(None),               Ok(None)),
        // Меняется номинал. Вес висит тот же (125.0). Относительная нагрузка должна прыгнуть до 250% (2.5)
        (04,      "/App/ied13/db905_visual_data_fast/Winch1.LoadR0", Value::Real(50.0),  Ok(Some(2.50)),        Ok(Some(2.50)),         Ok(None),               Ok(None)),
        // - Winch 2
        // Устанавливаем номинальную нагрузку (Делитель = 80.0) -> Относительная = 0.0 (так как текущий вес 0.0)
        (05,      "/App/ied13/db905_visual_data_fast/Winch2.LoadR0", Value::Real(80.0),  Ok(Some(2.50)),        Ok(Some(2.50)),         Ok(None),               Ok(None)),
        // Поднимаем 20 тонн -> Относительная = 25% (0.25)
        (06,      "/App/ied14/db906_visual_data/Winch2.Load",        Value::Real(20.0),  Ok(Some(2.50)),        Ok(Some(2.50)),         Ok(Some(0.25)),         Ok(None)),
        // Поднимаем 100 тонн -> Относительная = 125% (1.25)
        (07,      "/App/ied14/db906_visual_data/Winch2.Load",        Value::Real(100.0), Ok(Some(2.50)),        Ok(Some(2.50)),         Ok(Some(1.25)),         Ok(None)),
        // Меняется номинал. Вес висит тот же (125.0). Относительная нагрузка должна прыгнуть до 250% (2.5)
        (08,      "/App/ied13/db905_visual_data_fast/Winch2.LoadR0", Value::Real(40.0),  Ok(Some(2.50)),        Ok(Some(2.50)),         Ok(Some(2.50)),         Ok(None)),
        // - Winch 3
        // Устанавливаем номинальную нагрузку (Делитель = 70.0) -> Относительная = 0.0 (так как текущий вес 0.0)
        (10,      "/App/ied13/db905_visual_data_fast/Winch3.LoadR0", Value::Real(70.0),  Ok(Some(2.50)),        Ok(Some(2.50)),         Ok(Some(2.50)),         Ok(None)),
        // Поднимаем 20 тонн -> Относительная = 25% (0.25)
        (11,      "/App/ied14/db906_visual_data/Winch3.Load",        Value::Real(17.5),  Ok(Some(2.50)),        Ok(Some(2.50)),         Ok(Some(2.50)),         Ok(Some(0.25))),
        // Поднимаем 100 тонн -> Относительная = 125% (1.25)
        (12,      "/App/ied14/db906_visual_data/Winch3.Load",        Value::Real(87.5),  Ok(Some(2.50)),        Ok(Some(2.50)),         Ok(Some(2.50)),         Ok(Some(1.25))),
        // Меняется номинал. Вес висит тот же (125.0). Относительная нагрузка должна прыгнуть до 250% (2.5)
        (13,      "/App/ied13/db905_visual_data_fast/Winch3.LoadR0", Value::Real(35.0),  Ok(Some(2.50)),        Ok(Some(2.50)),         Ok(Some(2.50)),         Ok(Some(2.50))),
    ];
    let flow = FlowContext::new();
    for (step, name, val, target_crane, target_winch1, target_winch2, target_winch3) in test_data.iter().cloned() {
        // let point = val.to_point(0, name);
        let ts = chrono::Utc::now();
        let point = match val {
            Value::Real(v) => Point::Real(PointHlr::new(0, name, v, Status::Ok, Cot::Inf, ts)),
            Value::Double(v) => Point::Double(PointHlr::new(0, name, v, Status::Ok, Cot::Inf, ts)),
            Value::Int(v) => Point::Int(PointHlr::new(0, name, v, Status::Ok, Cot::Inf, ts)),
            Value::Bool(v) => Point::Bool(PointHlr::new(0, name, Bool(v), Status::Ok, Cot::Inf, ts)),
            Value::String(v) => Point::String(PointHlr::new(0, name, v, Status::Ok, Cot::Inf, ts)),
            _ => panic!("{dbg} | Step {step} | '{name}': Invalit type 'Bytes'"),
        };
        log::debug!("{dbg} | Step {step} | point: {:?}", point);
        task_nodes.eval(point);
        // Извлекаем целевые узлы напрямую из графа по именам переменных (let)
        // (Если метод vars() скрыт, сделай его pub(crate) или добавь getter get_var(&str) -> Option<FnInOutRef>)
        let crane_node = task_nodes.get_var("craneLoadRelative").expect("Variable 'craneLoadRelative' not found in DAG");
        let winch1_node = task_nodes.get_var("winch1LoadRelative").expect("Variable 'winch1LoadRelative' not found in DAG");
        let winch2_node = task_nodes.get_var("winch2LoadRelative").expect("Variable 'winch2LoadRelative' not found in DAG");
        let winch3_node = task_nodes.get_var("winch3LoadRelative").expect("Variable 'winch3LoadRelative' not found in DAG");
        // Проверяем Кран
        let result = flow.ignore(crane_node.borrow_mut().out());
        match (&result, &target_crane) {
            (Ok(Some(result)), Ok(Some(target))) => {
                log::debug!("{dbg} | Step {step} | craneLoadRelative: {:?}", result.value());
                let actual = result.value().as_real();
                assert!((actual - target).abs() < f32::EPSILON, "{dbg} | Step {step} | craneLoadRelative \n result: {actual} \n target: {:?}", target);
            }
            (Ok(None), Ok(None)) | (Err(_), Err(_)) => {}
            _ => panic!("{dbg} | Step {step} | \n result: {:?} \n target: {:?}", result, target_crane),
        }
        // Проверяем Лебедку 1
        let result = flow.ignore(winch1_node.borrow_mut().out());
        match (&result, &target_winch1) {
            (Ok(Some(result)), Ok(Some(target))) => {
                log::debug!("{dbg} | Step {step} | winch1LoadRelative: {:?}", result.value());
                let actual = result.value().as_real();
                assert!((actual - target).abs() < f32::EPSILON, "{dbg} | Step {step} | winch1LoadRelative \n result: {actual} \n target: {:?}", target);
            }
            (Ok(None), Ok(None)) | (Err(_), Err(_)) => {}
            _ => panic!("{dbg} | Step {step} | \n result: {:?} \n target: {:?}", result, target_winch1),
        }
        // Проверяем Лебедку 2
        let result = flow.ignore(winch2_node.borrow_mut().out());
        match (&result, &target_winch2) {
            (Ok(Some(result)), Ok(Some(target))) => {
                log::debug!("{dbg} | Step {step} | winch2LoadRelative: {:?}", result.value());
                let actual = result.value().as_real();
                assert!((actual - target).abs() < f32::EPSILON, "{dbg} | Step {step} | winch2LoadRelative \n result: {actual} \n target: {:?}", target);
            }
            (Ok(None), Ok(None)) | (Err(_), Err(_)) => {}
            _ => panic!("{dbg} | Step {step} | \n result: {:?} \n target: {:?}", result, target_winch2),
        }
        // Проверяем Лебедку 3
        let result = flow.ignore(winch3_node.borrow_mut().out());
        match (&result, &target_winch3) {
            (Ok(Some(result)), Ok(Some(target))) => {
                log::debug!("{dbg} | Step {step} | winch3LoadRelative: {:?}", result.value());
                let actual = result.value().as_real();
                assert!((actual - target).abs() < f32::EPSILON, "{dbg} | Step {step} | winch3LoadRelative \n result: {actual} \n target: {:?}", target);
            }
            (Ok(None), Ok(None)) | (Err(_), Err(_)) => {}
            _ => panic!("{dbg} | Step {step} | \n result: {:?} \n target: {:?}", result, target_winch3),
        }
    }
}
///
/// Динамический расчет порога 5% от номинала для определения начала рабочего цикла.
#[test]
fn calc_op_cycle_threshold() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    init_each();
    let dbg = "calc_op_cycle_threshold";
    log::debug!("{dbg}");
    let self_name = Name::new("", dbg);
    let mut task_nodes = TaskNodes::without_retain(dbg, 0);
    let conf = TaskConf::read(&self_name, "src/tests/unit/services/task/cma_recorder/cma-recorder.yaml").unwrap();
    let services = Arc::new(Services::new(dbg, ServicesConf::new(dbg, ConfTree::empty()), None).unwrap());
    task_nodes.build_nodes(&Name::from(dbg), &conf, services).unwrap();
    let test_data: &[(i32, &str, Value, Result<Option<f32>, ()>, Result<Option<f32>, ()>, Result<Option<f32>, ()>, Result<Option<f32>, ()>)] = &[
        // step  Входной сигнал (Событие)                     Значение                      crane                   winch1                  winch2                  winch3
        // Задаем LoadR0 для Winch1 (он же nominal для Crane). 5% от 100.0 = 5.0
        (01,      "/App/ied13/db905_visual_data_fast/Winch1.LoadR0", Value::Real(100.0),    Ok(Some(5.0)),          Ok(Some(5.0)),          Ok(None),               Ok(None)),
        // Задаем LoadR0 для Winch2. 5% от 80.0 = 4.0
        (02,      "/App/ied13/db905_visual_data_fast/Winch2.LoadR0", Value::Real(80.0),     Ok(Some(5.0)),          Ok(Some(5.0)),          Ok(Some(4.0)),          Ok(None)),
        // Задаем LoadR0 для Winch3. 5% от 70.0 = 3.5
        (03,      "/App/ied13/db905_visual_data_fast/Winch3.LoadR0", Value::Real(70.0),     Ok(Some(5.0)),          Ok(Some(5.0)),          Ok(Some(4.0)),          Ok(Some(3.5))),
        // Динамическое изменение: кран перешел в другой режим работы (сменилась запасовка/стрела), номинал упал до 50 тонн. 5% от 50.0 = 2.5
        (04,      "/App/ied13/db905_visual_data_fast/Winch1.LoadR0", Value::Real(50.0),     Ok(Some(2.5)),          Ok(Some(2.5)),          Ok(Some(4.0)),          Ok(Some(3.5))),
        // Кран ушел в ошибку (датчик отвалился), ПЛК прислал 0.0. Порог должен лечь в 0.0
        (05,      "/App/ied13/db905_visual_data_fast/Winch1.LoadR0", Value::Real(0.0),      Ok(Some(0.0)),          Ok(Some(0.0)),          Ok(Some(4.0)),          Ok(Some(3.5))),
    ];
    let flow = FlowContext::new();
    for (step, name, val, target_crane, target_winch1, target_winch2, target_winch3) in test_data.iter().cloned() {
        let ts = chrono::Utc::now();
        let point = match val {
            Value::Real(v) => Point::Real(PointHlr::new(0, name, v, Status::Ok, Cot::Inf, ts)),
            Value::Double(v) => Point::Double(PointHlr::new(0, name, v, Status::Ok, Cot::Inf, ts)),
            Value::Int(v) => Point::Int(PointHlr::new(0, name, v, Status::Ok, Cot::Inf, ts)),
            Value::Bool(v) => Point::Bool(PointHlr::new(0, name, Bool(v), Status::Ok, Cot::Inf, ts)),
            Value::String(v) => Point::String(PointHlr::new(0, name, v, Status::Ok, Cot::Inf, ts)),
            _ => panic!("{dbg} | Step {step} | '{name}': Invalid type"),
        };
        log::debug!("{dbg} | Step {step} | point: {:?}", point);
        task_nodes.eval(point);
        let crane_node = task_nodes.get_var("opCycleThreshold").expect("Variable 'opCycleThreshold' not found in DAG");
        let winch1_node = task_nodes.get_var("winch1OpCycleThreshold").expect("Variable 'winch1OpCycleThreshold' not found in DAG");
        let winch2_node = task_nodes.get_var("winch2OpCycleThreshold").expect("Variable 'winch2OpCycleThreshold' not found in DAG");
        let winch3_node = task_nodes.get_var("winch3OpCycleThreshold").expect("Variable 'winch3OpCycleThreshold' not found in DAG");
        // Проверяем порог Крана
        let result = flow.ignore(crane_node.borrow_mut().out());
        match (&result, &target_crane) {
            (Ok(Some(result)), Ok(Some(target))) => {
                log::debug!("{dbg} | Step {step} | crane threshold: {:?}", result.value());
                let actual = result.value().as_real();
                assert!((actual - target).abs() < f32::EPSILON, "{dbg} | Step {step} | crane threshold \n result: {actual} \n target: {:?}", target);
            }
            (Ok(None), Ok(None)) | (Err(_), Err(_)) => {}
            _ => panic!("{dbg} | Step {step} | \n result: {:?} \n target: {:?}", result, target_crane),
        }
        // Проверяем порог Лебедки 1
        let result = flow.ignore(winch1_node.borrow_mut().out());
        match (&result, &target_winch1) {
            (Ok(Some(result)), Ok(Some(target))) => {
                log::debug!("{dbg} | Step {step} | winch1 threshold: {:?}", result.value());
                let actual = result.value().as_real();
                assert!((actual - target).abs() < f32::EPSILON, "{dbg} | Step {step} | winch1 threshold \n result: {actual} \n target: {:?}", target);
            }
            (Ok(None), Ok(None)) | (Err(_), Err(_)) => {}
            _ => panic!("{dbg} | Step {step} | \n result: {:?} \n target: {:?}", result, target_winch1),
        }
        // Проверяем порог Лебедки 2
        let result = flow.ignore(winch2_node.borrow_mut().out());
        match (&result, &target_winch2) {
            (Ok(Some(result)), Ok(Some(target))) => {
                log::debug!("{dbg} | Step {step} | winch2 threshold: {:?}", result.value());
                let actual = result.value().as_real();
                assert!((actual - target).abs() < f32::EPSILON, "{dbg} | Step {step} | winch2 threshold \n result: {actual} \n target: {:?}", target);
            }
            (Ok(None), Ok(None)) | (Err(_), Err(_)) => {}
            _ => panic!("{dbg} | Step {step} | \n result: {:?} \n target: {:?}", result, target_winch2),
        }
        // Проверяем порог Лебедки 3
        let result = flow.ignore(winch3_node.borrow_mut().out());
        match (&result, &target_winch3) {
            (Ok(Some(result)), Ok(Some(target))) => {
                log::debug!("{dbg} | Step {step} | winch3 threshold: {:?}", result.value());
                let actual = result.value().as_real();
                assert!((actual - target).abs() < f32::EPSILON, "{dbg} | Step {step} | winch3 threshold \n result: {actual} \n target: {:?}", target);
            }
            (Ok(None), Ok(None)) | (Err(_), Err(_)) => {}
            _ => panic!("{dbg} | Step {step} | \n result: {:?} \n target: {:?}", result, target_winch3),
        }
    }
}
///
/// Фиксация активности крана с учетом гистерезиса нагрузки 5% (FnThreshold) и таймеров задержки (TimerOnDelay, TimerOffDelay 5000ms).
#[test]
fn detect_crane_is_active() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    init_each();
    let dbg = "detect_crane_is_active";
    log::debug!("{dbg}");
    let self_name = Name::new("", dbg);
    let mut task_nodes = TaskNodes::without_retain(dbg, 0);
    let conf = TaskConf::read(&self_name, "src/tests/unit/services/task/cma_recorder/cma-recorder.yaml").unwrap();
    let services = Arc::new(Services::new(dbg, ServicesConf::new(dbg, ConfTree::empty()), None).unwrap());
    task_nodes.build_nodes(&Name::from(dbg), &conf, services).unwrap();
    // Кортеж: (Шаг, Задержка_мс, Имя_сигнала, Значение, Ожидаемый_craneIsActive)
    let test_data: &[(i32, u64, &str, Value, Result<Option<bool>, ()>)] = &[
        // Инициализация номинала (LoadR0 = 100.0). Динамический порог активности (5%) = 5.0
        (1, 0,    "/App/ied13/db905_visual_data_fast/Winch1.LoadR0", Value::Real(100.0), Ok(None)),
        // Нагрузка 4.0 (< 5.0). Фильтр FnThreshold не пропускает, компаратор = false. Таймеры молчат.
        (2, 0,    "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(4.0),   Ok(Some(false))),
        // Нагрузка 10.0 (> 5.0). FnThreshold пропускает, FnGe = true. Запускается TimerOnDelay (5000ms).
        (3, 0,    "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(10.0),  Ok(Some(false))),
        // Ждем 2.5 секунды. Имитируем приход того же веса (шум датчика). Таймер включения еще идет.
        (4, 2500, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(10.0),  Ok(Some(false))),
        // Ждем еще 3 секунды (всего 5.5). TimerOnDelay пробивается и выдает true. Рабочий цикл начат!
        (5, 3000, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(11.0),  Ok(Some(true))),
        // Сброс нагрузки до 2.0 (упала ниже порога). FnGe = false. Запускается TimerOffDelay (5000ms).
        (6, 0,    "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(2.0),   Ok(Some(true))),
        // Прошло 2 секунды. Крюк пустой. Таймер выключения еще держит флаг цикла.
        (7, 2000, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(2.0),   Ok(Some(true))),
        // Прошло еще 3.5 секунды (всего 5.5). TimerOffDelay истекает и отпускает флаг. Цикл завершен!
        (8, 3500, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(2.0),   Ok(Some(false))),
    ];
    let flow = FlowContext::new();
    for (step, delay_ms, name, val, target_active) in test_data.iter().cloned() {
        if delay_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        }
        let ts = chrono::Utc::now();
        let point = match val {
            Value::Real(v) => Point::Real(PointHlr::new(0, name, v, Status::Ok, Cot::Inf, ts)),
            Value::Double(v) => Point::Double(PointHlr::new(0, name, v, Status::Ok, Cot::Inf, ts)),
            Value::Int(v) => Point::Int(PointHlr::new(0, name, v, Status::Ok, Cot::Inf, ts)),
            Value::Bool(v) => Point::Bool(PointHlr::new(0, name, Bool(v), Status::Ok, Cot::Inf, ts)),
            Value::String(v) => Point::String(PointHlr::new(0, name, v, Status::Ok, Cot::Inf, ts)),
            _ => panic!("{dbg} | Step {step} | '{name}': Invalid type"),
        };
        log::debug!("{dbg} | Step {step} | point: {:?}", point);
        task_nodes.eval(point);
        let active_node = task_nodes.get_var("craneIsActive").expect("Variable 'craneIsActive' not found in DAG");
        let result = flow.ignore(active_node.borrow_mut().out());
        log::debug!("{dbg} | Step {step} | craneIsActive: {:?}", result);
        match (&result, &target_active) {
            (Ok(Some(result)), Ok(Some(target))) => {
                log::debug!("{dbg} | Step {step} | craneIsActive: {:?}", result.value());
                let actual = result.as_bool().value.0;
                assert_eq!(actual, *target, "{dbg} | Step {step} | craneIsActive \n result: {actual} \n target: {target}");
            }
            (Ok(None), Ok(None)) | (Err(_), Err(_)) => {}
            _ => panic!("{dbg} | Step {step} | \n result: {:?} \n target: {:?}", result, target_active),
        }
    }
}
///
/// Фиксация активности крана с учетом гистерезиса нагрузки 5% (FnThreshold) и таймеров задержки (TimerOnDelay, TimerOffDelay 5000ms).
#[test]
fn detect_winch1_is_active() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    init_each();
    let dbg = "detect_winch1_is_active";
    log::debug!("{dbg}");
    let self_name = Name::new("", dbg);
    let mut task_nodes = TaskNodes::without_retain(dbg, 0);
    let conf = TaskConf::read(&self_name, "src/tests/unit/services/task/cma_recorder/cma-recorder.yaml").unwrap();
    let services = Arc::new(Services::new(dbg, ServicesConf::new(dbg, ConfTree::empty()), None).unwrap());
    task_nodes.build_nodes(&Name::from(dbg), &conf, services).unwrap();
    // Кортеж: (Шаг, Задержка_мс, Имя_сигнала, Значение, Ожидаемый_craneIsActive)
    let test_data: &[(i32, u64, &str, Value, Result<Option<bool>, ()>)] = &[
        // Инициализация номинала (LoadR0 = 100.0). Динамический порог активности (5%) = 5.0
        (1, 0,    "/App/ied13/db905_visual_data_fast/Winch1.LoadR0", Value::Real(100.0), Ok(None)),
        // Нагрузка 4.0 (< 5.0). Фильтр FnThreshold не пропускает, компаратор = false. Таймеры молчат.
        (2, 0,    "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(4.0),   Ok(Some(false))),
        // Нагрузка 10.0 (> 5.0). FnThreshold пропускает, FnGe = true. Запускается TimerOnDelay (5000ms).
        (3, 0,    "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(10.0),  Ok(Some(false))),
        // Ждем 2.5 секунды. Имитируем приход того же веса (шум датчика). Таймер включения еще идет.
        (4, 2500, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(10.0),  Ok(Some(false))),
        // Ждем еще 3 секунды (всего 5.5). TimerOnDelay пробивается и выдает true. Рабочий цикл начат!
        (5, 3000, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(11.0),  Ok(Some(true))),
        // Сброс нагрузки до 2.0 (упала ниже порога). FnGe = false. Запускается TimerOffDelay (5000ms).
        (6, 0,    "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(2.0),   Ok(Some(true))),
        // Прошло 2 секунды. Крюк пустой. Таймер выключения еще держит флаг цикла.
        (7, 2000, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(2.0),   Ok(Some(true))),
        // Прошло еще 3.5 секунды (всего 5.5). TimerOffDelay истекает и отпускает флаг. Цикл завершен!
        (8, 3500, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(2.0),   Ok(Some(false))),
    ];
    let flow = FlowContext::new();
    for (step, delay_ms, name, val, target_active) in test_data.iter().cloned() {
        if delay_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        }
        let ts = chrono::Utc::now();
        let point = match val {
            Value::Real(v) => Point::Real(PointHlr::new(0, name, v, Status::Ok, Cot::Inf, ts)),
            Value::Double(v) => Point::Double(PointHlr::new(0, name, v, Status::Ok, Cot::Inf, ts)),
            Value::Int(v) => Point::Int(PointHlr::new(0, name, v, Status::Ok, Cot::Inf, ts)),
            Value::Bool(v) => Point::Bool(PointHlr::new(0, name, Bool(v), Status::Ok, Cot::Inf, ts)),
            Value::String(v) => Point::String(PointHlr::new(0, name, v, Status::Ok, Cot::Inf, ts)),
            _ => panic!("{dbg} | Step {step} | '{name}': Invalid type"),
        };
        log::debug!("{dbg} | Step {step} | point: {:?}", point);
        task_nodes.eval(point);
        let active_node = task_nodes.get_var("winch1IsActive").expect("Variable 'winch1IsActive' not found in DAG");
        let result = flow.ignore(active_node.borrow_mut().out());
        log::debug!("{dbg} | Step {step} | winch1IsActive: {:?}", result);
        match (&result, &target_active) {
            (Ok(Some(result)), Ok(Some(target))) => {
                log::debug!("{dbg} | Step {step} | winch1IsActive: {:?}", result.value());
                let actual = result.as_bool().value.0;
                assert_eq!(actual, *target, "{dbg} | Step {step} | winch1IsActive \n result: {actual} \n target: {target}");
            }
            (Ok(None), Ok(None)) | (Err(_), Err(_)) => {}
            _ => panic!("{dbg} | Step {step} | \n result: {:?} \n target: {:?}", result, target_active),
        }
    }
}
///
/// Фиксация активности крана с учетом гистерезиса нагрузки 5% (FnThreshold) и таймеров задержки (TimerOnDelay, TimerOffDelay 5000ms).
#[test]
fn detect_winch2_is_active() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    init_each();
    let dbg = "detect_winch2_is_active";
    log::debug!("{dbg}");
    let self_name = Name::new("", dbg);
    let mut task_nodes = TaskNodes::without_retain(dbg, 0);
    let conf = TaskConf::read(&self_name, "src/tests/unit/services/task/cma_recorder/cma-recorder.yaml").unwrap();
    let services = Arc::new(Services::new(dbg, ServicesConf::new(dbg, ConfTree::empty()), None).unwrap());
    task_nodes.build_nodes(&Name::from(dbg), &conf, services).unwrap();
    // Кортеж: (Шаг, Задержка_мс, Имя_сигнала, Значение, Ожидаемый_craneIsActive)
    let test_data: &[(i32, u64, &str, Value, Result<Option<bool>, ()>)] = &[
        // Инициализация номинала (LoadR0 = 100.0). Динамический порог активности (5%) = 5.0
        (1, 0,    "/App/ied13/db905_visual_data_fast/Winch2.LoadR0", Value::Real(100.0), Ok(None)),
        // Нагрузка 4.0 (< 5.0). Фильтр FnThreshold не пропускает, компаратор = false. Таймеры молчат.
        (2, 0,    "/App/ied14/db906_visual_data/Winch2.Load",        Value::Real(4.0),   Ok(Some(false))),
        // Нагрузка 10.0 (> 5.0). FnThreshold пропускает, FnGe = true. Запускается TimerOnDelay (5000ms).
        (3, 0,    "/App/ied14/db906_visual_data/Winch2.Load",        Value::Real(10.0),  Ok(Some(false))),
        // Ждем 2.5 секунды. Имитируем приход того же веса (шум датчика). Таймер включения еще идет.
        (4, 2500, "/App/ied14/db906_visual_data/Winch2.Load",        Value::Real(10.0),  Ok(Some(false))),
        // Ждем еще 3 секунды (всего 5.5). TimerOnDelay пробивается и выдает true. Рабочий цикл начат!
        (5, 3000, "/App/ied14/db906_visual_data/Winch2.Load",        Value::Real(11.0),  Ok(Some(true))),
        // Сброс нагрузки до 2.0 (упала ниже порога). FnGe = false. Запускается TimerOffDelay (5000ms).
        (6, 0,    "/App/ied14/db906_visual_data/Winch2.Load",        Value::Real(2.0),   Ok(Some(true))),
        // Прошло 2 секунды. Крюк пустой. Таймер выключения еще держит флаг цикла.
        (7, 2000, "/App/ied14/db906_visual_data/Winch2.Load",        Value::Real(2.0),   Ok(Some(true))),
        // Прошло еще 3.5 секунды (всего 5.5). TimerOffDelay истекает и отпускает флаг. Цикл завершен!
        (8, 3500, "/App/ied14/db906_visual_data/Winch2.Load",        Value::Real(2.0),   Ok(Some(false))),
    ];
    let flow = FlowContext::new();
    for (step, delay_ms, name, val, target_active) in test_data.iter().cloned() {
        if delay_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        }
        let ts = chrono::Utc::now();
        let point = match val {
            Value::Real(v) => Point::Real(PointHlr::new(0, name, v, Status::Ok, Cot::Inf, ts)),
            Value::Double(v) => Point::Double(PointHlr::new(0, name, v, Status::Ok, Cot::Inf, ts)),
            Value::Int(v) => Point::Int(PointHlr::new(0, name, v, Status::Ok, Cot::Inf, ts)),
            Value::Bool(v) => Point::Bool(PointHlr::new(0, name, Bool(v), Status::Ok, Cot::Inf, ts)),
            Value::String(v) => Point::String(PointHlr::new(0, name, v, Status::Ok, Cot::Inf, ts)),
            _ => panic!("{dbg} | Step {step} | '{name}': Invalid type"),
        };
        log::debug!("{dbg} | Step {step} | point: {:?}", point);
        task_nodes.eval(point);
        let active_node = task_nodes.get_var("winch2IsActive").expect("Variable 'winch2IsActive' not found in DAG");
        let result = flow.ignore(active_node.borrow_mut().out());
        log::debug!("{dbg} | Step {step} | winch2IsActive: {:?}", result);
        match (&result, &target_active) {
            (Ok(Some(result)), Ok(Some(target))) => {
                log::debug!("{dbg} | Step {step} | winch2IsActive: {:?}", result.value());
                let actual = result.as_bool().value.0;
                assert_eq!(actual, *target, "{dbg} | Step {step} | winch2IsActive \n result: {actual} \n target: {target}");
            }
            (Ok(None), Ok(None)) | (Err(_), Err(_)) => {}
            _ => panic!("{dbg} | Step {step} | \n result: {:?} \n target: {:?}", result, target_active),
        }
    }
}
///
/// Фиксация активности крана с учетом гистерезиса нагрузки 5% (FnThreshold) и таймеров задержки (TimerOnDelay, TimerOffDelay 5000ms).
#[test]
fn detect_winch3_is_active() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    init_each();
    let dbg = "detect_winch3_is_active";
    log::debug!("{dbg}");
    let self_name = Name::new("", dbg);
    let mut task_nodes = TaskNodes::without_retain(dbg, 0);
    let conf = TaskConf::read(&self_name, "src/tests/unit/services/task/cma_recorder/cma-recorder.yaml").unwrap();
    let services = Arc::new(Services::new(dbg, ServicesConf::new(dbg, ConfTree::empty()), None).unwrap());
    task_nodes.build_nodes(&Name::from(dbg), &conf, services).unwrap();
    // Кортеж: (Шаг, Задержка_мс, Имя_сигнала, Значение, Ожидаемый_craneIsActive)
    let test_data: &[(i32, u64, &str, Value, Result<Option<bool>, ()>)] = &[
        // Инициализация номинала (LoadR0 = 100.0). Динамический порог активности (5%) = 5.0
        (1, 0,    "/App/ied13/db905_visual_data_fast/Winch3.LoadR0", Value::Real(100.0), Ok(None)),
        // Нагрузка 4.0 (< 5.0). Фильтр FnThreshold не пропускает, компаратор = false. Таймеры молчат.
        (2, 0,    "/App/ied14/db906_visual_data/Winch3.Load",        Value::Real(4.0),   Ok(Some(false))),
        // Нагрузка 10.0 (> 5.0). FnThreshold пропускает, FnGe = true. Запускается TimerOnDelay (5000ms).
        (3, 0,    "/App/ied14/db906_visual_data/Winch3.Load",        Value::Real(10.0),  Ok(Some(false))),
        // Ждем 2.5 секунды. Имитируем приход того же веса (шум датчика). Таймер включения еще идет.
        (4, 2500, "/App/ied14/db906_visual_data/Winch3.Load",        Value::Real(10.0),  Ok(Some(false))),
        // Ждем еще 3 секунды (всего 5.5). TimerOnDelay пробивается и выдает true. Рабочий цикл начат!
        (5, 3000, "/App/ied14/db906_visual_data/Winch3.Load",        Value::Real(11.0),  Ok(Some(true))),
        // Сброс нагрузки до 2.0 (упала ниже порога). FnGe = false. Запускается TimerOffDelay (5000ms).
        (6, 0,    "/App/ied14/db906_visual_data/Winch3.Load",        Value::Real(2.0),   Ok(Some(true))),
        // Прошло 2 секунды. Крюк пустой. Таймер выключения еще держит флаг цикла.
        (7, 2000, "/App/ied14/db906_visual_data/Winch3.Load",        Value::Real(2.0),   Ok(Some(true))),
        // Прошло еще 3.5 секунды (всего 5.5). TimerOffDelay истекает и отпускает флаг. Цикл завершен!
        (8, 3500, "/App/ied14/db906_visual_data/Winch3.Load",        Value::Real(2.0),   Ok(Some(false))),
    ];
    let flow = FlowContext::new();
    for (step, delay_ms, name, val, target_active) in test_data.iter().cloned() {
        if delay_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        }
        let ts = chrono::Utc::now();
        let point = match val {
            Value::Real(v) => Point::Real(PointHlr::new(0, name, v, Status::Ok, Cot::Inf, ts)),
            Value::Double(v) => Point::Double(PointHlr::new(0, name, v, Status::Ok, Cot::Inf, ts)),
            Value::Int(v) => Point::Int(PointHlr::new(0, name, v, Status::Ok, Cot::Inf, ts)),
            Value::Bool(v) => Point::Bool(PointHlr::new(0, name, Bool(v), Status::Ok, Cot::Inf, ts)),
            Value::String(v) => Point::String(PointHlr::new(0, name, v, Status::Ok, Cot::Inf, ts)),
            _ => panic!("{dbg} | Step {step} | '{name}': Invalid type"),
        };
        log::debug!("{dbg} | Step {step} | point: {:?}", point);
        task_nodes.eval(point);
        let active_node = task_nodes.get_var("winch3IsActive").expect("Variable 'winch3IsActive' not found in DAG");
        let result = flow.ignore(active_node.borrow_mut().out());
        log::debug!("{dbg} | Step {step} | winch3IsActive: {:?}", result);
        match (&result, &target_active) {
            (Ok(Some(result)), Ok(Some(target))) => {
                log::debug!("{dbg} | Step {step} | winch3IsActive: {:?}", result.value());
                let actual = result.as_bool().value.0;
                assert_eq!(actual, *target, "{dbg} | Step {step} | winch3IsActive \n result: {actual} \n target: {target}");
            }
            (Ok(None), Ok(None)) | (Err(_), Err(_)) => {}
            _ => panic!("{dbg} | Step {step} | \n result: {:?} \n target: {:?}", result, target_active),
        }
    }
}
