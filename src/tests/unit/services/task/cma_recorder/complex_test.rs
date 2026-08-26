// #[cfg(test)]

// use sal_core::{dbg::Dbg, error::Error};
use sal_sync::services::{Services, conf::{ConfTree, ServicesConf}, entity::{Cot, Name, Point, PointHlr, Status}, types::Bool};
use testing::entities::test_value::Value;
use std::sync::{Arc, Once};
use debugging::session::{DebugSession, LogLevel};
use crate::services::task::{FlowContext, TaskConf, TaskNodes};
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
///
/// ****************** Нормализация входов ******************
///
/// Расчет относительной нагрузки
/// - Расчет craneLoadRelative
/// - Расчет winch1LoadRelative
/// - Реакция на изменение номинальной нагрузки "на лету"
#[test]
fn calc_relative_load() {
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
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
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
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
///
/// ****************** Определение рабочего цикла ******************
///
///
/// Фиксация активности крана с учетом гистерезиса нагрузки 5% (FnThreshold) и таймеров задержки (TimerOnDelay, TimerOffDelay 5000ms).
#[test]
fn detect_crane_is_active() {
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
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
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
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
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
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
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
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
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
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
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
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
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
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
///
/// ****************** Формирование ID цикла ******************
///
///
/// Инкремент и сохранение (FnRetain + FnAcc) сквозного идентификатора по началу каждого нового рабочего цикла.
#[test]
fn gen_op_cycle_id() {
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
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
///
///
/// ****************** Расчет основных метрик ******************
///
///
/// Сбор и фиксация (FnHold + FnAverage) средней нагрузки на кран за цикл.
/// Проверка работы интегратора FnAverage и корректности сброса по фронту начала цикла.
#[test]
fn calc_crane_cycle_avg_load() {
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
    init_once();
    init_each();
    let dbg = "calc_crane_cycle_avg_load";
    log::debug!("{dbg}");
    let self_name = Name::new("", dbg);
    let mut task_nodes = TaskNodes::without_retain(dbg, 0);
    let conf = TaskConf::read(&self_name, "src/tests/unit/services/task/cma_recorder/cma-recorder.yaml").unwrap();
    let services = Arc::new(Services::new(dbg, ServicesConf::new(dbg, ConfTree::empty()), None).unwrap());
    task_nodes.build_nodes(&Name::from(dbg), &conf, services).unwrap();
    // Кортеж: (Шаг, Задержка_мс, Имя_сигнала, Значение, Ожидаемое_Среднее)
    let test_data: &[(i32, u64, &str, Value, Option<f32>)] = &[
        // Устанавливаем номинал. Порог активности (5%) = 5.0.
        (01, 0,    "/App/ied13/db905_visual_data_fast/Winch1.LoadR0", Value::Real(100.0), None),
        (01, 0,    "/App/ied13/db905_visual_data_fast/Winch2.LoadR0", Value::Real(100.0), None),
        (01, 0,    "/App/ied13/db905_visual_data_fast/Winch3.LoadR0", Value::Real(100.0), None),

        // Явный пуш 4.0 (до цикла). Порог не пройден. Count=4, Sum=4.0 -> Avg=1.0
        (02, 0,    "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(4.0),   Some(1.0)),

        // Пуш 16.0 (> 5.0). TimerOnDelay(5000ms) стартует. Count=5, Sum=20.0 -> Avg=4.0
        (03, 0,    "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(16.0),  Some(4.0)),

        // Прошло 2.5 сек. Таймер еще идет. Count=6, Sum=36.0 -> Avg=6.0
        (04, 2500, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(16.0),  Some(6.0)),

        // Прошло еще 3.0 сек (всего 5.5). TimerOnDelay пробивается -> СТАРТ ЦИКЛА.
        // FnAverage получает reset (opCycleIsStarted = true), сбрасывает сумму и счетчик в 0,
        // а затем сразу добавляет текущее значение (16.0). Count=1, Sum=16.0 -> Avg=16.0
        (05, 3000, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(16.0),  Some(16.0)),

        // В цикле. Нагрузка 20.0. Count=2, Sum=36.0 -> Avg=18.0
        (06, 100,  "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(20.0),  Some(18.0)),

        // В цикле. Нагрузка 24.0. Count=3, Sum=60.0 -> Avg=20.0
        (07, 100,  "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(24.0),  Some(20.0)),

        // Сброс нагрузки до 0.0. TimerOffDelay(5000ms) стартует. Цикл еще активен. Count=4, Sum=60.0 -> Avg=15.0
        (08, 0,    "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(0.0),   Some(15.0)),

        // Прошло 2.5 сек. TimerOffDelay еще идет. Count=5, Sum=60.0 -> Avg=12.0
        (09, 2500, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(0.0),   Some(12.0)),

        // Прошло еще 3.0 сек. TimerOffDelay пробивается -> КОНЕЦ ЦИКЛА. Count=6, Sum=60.0 -> Avg=10.0
        (10, 3000, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(0.0),   Some(10.0)),

        // Вне цикла. Нагрузка 10.0 (Таймер ON стартует снова). Сброса нет, т.к. цикл не начался. Count=7, Sum=70.0 -> Avg=10.0
        (11, 100,  "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(10.0),  Some(10.0)),
    ];
    let flow = FlowContext::new();
    for (step, delay_ms, name, val, target_avg) in test_data.iter().cloned() {
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
        let avg_node = task_nodes.get_var("craneCycleAverageLoad").expect("Variable 'craneCycleAverageLoad' not found in DAG");
        let result = flow.ignore(avg_node.borrow_mut().out());
        match (&result, &target_avg) {
            (Ok(Some(res)), Some(target)) => {
                log::debug!("{dbg} | Step {step} | craneCycleAverageLoad: {:?}", res.value());
                let actual = res.value().as_real();
                assert!((actual - target).abs() < f32::EPSILON, "{dbg} | Step {step} | craneCycleAverageLoad \n result: {actual} \n target: {target}");
            }
            (Ok(None), None) | (Err(_), None) | (Ok(Some(_)), None) => {
                // Если target = None, мы пропускаем строгую проверку (используется для шагов инициализации)
            }
            _ => panic!("{dbg} | Step {step} | \n result: {:?} \n target: {:?}", result, target_avg),
        }
    }
}
///
/// Сбор и фиксация средней нагрузки на каждую лебедку за цикл.
#[test]
fn calc_winch_cycle_avg_load() {
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
    init_once();
    init_each();
    let dbg = "calc_winch_cycle_avg_load";
    log::debug!("{dbg}");
    let self_name = Name::new("", dbg);
    let mut task_nodes = TaskNodes::without_retain(dbg, 0);
    let conf = TaskConf::read(&self_name, "src/tests/unit/services/task/cma_recorder/cma-recorder.yaml").unwrap();
    let services = Arc::new(Services::new(dbg, ServicesConf::new(dbg, ConfTree::empty()), None).unwrap());
    task_nodes.build_nodes(&Name::from(dbg), &conf, services).unwrap();
    // Кортеж: (Шаг, Задержка_мс, Имя_сигнала, Значение, Ожидаемое_Winch1, Ожидаемое_Winch2, Ожидаемое_Winch3)
    // None используется там, где результат является бесконечной дробью. Мы проверяем только "красивые" узловые точки.
    let test_data: &[(i32, u64, &str, Value, Option<f32>, Option<f32>, Option<f32>)] = &[
        // 1-3. Инициализация номиналов. Порог активности (5%) = 5.0
        (01, 0, "/App/ied13/db905_visual_data_fast/Winch1.LoadR0", Value::Real(100.0), None, None, None),
        (02, 0, "/App/ied13/db905_visual_data_fast/Winch2.LoadR0", Value::Real(100.0), None, None, None),
        (03, 0, "/App/ied13/db905_visual_data_fast/Winch3.LoadR0", Value::Real(100.0), None, None, None),

        // 4. Подаем шум на Лебедку 1 (4.0 < 5.0). Такт 4. W1_sum = 4.0. Avg = 4/4 = 1.0. Цикл не стартует.
        (04, 0, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(4.0), Some(1.0), Some(0.0), Some(0.0)),

        // 5. Нагружаем Лебедку 1 (16.0 > 5.0). Такт 5. W1_sum = 4+16=20. Avg = 20/5 = 4.0. Таймер включения пошел.
        (05, 0, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(16.0), Some(4.0), Some(0.0), Some(0.0)),

        // 6. Нагружаем Лебедку 2 (20.0 > 5.0). Такт 6. W1_avg = 36/6 = 6.0. W2_avg = 20/6 = 3.333 (None)
        (06, 0, "/App/ied14/db906_visual_data/Winch2.Load", Value::Real(20.0), Some(6.0), None, None),

        // 7. Ждем 2.5 сек (Таймеры идут). Такт 7.
        (07, 2500, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(16.0), None, None, None),

        // 8. Ждем 3.0 сек. Таймер W1 (5000ms) пробивается -> СТАРТ ЦИКЛА (opCycleIsStarted = true).
        // Все интеграторы получают reset, обнуляют память и захватывают текущие значения как первую точку. Такт 1.
        (08, 3000, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(16.0), Some(16.0), Some(20.0), Some(0.0)),

        // 9. Рабочий цикл. Такт 2. W1=20.0. W1_sum = 16+20 = 36. Avg = 18.0. W2=20. W2_sum = 20+20 = 40. Avg = 20.0.
        (09, 100, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(20.0), Some(18.0), Some(20.0), Some(0.0)),

        // 10. Рабочий цикл. Такт 3. W3=30.0. W3_sum = 0+0+30 = 30. Avg = 10.0. W2_sum = 40+20=60. Avg = 20.0.
        (10, 100, "/App/ied14/db906_visual_data/Winch3.Load", Value::Real(30.0), None, Some(20.0), Some(10.0)),

        // 11. Сброс W1 в 0.0. Такт 4. W1_sum = 56+0 = 56. Avg = 14.0. Стартует таймер отключения W1.
        (11, 0, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(0.0), Some(14.0), Some(20.0), Some(15.0)),

        // 12. Сброс W2 в 0.0. Такт 5. W2_sum = 80+0 = 80. Avg = 16.0. Стартует таймер отключения W2.
        (12, 0, "/App/ied14/db906_visual_data/Winch2.Load", Value::Real(0.0), None, Some(16.0), Some(18.0)),

        // 13. Сброс W3 в 0.0. Такт 6. W3_sum = 90+0 = 90. Avg = 15.0. Стартует таймер отключения W3.
        (13, 0, "/App/ied14/db906_visual_data/Winch3.Load", Value::Real(0.0), None, None, Some(15.0)),

        // 14. Ждем 5.5 сек. Все таймеры отключаются -> КОНЕЦ ЦИКЛА. Такт 7.
        // За этот проход W1 добавляет еще один 0. W1_sum = 56. Avg = 56/7 = 8.0.
        (14, 5500, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(0.0), Some(8.0), None, None),
    ];
    let flow = FlowContext::new();
    for (step, delay_ms, name, val, tgt_w1, tgt_w2, tgt_w3) in test_data.iter().cloned() {
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
        // Извлекаем все три узла интеграторов из DAG
        let node_w1 = task_nodes.get_var("winch1CycleAverageLoad").expect("W1 Var not found");
        let node_w2 = task_nodes.get_var("winch2CycleAverageLoad").expect("W2 Var not found");
        let node_w3 = task_nodes.get_var("winch3CycleAverageLoad").expect("W3 Var not found");
        let res_w1 = flow.ignore(node_w1.borrow_mut().out());
        let res_w2 = flow.ignore(node_w2.borrow_mut().out());
        let res_w3 = flow.ignore(node_w3.borrow_mut().out());
        // Вспомогательное замыкание для проверки (используем 0.001 для защиты от погрешности f32)
        let check = |res: Result<Option<Point>, String>, tgt: Option<f32>, label: &str| {
            match (&res, &tgt) {
                (Ok(Some(r)), Some(t)) => {
                    let actual = r.as_real().value;
                    log::debug!("{dbg} | Step {step} | {label}: {actual}");
                    assert!((actual - *t).abs() < 0.001, "{dbg} | Step {step} | {label} \n result: {actual} \n target: {t}");
                }
                (Ok(None), None) | (Err(_), None) | (Ok(Some(_)), None) => {
                    // Игнорируем проверку, если цель не задана
                }
                _ => panic!("{dbg} | Step {step} | {label} \n result: {:?} \n target: {:?}", res, tgt),
            }
        };
        check(res_w1, tgt_w1, "winch1CycleAverageLoad");
        check(res_w2, tgt_w2, "winch2CycleAverageLoad");
        check(res_w3, tgt_w3, "winch3CycleAverageLoad");
    }
}
///
/// Сбор и фиксация (FnHold + FnMax) пиковой нагрузки на кран за цикл.
#[test]
fn calc_crane_cycle_max_load() {
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
    init_once();
    init_each();
    let dbg = "calc_crane_cycle_max_load";
    log::debug!("{dbg}");
    let self_name = Name::new("", dbg);
    let mut task_nodes = TaskNodes::without_retain(dbg, 0);
    let conf = TaskConf::read(&self_name, "src/tests/unit/services/task/cma_recorder/cma-recorder.yaml").unwrap();
    let services = Arc::new(Services::new(dbg, ServicesConf::new(dbg, ConfTree::empty()), None).unwrap());
    task_nodes.build_nodes(&Name::from(dbg), &conf, services).unwrap();
    // Кортеж: (Шаг, Задержка_мс, Имя_сигнала, Значение, Ожидаемый_MaxLoad)
    let test_data: &[(i32, u64, &str, Value, Result<Option<f32>, ()>)] = &[
        // Инициализация номинала (LoadR0 = 100.0). Порог активности = 5.0. Максимум пока не вычислен.
        (1, 0,    "/App/ied13/db905_visual_data_fast/Winch1.LoadR0", Value::Real(100.0), Ok(Some(0.0))),
        (1, 0,    "/App/ied13/db905_visual_data_fast/Winch2.LoadR0", Value::Real(100.0), Ok(Some(0.0))),
        (1, 0,    "/App/ied13/db905_visual_data_fast/Winch3.LoadR0", Value::Real(100.0), Ok(Some(0.0))),
        // Кидаем мусорный пик 80.0. Таймер не истек, цикл не начат. FnMax ловит его, т.к. работает непрерывно.
        (2, 0,    "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(80.0),  Ok(Some(80.0))),
        // Сбрасываем нагрузку до 10.0 (цикл так и не начался). FnMax всё еще помнит 80.0.
        (3, 100,  "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(10.0),  Ok(Some(80.0))),
        // Держим 10.0. Прошло 5.5 сек. Таймер пробивается -> Cycle Started! FnMax получает reset и захватывает текущие 10.0.
        (4, 5500, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(10.0),  Ok(Some(10.0))),
        // Внутри рабочего цикла нагрузка растет до 45.0. Максимум обновляется.
        (5, 0,    "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(45.0),  Ok(Some(45.0))),
        // Нагрузка падает до 25.0. Максимум остается 45.0.
        (6, 0,    "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(25.0),  Ok(Some(45.0))),
        // Крюк пустой (0.0). Запускается таймер выключения (5 сек). Максимум = 45.0.
        (7, 0,    "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(0.0),   Ok(Some(45.0))),
        // Цикл завершен (прошло 5.5 сек). FnHold и FnMax замирают, сохраняя 45.0.
        (8, 5500, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(0.0),   Ok(Some(45.0))),
        // Старт второго цикла. Нагрузка 20.0. До импульса старта Максимум остается 45.0 (т.к. 20 < 45).
        (9, 0,    "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(20.0),  Ok(Some(45.0))),
        // Прошло 5.5 сек. Cycle 2 Started! FnMax сбрасывается и захватывает текущие 20.0.
        (10, 5500, "/App/ied14/db906_visual_data/Winch1.Load",       Value::Real(20.0),  Ok(Some(20.0))),
    ];

    let flow = FlowContext::new();
    for (step, delay_ms, name, val, target_max) in test_data.iter().cloned() {
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

        let max_node = task_nodes.get_var("craneCycleMaxLoad").expect("Variable 'craneCycleMaxLoad' not found in DAG");
        let result = flow.ignore(max_node.borrow_mut().out());

        match (&result, &target_max) {
            (Ok(Some(result)), Ok(Some(target))) => {
                log::debug!("{dbg} | Step {step} | craneCycleMaxLoad: {:?}", result.value());
                let actual = result.value().as_real();
                assert!((actual - target).abs() < f32::EPSILON, "{dbg} | Step {step} | craneCycleMaxLoad \n result: {actual} \n target: {:?}", target);
            }
            (Ok(None), Ok(None)) | (Err(_), Err(_)) => {}
            _ => panic!("{dbg} | Step {step} | \n result: {:?} \n target: {:?}", result, target_max),
        }
    }
}
///
/// Подсчет длительности (FnTimer) активного состояния цикла в секундах.
#[test]
fn calc_crane_op_cycle_secs() {
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
    init_once();
    init_each();
    let dbg = "calc_crane_op_cycle_secs";
    log::debug!("{dbg}");
    let self_name = Name::new("", dbg);
    let mut task_nodes = TaskNodes::without_retain(dbg, 0);
    let conf = TaskConf::read(&self_name, "src/tests/unit/services/task/cma_recorder/cma-recorder.yaml").unwrap();
    let services = Arc::new(Services::new(dbg, ServicesConf::new(dbg, ConfTree::empty()), None).unwrap());
    task_nodes.build_nodes(&Name::from(dbg), &conf, services).unwrap();
    // Кортеж: (Шаг, Задержка_мс, Имя_сигнала, Значение, Ожидаемые_секунды)
    let test_data: &[(i32, u64, &str, Value, Option<f64>)] = &[
        // Инициализируем номиналы всех лебедок. Порог активности крана (5%) = 5.0
        (1, 0, "/App/ied13/db905_visual_data_fast/Winch1.LoadR0", Value::Real(100.0), None),
        (2, 0, "/App/ied13/db905_visual_data_fast/Winch2.LoadR0", Value::Real(100.0), None),
        (3, 0, "/App/ied13/db905_visual_data_fast/Winch3.LoadR0", Value::Real(100.0), None),

        // Нагрузка 4.0 (< 5.0). Рабочий цикл не запущен. Таймер = 0.0
        (4, 0, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(4.0), Some(0.0)),

        // Нагрузка 16.0 (> 5.0). Запуск TimerOnDelay (5000ms). Цикл еще не начался.
        (5, 0, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(16.0), Some(0.0)),

        // Прошло 2.5 сек. Таймер на старте еще идет. Цикл не начался.
        (6, 2500, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(16.0), Some(0.0)),

        // Прошло еще 3.0 сек (всего 5.5). TimerOnDelay пробивается -> СТАРТ ЦИКЛА.
        // opCycleIsStarted генерирует RisingEdge и сбрасывает FnTimer в 0.0. Но секундомер на этом такте еще не запущен
        (7, 3000, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(16.0), Some(0.0)),
        // Вот тут секундомер пошел.
        (71, 0, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(16.0), Some(0.0)),

        // В цикле. Прошло 2.0 сек. Секундомер должен показать 2.0.
        (8, 2000, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(16.0), Some(2.0)),

        // Сброс нагрузки до 0.0. TimerOffDelay (5000ms) стартует. Цикл ЕЩЕ АКТИВЕН.
        // Прошло еще 3.0 сек. Секундомер продолжает считать (2.0 + 3.0 = 5.0).
        (9, 3000, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(0.0), Some(5.0)),

        // Прошло 2.5 сек. TimerOffDelay еще идет. Секундомер продолжает (5.0 + 2.5 = 7.5).
        (10, 2500, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(0.0), Some(7.5)),

        // Прошло еще 3.0 сек. TimerOffDelay пробивается -> КОНЕЦ ЦИКЛА.
        // opCycleIsActive падает в false. FnTimer фиксирует время (7.5 + 3.0 = 10.5).
        (11, 3000, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(0.0), Some(10.5)),

        // Вне цикла. Прошло 2.0 сек. Цикл остановлен, секундомер удерживает последнее значение (10.5).
        (12, 2000, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(0.0), Some(10.5)),
    ];
    let flow = FlowContext::new();
    for (step, delay_ms, name, val, target_secs) in test_data.iter().cloned() {
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
        let timer_node = task_nodes.get_var("craneOperatingCycleSecs").expect("Variable 'craneOperatingCycleSecs' not found in DAG");
        let result = flow.ignore(timer_node.borrow_mut().out());
        match (&result, &target_secs) {
            (Ok(Some(res)), Some(target)) => {
                log::debug!("{dbg} | Step {step} | craneOperatingCycleSecs: {:?}", res.value());
                // Приводим к double, так как время в FnTimer считается в f64
                let actual = res.as_double().value;
                // Допуск 0.1 секунды (100 мс). Вызов thread::sleep зависит от планировщика ОС
                // и не дает идеальной точности, поэтому f32::EPSILON тут приведет к flaky-тестам.
                let epsilon = 0.1;
                assert!((actual - target).abs() < epsilon,
                    "{dbg} | Step {step} | craneOperatingCycleSecs \n result: {actual} \n target: {target} \n diff: {}", (actual - target).abs());
            }
            (Ok(None), None) | (Err(_), None) | (Ok(Some(_)), None) => {
                // Если target = None, мы пропускаем строгую проверку (используется для шагов инициализации)
            }
            _ => panic!("{dbg} | Step {step} | \n result: {:?} \n target: {:?}", result, target_secs),
        }
    }
}
///
/// Расчет класса тревоги. Суммирование Overload > 25% (Class 4), Overload > 10% (Class 2) и срабатываний SWL.
#[test]
fn calc_alarm_class() {}
///
/// Расчет характеристического числа крана (усталость по кубу максимальной относительной нагрузки). Обновление стейта по завершению цикла.
#[test]
fn calc_crane_eigen_value() {
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
    init_once();
    init_each();
    let dbg = "calc_crane_eigen_value";
    log::debug!("{dbg}");
    let self_name = Name::new("", dbg);
    let mut task_nodes = TaskNodes::without_retain(dbg, 0);
    let conf = TaskConf::read(&self_name, "src/tests/unit/services/task/cma_recorder/cma-recorder.yaml").unwrap();
    let services = Arc::new(Services::new(dbg, ServicesConf::new(dbg, ConfTree::empty()), None).unwrap());
    task_nodes.build_nodes(&Name::from(dbg), &conf, services).unwrap();
    // Кортеж: (Шаг, Задержка_мс, Имя_сигнала, Значение, Ожидаемое_Значение)
    let test_data: &[(i32, u64, &str, Value, Option<f32>)] = &[
        // Инициализация номиналов. Кран и лебедки = 100.0. Порог активности = 5.0
        (1, 0, "/App/ied13/db905_visual_data_fast/Winch1.LoadR0", Value::Real(100.0), None),
        (2, 0, "/App/ied13/db905_visual_data_fast/Winch2.LoadR0", Value::Real(100.0), None),
        (3, 0, "/App/ied13/db905_visual_data_fast/Winch3.LoadR0", Value::Real(100.0), None),
        // Пуш 50.0 (Относительная 0.5). Запуск TimerOnDelay. Характеристическое число пока 0.0
        (4, 0, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(50.0), None),
        // Пробиваем таймер старта (5500мс). СТАРТ ЦИКЛА. max_load фиксирует 50.0
        (5, 5500, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(50.0), None),
        // Сброс груза до 0.0. Запуск TimerOffDelay. Цикл еще идет
        (6, 0, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(0.0), None),
        // Пробиваем таймер остановки (5500мс). КОНЕЦ ЦИКЛА (opCycleIsDone = true).
        // Расчет: 0.0 + (50.0 / 100.0)^3 = 0.125
        (7, 5500, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(0.0), Some(0.125)),
        // Пуш 100.0 (Относительная 1.0). Запуск таймера старта второго цикла
        (8, 0, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(100.0), None),
        // Пробиваем старт второго цикла. max_load фиксирует 100.0
        (9, 5500, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(100.0), None),
        // Сброс груза до 0.0. Запуск таймера остановки второго цикла
        (10, 0, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(0.0), None),
        // Пробиваем остановку второго цикла.
        // Так как сумму мы аккумулируем на уровне БД (SET value = value + current), то тут снова значение за цикл
        // Расчет: 0.0 + (100.0 / 100.0)^3 = 1.000
        (11, 5500, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(0.0), Some(1.000)),
    ];
    let flow = FlowContext::new();
    for (step, delay_ms, name, val, target_eigen) in test_data.iter().cloned() {
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
        let eigen_node = task_nodes.get_var("craneEigenValue").expect("Variable 'craneEigenValue' not found in DAG");
        let result = flow.ignore(eigen_node.borrow_mut().out());
        match (&result, &target_eigen) {
            (Ok(Some(res)), Some(target)) => {
                log::debug!("{dbg} | Step {step} | craneEigenValue: {:?}", res.value());
                let actual = res.value().as_real();
                assert!((actual - target).abs() < f32::EPSILON, "{dbg} | Step {step} | craneEigenValue \n result: {actual} \n target: {target}");
            }
            (Ok(None), None) | (Err(_), None) | (Ok(Some(_)), None) => {}
            _ => panic!("{dbg} | Step {step} | \n result: {:?} \n target: {:?}", result, target_eigen),
        }
    }
}
///
/// Расчет характеристического числа лебедок (усталость по кубу средней нагрузки, умноженная на длительность).
#[test]
fn calc_winch_eigen_value() {}
///
///
/// ****************** Расчет диапазонов загрузки ******************
///
/// Формирование 13 логических флагов попадания средней нагрузки (0.05..1.25+) по итогам цикла (FnAnd + FnGe + FnLt).
#[test]
fn eval_load_ranges_matrix() {
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
    init_once();
    init_each();
    let dbg = "eval_load_ranges_matrix";
    log::debug!("{dbg}");
    let self_name = Name::new("", dbg);
    let mut task_nodes = TaskNodes::without_retain(dbg, 0);
    let conf = TaskConf::read(&self_name, "src/tests/unit/services/task/cma_recorder/cma-recorder.yaml").unwrap();
    let services = Arc::new(Services::new(dbg, ServicesConf::new(dbg, ConfTree::empty()), None).unwrap());
    task_nodes.build_nodes(&Name::from(dbg), &conf, services).unwrap();
    // Кортеж: (Шаг, Задержка_мс, Имя_сигнала, Значение, Ожидаемый_0_05_0_15, Ожидаемый_0_25_0_35)
    let test_data: &[(i32, u64, &str, Value, Option<bool>, Option<bool>)] = &[
        // 1. Инициализация номиналов. Порог активности = 5.0
        (1,  0,    "/App/ied13/db905_visual_data_fast/Winch1.LoadR0", Value::Real(100.0), None, None),
        (2,  0,    "/App/ied13/db905_visual_data_fast/Winch2.LoadR0", Value::Real(100.0), None, None),
        (3,  0,    "/App/ied13/db905_visual_data_fast/Winch3.LoadR0", Value::Real(100.0), None, None),
        // --- ЦИКЛ 1: Целевая средняя нагрузка 12.0 (Относительная 0.12) -> Диапазон 0.05..0.15 ---
        // Шаг 4: Нагрузка 30.0. Порог пройден.
        (4,  0,    "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(30.0),  Some(false), Some(false)),
        // Шаг 5: TimerOnDelay пробивается -> СТАРТ ЦИКЛА. Сброс среднего. Count=1, Sum=30, Avg=30.0
        (5,  5500, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(30.0),  Some(false), Some(false)),
        // Шаг 6: Середина цикла. Count=2, Sum=60, Avg=30.0
        (6,  100,  "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(30.0),  Some(false), Some(false)),
        // Шаг 7: Упали ниже порога (старт TimerOffDelay). Count=3, Sum=60, Avg=20.0
        (7,  0,    "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(0.0),   Some(false), Some(false)),
        // Шаг 8: Ждем таймер. Count=4, Sum=60, Avg=15.0
        (8,  2500, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(0.0),   Some(false), Some(false)),
        // Шаг 9: TimerOffDelay пробивается -> КОНЕЦ ЦИКЛА! Count=5, Sum=60, Avg=12.0 (0.12). Флаг 0_05_0_15 зажигается на 1 такт!
        (9,  3000, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(0.0),   Some(true),  Some(false)),
        // Шаг 10: Следующий такт покоя. Флаг снова гаснет.
        (10, 100,  "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(0.0),   Some(false), Some(false)),
        // --- ЦИКЛ 2: Целевая средняя нагрузка 30.0 (Относительная 0.30) -> Диапазон 0.25..0.35 ---
        // Шаг 11: Нагрузка 75.0. Порог пройден.
        (11, 100,  "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(75.0),  Some(false), Some(false)),
        // Шаг 12: TimerOnDelay пробивается -> СТАРТ ЦИКЛА. Сброс среднего. Count=1, Sum=75, Avg=75.0
        (12, 5500, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(75.0),  Some(false), Some(false)),
        // Шаг 13: Середина цикла. Count=2, Sum=150, Avg=75.0
        (13, 100,  "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(75.0),  Some(false), Some(false)),
        // Шаг 14: Упали ниже порога (старт TimerOffDelay). Count=3, Sum=150, Avg=50.0
        (14, 0,    "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(0.0),   Some(false), Some(false)),
        // Шаг 15: Ждем таймер. Count=4, Sum=150, Avg=37.5
        (15, 2500, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(0.0),   Some(false), Some(false)),
        // Шаг 16: TimerOffDelay пробивается -> КОНЕЦ ЦИКЛА! Count=5, Sum=150, Avg=30.0 (0.30). Флаг 0_25_0_35 зажигается на 1 такт!
        (16, 3000, "/App/ied14/db906_visual_data/Winch1.Load",        Value::Real(0.0),   Some(false), Some(true)),
    ];
    let flow = FlowContext::new();
    for (step, delay_ms, name, val, target_05_15, target_25_35) in test_data.iter().cloned() {
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
        let node_05_15 = task_nodes.get_var("craneLoadInRange_0_05_0_15").expect("Variable 'craneLoadInRange_0_05_0_15' not found");
        let node_25_35 = task_nodes.get_var("craneLoadInRange_0_25_0_35").expect("Variable 'craneLoadInRange_0_25_0_35' not found");
        let res_05_15 = flow.ignore(node_05_15.borrow_mut().out());
        let res_25_35 = flow.ignore(node_25_35.borrow_mut().out());
        match (&res_05_15, &target_05_15) {
            (Ok(Some(res)), Some(target)) => {
                let actual = res.to_bool().as_bool().value.0;
                assert_eq!(actual, *target, "{dbg} | Step {step} | 0.05..0.15 mismatch \n result: {actual} \n target: {target}");
            }
            (Ok(None), None) | (Err(_), None) | (Ok(Some(_)), None) => {}
            _ => panic!("{dbg} | Step {step} | 0.05..0.15 \n result: {:?} \n target: {:?}", res_05_15, target_05_15),
        }
        match (&res_25_35, &target_25_35) {
            (Ok(Some(res)), Some(target)) => {
                let actual = res.to_bool().as_bool().value.0;
                assert_eq!(actual, *target, "{dbg} | Step {step} | 0.25..0.35 mismatch \n result: {actual} \n target: {target}");
            }
            (Ok(None), None) | (Err(_), None) | (Ok(Some(_)), None) => {}
            _ => panic!("{dbg} | Step {step} | 0.25..0.35 \n result: {:?} \n target: {:?}", res_25_35, target_25_35),
        }
    }
}
///
/// Каскадный выбор (FnSelect) строкового ключа диапазона ('0_05-0_15' и т.д.) на основе матрицы логических флагов.
#[test]
fn eval_cycle_load_range_str() {
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
    init_once();
    init_each();
    let dbg = "eval_cycle_load_range_str";
    log::debug!("{dbg}");
    let self_name = Name::new("", dbg);
    let mut task_nodes = TaskNodes::without_retain(dbg, 0);
    let conf = TaskConf::read(&self_name, "src/tests/unit/services/task/cma_recorder/cma-recorder.yaml").unwrap();
    let services = Arc::new(Services::new(dbg, ServicesConf::new(dbg, ConfTree::empty()), None).unwrap());
    task_nodes.build_nodes(&Name::from(dbg), &conf, services).unwrap();
    // Кортеж: (Шаг, Задержка_мс, Имя_сигнала, Значение, Ожидаемый_Range)
    // Target типа Option<&str>, так как узел возвращает строку только один такт в конце цикла
    let test_data: &[(i32, u64, &str, Value, Option<&str>)] = &[
        // 1. Инициализация номиналов (LoadR0). Чтобы не словить деление на ноль по всему графу.
        (01, 0, "/App/ied13/db905_visual_data_fast/Winch1.LoadR0", Value::Real(100.0), None),
        (02, 0, "/App/ied13/db905_visual_data_fast/Winch2.LoadR0", Value::Real(100.0), None),
        (03, 0, "/App/ied13/db905_visual_data_fast/Winch3.LoadR0", Value::Real(100.0), None),

        // ====================================================================================
        // ЦИКЛ 1: Тестируем попадание в диапазон '0_05-0_15' (целевое среднее = 10%)
        // ====================================================================================
        // Имитируем подъем груза. Подаем 30.0
        (04, 0,    "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(30.0), None),
        // Ждем > 5 сек, чтобы TimerOnDelay зафиксировал opCycleIsStarted (Старт цикла)
        // В этот момент сбрасывается среднее (Count=1, Sum=30.0 -> Avg=30.0)
        (05, 5500, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(30.0), None),
        // Груз снят, нагрузка падает до 0.0 (Count=2, Sum=30.0 -> Avg=15.0). Запускается TimerOffDelay.
        (06, 0,    "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(0.0),  None),
        // Ждем > 5 сек, TimerOffDelay пробивается -> opCycleIsDone = true (Конец цикла).
        // Добавляется еще одно событие 0.0 (Count=3, Sum=30.0 -> Avg=10.0).
        // Относительная нагрузка: 10.0 / 100.0 = 0.10. Узел FnSelect выдает совпадение!
        (07, 5500, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(0.0),  Some("0_05-0_15")),
        // Следующий такт. opCycleIsDone = false. Каскад FnSelect уходит в пустоту (вернет None).
        (08, 100,  "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(0.0),  None),

        // ====================================================================================
        // ЦИКЛ 2: Тестируем попадание в диапазон '0_15-0_25' (целевое среднее = 20%)
        // ====================================================================================
        // Старт (Count=1, Sum=60.0 -> Avg=60.0)
        (09, 0,    "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(60.0), None),
        (10, 5500, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(60.0), None),
        // Сброс (Count=2, Sum=60.0 -> Avg=30.0)
        (11, 0,    "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(0.0),  None),
        // Финиш (Count=3, Sum=60.0 -> Avg=20.0). Относительная = 0.20
        (12, 5500, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(0.0),  Some("0_15-0_25")),
        // Холостой такт
        (13, 100,  "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(0.0),  None),

        // ====================================================================================
        // ЦИКЛ 3: Тестируем жесткий перегруз '1_25' (целевое среднее = 130%)
        // ====================================================================================
        // Проверяем самую последнюю ветку без default
        // Старт (Count=1, Sum=390.0 -> Avg=390.0)
        (14, 0,    "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(390.0), None),
        (15, 5500, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(390.0), None),
        // Сброс (Count=2, Sum=390.0 -> Avg=195.0)
        (16, 0,    "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(0.0),   None),
        // Финиш (Count=3, Sum=390.0 -> Avg=130.0). Относительная = 1.30
        (17, 5500, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(0.0),   Some("1_25")),
        // Холостой такт
        (18, 100,  "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(0.0),   None),
    ];
    let flow = FlowContext::new();
    for (step, delay_ms, name, val, target_range) in test_data.iter().cloned() {
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
        let cycle_node = task_nodes.get_var("cycleLoadRange").expect("Variable 'cycleLoadRange' not found in DAG");
        let result = flow.ignore(cycle_node.borrow_mut().out());
        match (&result, &target_range) {
            (Ok(Some(res)), Some(target)) => {
                log::debug!("{dbg} | Step {step} | cycleLoadRange: {:?}", res.value());
                // Извлекаем строковое значение из точки Point::String
                let actual = res.as_string().value;
                assert_eq!(actual, *target, "{dbg} | Step {step} | cycleLoadRange \n actual: {actual} \n target: {target}");
            }
            // Пропускаем шаги, где мы не ждем формирования строкового ключа диапазона
            (Ok(None), None) | (Err(_), None) | (Ok(Some(_)), None) => {}

            _ => panic!("{dbg} | Step {step} | \n result: {:?} \n target: {:?}", result, target_range),
        }
    }
}
///
///
/// ****************** Накопители (Basic Metrics) ******************
///
///
/// Накопление общего времени работы в секундах (FnSql UPDATE) для крана, лебедок и насосной станции.
#[test]
fn metric_total_op_secs() {
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
    init_once();
    init_each();
    let dbg = "metric_total_op_secs";
    log::debug!("{dbg}");
    let self_name = Name::new("", dbg);
    let mut task_nodes = TaskNodes::without_retain(dbg, 0);
    let conf = TaskConf::read(&self_name, "src/tests/unit/services/task/cma_recorder/cma-recorder.yaml").unwrap();
    let services = Arc::new(Services::new(dbg, ServicesConf::new(dbg, ConfTree::empty()), None).unwrap());
    task_nodes.build_nodes(&Name::from(dbg), &conf, services).unwrap();
    // Кортеж: (Шаг, Задержка_мс, Имя_сигнала, Значение, Ожидаемое_Время_Сек)
    let test_data: &[(i32, u64, &str, Value, Option<f64>)] = &[
        // 1. Инициализация номиналов (LoadR0 = 100.0). Порог активности (5%) = 5.0.
        (1, 0, "/App/ied13/db905_visual_data_fast/Winch1.LoadR0", Value::Real(100.0), None),
        (2, 0, "/App/ied13/db905_visual_data_fast/Winch2.LoadR0", Value::Real(100.0), None),
        (3, 0, "/App/ied13/db905_visual_data_fast/Winch3.LoadR0", Value::Real(100.0), None),
        // 4. Нагрузка 15.0 (> 5.0). TimerOnDelay(5000ms) пошел. Цикл НЕ начат, время 0.0.
        (4, 0, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(15.0), Some(0.0)),
        // 5. Прошло 3 сек. Цикл еще не начат, таймер стоит.
        (5, 3000, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(15.0), Some(0.0)),
        // 6. Прошло еще 2.5 сек (всего 5.5). TimerOnDelay пробивается -> СТАРТ ЦИКЛА. Сброс и старт секундомера.
        (6, 2500, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(15.0), Some(0.0)),
        // 7. Прошло 2 сек внутри цикла. Таймер насчитал ~2.0 сек.
        (7, 2000, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(15.0), Some(2.0)),
        // 8. Груз снят (0.0). TimerOffDelay(5000ms) пошел. Цикл еще активен!
        (8, 0, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(0.0), Some(2.0)),
        // 9. Прошло 3 сек без груза. Таймер Off еще идет, цикл активен, секундомер тикает. Время ~5.0.
        (9, 3000, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(0.0), Some(5.0)),
        // 10. Прошло 2.5 сек (всего 5.5). TimerOffDelay истек -> КОНЕЦ ЦИКЛА. Таймер останавливается на ~7.5.
        (10, 2500, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(0.0), Some(7.5)),
        // 11. Прошла 1 сек вне цикла. Значение заморожено.
        (11, 1000, "/App/ied14/db906_visual_data/Winch1.Load", Value::Real(0.0), Some(7.5)),
    ];
    let flow = FlowContext::new();
    for (step, delay_ms, name, val, target_secs) in test_data.iter().cloned() {
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
        let secs_node = task_nodes.get_var("craneOperatingCycleSecs").expect("Variable 'craneOperatingCycleSecs' not found in DAG");
        let result = flow.ignore(secs_node.borrow_mut().out());
        match (&result, &target_secs) {
            (Ok(Some(res)), Some(target)) => {
                log::debug!("{dbg} | Step {step} | craneOperatingCycleSecs: {:?}", res.value());
                let actual = res.value().as_double();
                let diff = (actual - target).abs();
                assert!(diff < 0.25, "{dbg} | Step {step} | craneOperatingCycleSecs \n result: {actual} \n target: {target} \n diff: {diff}");
            }
            (Ok(None), None) | (Err(_), None) | (Ok(Some(_)), None) => {}
            _ => panic!("{dbg} | Step {step} | \n result: {:?} \n target: {:?}", result, target_secs),
        }
    }
}
///
/// Инкремент суммарного числа рабочих циклов (по заднему фронту).
#[test]
fn metric_total_cycles_count() {}
///
/// Инкремент счетчика циклов в конкретной корзине диапазона нагрузки (используя строковый ключ диапазона).
#[test]
fn metric_cycles_distribution() {}
///
/// Суммирование поднятой массы в конкретной корзине диапазона нагрузки.
#[test]
fn metric_mass_distribution() {}
///
/// Суммирование общей массы всех поднятых грузов для крана и индивидуально для лебедок.
#[test]
fn metric_total_lifted_mass() {}
///
/// Инкремент счетчика срабатываний ограничителя грузоподъемности по переднему фронту датчика (SWLProtection).
#[test]
fn metric_swl_trip_count() {}
///
/// Запись текущего характеристического числа (FnSql UPDATE) в БД по завершении цикла.
#[test]
fn metric_eigen_value_sync() {}
///
///
/// ****************** Журнал рабочих циклов ******************
///
///
/// Запись в БД rec_operating_cycle с таймстемпами начала/конца и Alarm Class.
#[test]
fn export_op_cycle_record() {}
///
/// Запись в БД rec_operating_metric значений average_load и max_load с привязкой к ID цикла.
#[test]
fn export_op_metric_record() {}
///
///
/// ****************** Телеметрия и события (Live) ******************
///
///
/// Экспорт в БД событий CraneMode.MOPS по факту их изменения (IsChangedValue).
#[test]
fn live_mops_event() {}
///
/// Экспорт в БД событий CraneMode.AOPS по факту их изменения.
#[test]
fn live_aops_event() {}
///
/// Расчет плавающего порога чувствительности (PiecewiseLineApprox) от 30% на пустом крюке до 0.1% на максимальной нагрузке.
#[test]
fn live_dynamic_deadband() {}
///
/// Экспорт в БД изменения нагрузки крана и лебедок. Срабатывает только при превышении динамического порога (FnThreshold) для снижения шума.
#[test]
fn live_load_filter_event() {}
