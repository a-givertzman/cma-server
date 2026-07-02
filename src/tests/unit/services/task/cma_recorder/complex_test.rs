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
        (01,      "/App/ied13/db905_visual_data_fast/Winch1.LoadR0", Value::Real(100.0), Ok(Some(0.0)),         Ok(Some(0.0)),          Ok(None),               Ok(None)),
        // Поднимаем 25 тонн -> Относительная = 25% (0.25)
        (02,      "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(25.0),  Ok(Some(0.25)),        Ok(Some(0.25)),         Ok(None),               Ok(None)),
        // Поднимаем 125 тонн -> Относительная = 125% (1.25)
        (03,      "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(125.0), Ok(Some(1.25)),        Ok(Some(1.25)),         Ok(None),               Ok(None)),
        // Меняется номинал. Вес висит тот же (125.0). Относительная нагрузка должна прыгнуть до 250% (2.5)
        (04,      "/App/ied13/db905_visual_data_fast/Winch1.LoadR0", Value::Real(50.0),  Ok(Some(2.50)),        Ok(Some(2.50)),         Ok(None),               Ok(None)),
        // - Winch 2
        // Устанавливаем номинальную нагрузку (Делитель = 80.0) -> Относительная = 0.0 (так как текущий вес 0.0)
        (05,      "/App/ied13/db905_visual_data_fast/Winch2.LoadR0", Value::Real(80.0),  Ok(Some(2.50)),        Ok(Some(2.50)),         Ok(Some(0.0)),          Ok(None)),
        // Поднимаем 20 тонн -> Относительная = 25% (0.25)
        (06,      "/App/ied14/db906_visual_data/Winch2.Load",        Value::Real(20.0),  Ok(Some(2.50)),        Ok(Some(2.50)),         Ok(Some(0.25)),         Ok(None)),
        // Поднимаем 100 тонн -> Относительная = 125% (1.25)
        (07,      "/App/ied14/db906_visual_data/Winch2.Load",        Value::Real(100.0), Ok(Some(2.50)),        Ok(Some(2.50)),         Ok(Some(1.25)),         Ok(None)),
        // Меняется номинал. Вес висит тот же (125.0). Относительная нагрузка должна прыгнуть до 250% (2.5)
        (08,      "/App/ied13/db905_visual_data_fast/Winch2.LoadR0", Value::Real(40.0),  Ok(Some(2.50)),        Ok(Some(2.50)),         Ok(Some(2.50)),         Ok(None)),
        // - Winch 3
        // Устанавливаем номинальную нагрузку (Делитель = 70.0) -> Относительная = 0.0 (так как текущий вес 0.0)
        (10,      "/App/ied13/db905_visual_data_fast/Winch3.LoadR0", Value::Real(70.0),  Ok(Some(2.50)),        Ok(Some(2.50)),         Ok(Some(2.50)),         Ok(Some(0.0))),
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
        (1, 0,    "/App/ied13/db905_visual_data_fast/Winch1.LoadR0", Value::Real(100.0), Ok(Some(false))),
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
        (1, 0,    "/App/ied13/db905_visual_data_fast/Winch1.LoadR0", Value::Real(100.0), Ok(Some(false))),
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
        (1, 0,    "/App/ied13/db905_visual_data_fast/Winch2.LoadR0", Value::Real(100.0), Ok(Some(false))),
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
        (1, 0,    "/App/ied13/db905_visual_data_fast/Winch3.LoadR0", Value::Real(100.0), Ok(Some(false))),
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
///
/// Проверка совпадения активности гидростанции с активностью рабочего цикла.
#[test]
fn detect_op_cycle_active() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    init_each();
    let dbg = "detect_op_cycle_active";
    log::debug!("{dbg}");
    let self_name = Name::new("", dbg);
    let mut task_nodes = TaskNodes::without_retain(dbg, 0);
    let conf = TaskConf::read(&self_name, "src/tests/unit/services/task/cma_recorder/cma-recorder.yaml").unwrap();
    let services = Arc::new(Services::new(dbg, ServicesConf::new(dbg, ConfTree::empty()), None).unwrap());
    task_nodes.build_nodes(&Name::from(dbg), &conf, services).unwrap();
    // Кортеж: (Шаг, Задержка_мс, Имя_сигнала, Значение, Ожидаемый_pumpIsActive)
    let test_data: &[(i32, u64, &str, Value, Result<Option<bool>, ()>)] = &[
        // Инициализация номиналов. Порог активности (5%) = 5.0 для обеих лебедок.
        (01, 0,    "/App/ied13/db905_visual_data_fast/Winch1.LoadR0", Value::Real(100.0), Ok(None)),
        (02, 0,    "/App/ied13/db905_visual_data_fast/Winch2.LoadR0", Value::Real(100.0), Ok(None)),
        (03, 0,    "/App/ied13/db905_visual_data_fast/Winch3.LoadR0", Value::Real(100.0), Ok(Some(false))),
        // Поднимаем груз на Лебедке 1 (10.0 > 5.0). Запуск OnDelay таймера.
        (04, 0,    "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(10.0),  Ok(Some(false))),
        // Прошло 5.5с. Лебедка 1 перешла в Active -> Насос ВКЛ.
        (05, 5500, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(10.0),  Ok(Some(true))),
        // Поднимаем груз на Лебедке 2 (20.0 > 5.0). Запуск OnDelay для нее. Насос уже работает.
        (06, 0,    "/App/ied14/db906_visual_data/Winch2.Load",        Value::Real(20.0),  Ok(Some(true))),
        // Прошло 5.5с. Лебедка 2 тоже Active. Насос продолжает работу (OR).
        (07, 5500, "/App/ied14/db906_visual_data/Winch2.Load",        Value::Real(20.0),  Ok(Some(true))),
        // Сбрасываем Лебедку 1 (0.0). Запуск OffDelay таймера.
        (08, 0,    "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(0.0),   Ok(Some(true))),
        // Прошло 5.5с. Лебедка 1 отключилась. Но Лебедка 2 всё еще Active -> Насос не останавливается.
        (09, 5500, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(0.0),   Ok(Some(true))),
        // Сбрасываем Лебедку 2 (0.0). Запуск OffDelay.
        (10, 0,    "/App/ied14/db906_visual_data/Winch2.Load",        Value::Real(0.0),   Ok(Some(true))),
        // Прошло 5.5с. Лебедка 2 отключилась. Все механизмы стоят -> Насос ВЫКЛ.
        (11, 5500, "/App/ied14/db906_visual_data/Winch2.Load",       Value::Real(0.0),   Ok(Some(false))),
    ];
    let flow = FlowContext::new();
    for (step, delay_ms, name, val, target_pump) in test_data.iter().cloned() {
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
        let pump_node = task_nodes.get_var("pumpIsActive").expect("Variable 'pumpIsActive' not found in DAG");
        let result = flow.ignore(pump_node.borrow_mut().out());
        match (&result, &target_pump) {
            (Ok(Some(result)), Ok(Some(target))) => {
                log::debug!("{dbg} | Step {step} | pumpIsActive: {:?}", result.value());
                let actual = result.as_bool().value.0;
                assert_eq!(actual, *target, "{dbg} | Step {step} | pumpIsActive \n result: {actual} \n target: {target}");
            }
            (Ok(None), Ok(None)) | (Err(_), Err(_)) => {}
            _ => panic!("{dbg} | Step {step} | \n result: {:?} \n target: {:?}", result, target_pump),
        }
    }
}
///
/// Проверка совпадения активности гидростанции с активностью рабочего цикла.
#[test]
fn detect_pump_is_active() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    init_each();
    let dbg = "detect_pump_is_active";
    log::debug!("{dbg}");
    let self_name = Name::new("", dbg);
    let mut task_nodes = TaskNodes::without_retain(dbg, 0);
    let conf = TaskConf::read(&self_name, "src/tests/unit/services/task/cma_recorder/cma-recorder.yaml").unwrap();
    let services = Arc::new(Services::new(dbg, ServicesConf::new(dbg, ConfTree::empty()), None).unwrap());
    task_nodes.build_nodes(&Name::from(dbg), &conf, services).unwrap();
    // Кортеж: (Шаг, Задержка_мс, Имя_сигнала, Значение, Ожидаемый_pumpIsActive)
    let test_data: &[(i32, u64, &str, Value, Result<Option<bool>, ()>)] = &[
        // Инициализация номиналов. Порог активности (5%) = 5.0 для обеих лебедок.
        (01, 0,    "/App/ied13/db905_visual_data_fast/Winch1.LoadR0", Value::Real(100.0), Ok(None)),
        (02, 0,    "/App/ied13/db905_visual_data_fast/Winch2.LoadR0", Value::Real(100.0), Ok(None)),
        (03, 0,    "/App/ied13/db905_visual_data_fast/Winch3.LoadR0", Value::Real(100.0), Ok(Some(false))),
        // Поднимаем груз на Лебедке 1 (10.0 > 5.0). Запуск OnDelay таймера.
        (03, 0,    "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(10.0),  Ok(Some(false))),
        // Прошло 5.5с. Лебедка 1 перешла в Active -> Насос ВКЛ.
        (04, 5500, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(10.0),  Ok(Some(true))),
        // Поднимаем груз на Лебедке 2 (20.0 > 5.0). Запуск OnDelay для нее. Насос уже работает.
        (05, 0,    "/App/ied14/db906_visual_data/Winch2.Load",        Value::Real(20.0),  Ok(Some(true))),
        // Прошло 5.5с. Лебедка 2 тоже Active. Насос продолжает работу (OR).
        (06, 5500, "/App/ied14/db906_visual_data/Winch2.Load",        Value::Real(20.0),  Ok(Some(true))),
        // Сбрасываем Лебедку 1 (0.0). Запуск OffDelay таймера.
        (07, 0,    "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(0.0),   Ok(Some(true))),
        // Прошло 5.5с. Лебедка 1 отключилась. Но Лебедка 2 всё еще Active -> Насос не останавливается.
        (08, 5500, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(0.0),   Ok(Some(true))),
        // Сбрасываем Лебедку 2 (0.0). Запуск OffDelay.
        (09, 0,    "/App/ied14/db906_visual_data/Winch2.Load",        Value::Real(0.0),   Ok(Some(true))),
        // Прошло 5.5с. Лебедка 2 отключилась. Все механизмы стоят -> Насос ВЫКЛ.
        (10, 5500, "/App/ied14/db906_visual_data/Winch2.Load",       Value::Real(0.0),   Ok(Some(false))),
    ];
    let flow = FlowContext::new();
    for (step, delay_ms, name, val, target_pump) in test_data.iter().cloned() {
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
        let pump_node = task_nodes.get_var("pumpIsActive").expect("Variable 'pumpIsActive' not found in DAG");
        let result = flow.ignore(pump_node.borrow_mut().out());
        match (&result, &target_pump) {
            (Ok(Some(result)), Ok(Some(target))) => {
                log::debug!("{dbg} | Step {step} | pumpIsActive: {:?}", result.value());
                let actual = result.as_bool().value.0;
                assert_eq!(actual, *target, "{dbg} | Step {step} | pumpIsActive \n result: {actual} \n target: {target}");
            }
            (Ok(None), Ok(None)) | (Err(_), Err(_)) => {}
            _ => panic!("{dbg} | Step {step} | \n result: {:?} \n target: {:?}", result, target_pump),
        }
    }
}
///
/// Выделение переднего (Started) и заднего (Done) фронтов рабочего цикла.
#[test]
fn detect_op_cycle_edges() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    init_each();
    let dbg = "detect_op_cycle_edges";
    log::debug!("{dbg}");
    let self_name = Name::new("", dbg);
    let mut task_nodes = TaskNodes::without_retain(dbg, 0);
    let conf = TaskConf::read(&self_name, "src/tests/unit/services/task/cma_recorder/cma-recorder.yaml").unwrap();
    let services = Arc::new(Services::new(dbg, ServicesConf::new(dbg, ConfTree::empty()), None).unwrap());
    task_nodes.build_nodes(&Name::from(dbg), &conf, services).unwrap();
    // Кортеж: (Шаг, Задержка_мс, Имя_сигнала,                      Значение,           target_started,     target_done)
    let test_data: &[(i32, u64, &str, Value, Result<Option<bool>, ()>, Result<Option<bool>, ()>)] = &[
        // Инициализация номинала Winch1 (LoadR0 = 100.0). Порог активности (5%) = 5.0
        (01, 0,    "/App/ied13/db905_visual_data_fast/Winch1.LoadR0", Value::Real(100.0), Ok(None), Ok(None)),
        (01, 0,    "/App/ied13/db905_visual_data_fast/Winch2.LoadR0", Value::Real(100.0), Ok(None), Ok(None)),
        (01, 0,    "/App/ied13/db905_visual_data_fast/Winch3.LoadR0", Value::Real(100.0), Ok(Some(false)), Ok(Some(false))),
        // Нагрузка 15.0 (> 5.0). TimerOnDelay (5000ms) запускается. Цикл еще не начат.
        (02, 0,    "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(15.0),  Ok(Some(false)), Ok(Some(false))),
        // Прошло 2 секунды. Нагрузка висит. Таймер в процессе.
        (03, 2000, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(15.0),  Ok(Some(false)), Ok(Some(false))),
        // Прошло еще 3.5 секунды (всего 5.5). TimerOnDelay пробивается -> opCycleIsActive = true.
        // Срабатывает передний фронт. opCycleIsStarted выдает одиночный импульс true.
        (04, 3500, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(15.0),  Ok(Some(true)),  Ok(Some(false))),
        // Следующий такт с той же нагрузкой. Импульс Started обязан упасть в false (одновибратор).
        (05, 100,  "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(15.0),  Ok(Some(false)), Ok(Some(false))),
        // Сброс нагрузки до 1.0 (< 5.0). TimerOffDelay (5000ms) запускается. Цикл все еще активен.
        (06, 0,    "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(1.0),   Ok(Some(false)), Ok(Some(false))),
        // Прошло 3 секунды. Крюк пустой. Таймер отключения в процессе.
        (07, 3000, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(1.0),   Ok(Some(false)), Ok(Some(false))),
        // Прошло еще 2.5 секунды (всего 5.5). TimerOffDelay истекает -> opCycleIsActive = false.
        // Срабатывает задний фронт. opCycleIsDone выдает одиночный импульс true.
        (08, 2500, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(1.0),   Ok(Some(false)), Ok(Some(true))),
        // Следующий такт холостого хода. Импульс Done обязан упасть в false.
        (09, 100,  "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(1.0),   Ok(Some(false)), Ok(Some(false))),
    ];
    let flow = FlowContext::new();
    for (step, delay_ms, name, val, target_started, target_done) in test_data.iter().cloned() {
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
        let started_node = task_nodes.get_var("opCycleIsStarted").expect("Variable 'opCycleIsStarted' not found in DAG");
        let done_node = task_nodes.get_var("opCycleIsDone").expect("Variable 'opCycleIsDone' not found in DAG");
        // Проверяем opCycleIsStarted
        let result_started = flow.ignore(started_node.borrow_mut().out());
        match (&result_started, &target_started) {
            (Ok(Some(result)), Ok(Some(target))) => {
                log::debug!("{dbg} | Step {step} | opCycleIsStarted: {:?}", result.value());
                let actual = result.as_bool().value.0;
                assert_eq!(actual, *target, "{dbg} | Step {step} | opCycleIsStarted \n result: {actual} \n target: {target}");
            }
            (Ok(None), Ok(None)) | (Err(_), Err(_)) => {}
            _ => panic!("{dbg} | Step {step} | opCycleIsStarted \n result: {:?} \n target: {:?}", result_started, target_started),
        }
        // Проверяем opCycleIsDone
        let result_done = flow.ignore(done_node.borrow_mut().out());
        match (&result_done, &target_done) {
            (Ok(Some(result)), Ok(Some(target))) => {
                log::debug!("{dbg} | Step {step} | opCycleIsDone: {:?}", result.value());
                let actual = result.as_bool().value.0;
                assert_eq!(actual, *target, "{dbg} | Step {step} | opCycleIsDone \n result: {actual} \n target: {target}");
            }
            (Ok(None), Ok(None)) | (Err(_), Err(_)) => {}
            _ => panic!("{dbg} | Step {step} | opCycleIsDone \n result: {:?} \n target: {:?}", result_done, target_done),
        }
    }
}
///
/// Инкремент и сохранение (FnRetain + FnAcc) сквозного идентификатора по началу каждого нового рабочего цикла.
#[test]
fn gen_op_cycle_id() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    init_each();
    let dbg = "gen_op_cycle_id";
    log::debug!("{dbg}");
    let self_name = Name::new("", dbg);
    let mut task_nodes = TaskNodes::without_retain(dbg, 0);
    let conf = TaskConf::read(&self_name, "src/tests/unit/services/task/cma_recorder/cma-recorder.yaml").unwrap();
    let services = Arc::new(Services::new(dbg, ServicesConf::new(dbg, ConfTree::empty()), None).unwrap());
    task_nodes.build_nodes(&Name::from(dbg), &conf, services).unwrap();
    // Кортеж: (Шаг, Задержка_мс, Имя_сигнала, Значение, Ожидаемый_opCycleId)
    let test_data: &[(i32, u64, &str, Value, Result<Option<i64>, ()>)] = &[
        // Инициализация номинала (LoadR0 = 100.0). Порог активности 5.0. ID цикла = 0 (default)
        (01, 0,    "/App/ied13/db905_visual_data_fast/Winch1.LoadR0", Value::Real(100.0), Ok(None)),
        (02, 0,    "/App/ied13/db905_visual_data_fast/Winch2.LoadR0", Value::Real(100.0), Ok(None)),
        (03, 0,    "/App/ied13/db905_visual_data_fast/Winch3.LoadR0", Value::Real(100.0), Ok(Some(0))),
        // Нагрузка 10.0 (> 5.0). FnThreshold пропускает. Запускается TimerOnDelay (5000ms) для craneIsActive.
        (04, 0,    "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(10.0),  Ok(Some(0))),
        // Прошло 2.5 секунды. Таймер еще идет. Цикл не начат.
        (05, 2500, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(10.0),  Ok(Some(0))),
        // Прошло еще 3 секунды (всего 5.5). TimerOnDelay выдает true -> opCycleIsStarted дает импульс -> ID = 1
        (06, 3000, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(10.0),  Ok(Some(1))),
        // Импульс снят (задний фронт opCycleIsStarted). Счетчик не меняется. ID = 1.
        (07, 100,  "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(10.0),  Ok(Some(1))),
        // Сброс нагрузки до 0.0. Запускается TimerOffDelay (5000ms). Цикл все еще активен. ID = 1.
        (08, 0,    "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(0.0),   Ok(Some(1))),
        // Прошло 5.5 секунды. TimerOffDelay истекает -> Цикл завершен. ID не меняется.
        (09, 5500, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(0.0),   Ok(Some(1))),
        // Стартуем ВТОРОЙ цикл. Нагрузка 20.0. TimerOnDelay пошел.
        (10, 0,    "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(20.0),  Ok(Some(1))),
        // Прошло 5.5 секунды. Таймер пробивается -> Cycle Started импульс -> ID инкрементируется до 2.
        (11, 5500, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(20.0),  Ok(Some(2))),
    ];
    let flow = FlowContext::new();
    for (step, delay_ms, name, val, target_id) in test_data.iter().cloned() {
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
        log::debug!("{dbg} | Step {step} | point '{}': {:?}", point.name(), point.value());
        task_nodes.eval(point);
        let id_node = task_nodes.get_var("opCycleId").expect("Variable 'opCycleId' not found in DAG");
        let result = flow.ignore(id_node.borrow_mut().out());
        // log::debug!("{dbg} | Step {step} | opCycleId: {:?}", result);
        match (&result, &target_id) {
            (Ok(Some(result)), Ok(Some(target))) => {
                log::debug!("{dbg} | Step {step} | opCycleId: {:?}", result.value());
                let actual = result.as_int().value;
                assert_eq!(actual, *target, "{dbg} | Step {step} | opCycleId \n result: {actual} \n target: {target}");
            }
            (Ok(None), Ok(None)) | (Err(_), Err(_)) => {}
            _ => panic!("{dbg} | Step {step} | \n result: {:?} \n target: {:?}", result, target_id),
        }
    }
}
