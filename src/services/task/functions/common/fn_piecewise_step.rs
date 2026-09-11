use function_name::named;
use regex::Regex;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::services::entity::{Cot, Point, PointHlr};
use std::sync::{LazyLock, atomic::{AtomicUsize, Ordering}};
use concat_string::concat_string;
use crate::{
    domain::FnOutRef, err, err_pass, services::task::{FlowContext, FnFlow, functions::{
        FnKind, FnOut, FnResult
    }}
};
///
/// ### Function | Piecewise Step (кусочно-ступенчатая функция)
/// 
/// Делает выбор выходного значения исходя из значения на входе.
/// Значение и его тип задаются в конфиге для каждого диапазона.
/// Диапазоны обрабатываются как `left <= x < right` (левая граница включена, правая исключена).
/// 
/// Если входное значение не попадает ни в один из диапазонов, возвращает Ok(None).
/// 
/// ```yaml
/// fn PiecewiseStep:
///     piecewise:
///         .. 0.05:        '-0_05'
///         0.05 .. 0.15:   '0_05-0_15'
///         0.15 .. 0.25:   '0_15-0_25'
///         0.25 .. 0.35:   '0_25-0_35'
///         0.35..:         '0_35-'
///     input: inputValue
/// ```
#[derive(Debug)]
pub struct FnPiecewiseStep {
    id: String,
    kind: FnKind,
    input: FnOutRef,
    piecewise: PiecewiseStep,
}
// 
impl FnPiecewiseStep {
    ///
    /// Creates new instance of the FnPiecewiseStep
    #[named]
    pub fn new(parent: impl AsRef<str>, txid: usize, input: FnOutRef, piecewise: &serde_yaml::Value) -> Result<Self, Error> {
        let id = format!("{}/FnPiecewiseStep{}", parent.as_ref(), COUNT.fetch_add(1, Ordering::SeqCst));
        let piecewise = PiecewiseStep::try_new(&id, txid, piecewise)
            .map_err(|err| err_pass!(id, err, "Wrong conf"))?;
        Ok(Self { 
            id,
            kind: FnKind::Fn,
            input,
            piecewise,
        })
    }
}
//
impl FnOut for FnPiecewiseStep { 
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
        let value: f64 = match &input {
            Point::Bool(_) | Point::Int(_) | Point::Real(_) | Point::Double(_) => input.to_double().as_double().value,
            Point::String(val) => {
                val.value.parse()
                    .map_err(|_| concat_string!(self.id, ".out | Invalid input '", val.value, "'"))?
            }
            _ => return Err(concat_string!(self.id, ".out | Invalid input type '", input.typ().to_string(), "'")),
        };
        if value.is_nan() {
            return Err(concat_string!(self.id, ".out | Math error: Received NaN value instead of valid number"));
        }
        let out = match self.piecewise.eval(value) {
            Some(v) => v.clone()
                .with_cot(Cot::Inf)
                .with_status(input.status())
                .with_ts(input.ts()),
            None => return Ok(None), // Если нет значения, попали в разрыв функции -> останавливаем ветку вычислений
        };
        log::trace!("{}.out | out: {:?}", self.id, &out);
        flow.wrap(out)
    }
    //
    fn hard_reset(&mut self) {
        self.input.borrow_mut().hard_reset();
    }
    //
    fn reset(&mut self) {}
}
///
/// Global static counter of FnPiecewiseStep instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Single step of the piecewise step function
#[derive(Debug)]
struct LinearStep {
    left: Option<f64>,
    right: Option<f64>,
    val: Point,
}
impl LinearStep {
    const RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^\s*(?P<start>\d+(?:\.\d+)?)?\s*\.\.\s*(?P<end>\d+(?:\.\d+)?)?\s*$").unwrap()
    });
    /// ### Создает новый интервал на основе строкового представления диапазона и значения.
    ///
    /// ```yaml
    /// ..0.05:      'x < 0.05'
    /// 0.05..0.15:  '0.05 <= x < 0.15'
    /// 0.15..:      'x >= 0.15'
    /// ```
    /// 
    /// Поддерживает форматы: `start..end`, `start..` (открытый справа) и `..end` (открытый слева).
    /// Если `start > end`, границы автоматически меняются местами.
    ///
    /// Возвращает ошибку, если:
    /// * Строка `range` не соответствует синтаксису диапазонов.
    /// * Границы не парсятся в `f64`.
    /// * Отсутствуют обе границы (`..`).
    /// * Начало и конец диапазона математически равны.
    #[named]
    fn try_new(range: impl AsRef<str>, val: Point) -> Result<Self, Error> {
        let range = range.as_ref();
        let caps = Self::RE.captures(range)
            .ok_or_else(|| err!(Self, "Неверный формат диапазона: '{}'", range))?;
        let mut left: Option<f64> = caps.name("start").map(|m| m.as_str()
            .parse()
            .map_err(|err: std::num::ParseFloatError| err_pass!(Self, err, "Неверный формат начала диапазона: '{}'", m.as_str())))
            .transpose()?;
        let mut right: Option<f64> = caps.name("end").map(|m| m.as_str()
            .parse()
            .map_err(|err: std::num::ParseFloatError| err_pass!(Self, err, "Неверный формат конца диапазона: '{}'", m.as_str())))
            .transpose()?;
        if left.is_none() && right.is_none() {
            return Err(err!(Self, "Неверный диапазон '{range}', должна быть задана хотя бы одна граница"));
        }
        if let (Some(s), Some(e)) = (left, right) {
            if s > e {
                std::mem::swap(&mut left, &mut right);
            }
            if (e - s).abs() <= f64::MIN_POSITIVE {
                return Err(err!(Self, "Can't create step {range}, Start and End are equals"));
            }
        }
        Ok(Self {left, right, val})
    }
    ///
    /// Returns step's value if `x` in the it's range [left, right)
    #[inline]
    fn eval(&self, x: f64) -> Option<&Point> {
        match (self.left, self.right) {
            (None, None) => return None,
            (None, Some(right)) => if x < right {
                return Some(&self.val);
            },
            (Some(left), None) => if x >= left {
                return Some(&self.val);
            },
            (Some(left), Some(right)) => if x >= left && x < right {
                return Some(&self.val);
            },
        }
        None
    }
}
///
/// The collection of the LinearStep
#[derive(Debug)]
pub struct PiecewiseStep {
    id: Dbg,
    lines: Vec<LinearStep>,
}
impl PiecewiseStep {
    /// ### Returns `PiecewiseStep` parsed from YAML
    /// 
    /// ```yaml
    /// ..0.05:      '- 0.05'
    /// 0.05..0.15:  '0.05 - 0.15'
    /// 0.15..:      '0.15 -'
    /// ```
    #[named]
    pub fn try_new(parent: impl AsRef<str>, txid: usize, points: &serde_yaml::Value) -> Result<Self, Error> {
        let parent = parent.as_ref();
        let dbg = Dbg::new(parent, crate::me::<Self>());
        let pairs: serde_yaml::Mapping = serde_yaml::from_value(points.clone())
            .map_err(|err| err_pass!(dbg, err, "Can't parse points from {:?}", points))?;
        let mut ranges = Vec::with_capacity(pairs.len());
        for (range, val) in pairs {
            let range = range.as_str().ok_or(err!(dbg, "Can't parse range {:?} as str", range))?;
            let val = match &val {
                serde_yaml::Value::Null => return Err(err!(dbg, "In range '{range} Null value isn't supported: {:?}", val)),
                serde_yaml::Value::Bool(v) => Point::new(txid, parent, *v),
                serde_yaml::Value::Number(number) => {
                    if let Some(v) = number.as_i64() {
                        Point::new(txid, parent, v)
                    } else if let Some(v) = number.as_f64() {
                        Point::new(txid, parent, v)
                    } else {
                        return Err(err!(dbg, "In range '{range} value {:?} isn't a valid number", val));
                    }
                },
                serde_yaml::Value::String(v) => Point::new(txid, parent, v.to_string()),
                serde_yaml::Value::Sequence(_) => return Err(err!(dbg, "In range '{range} value isn't supported: {:?}", val)),
                serde_yaml::Value::Mapping(_) => return Err(err!(dbg, "In range '{range} value isn't supported: {:?}", val)),
                serde_yaml::Value::Tagged(_) => return Err(err!(dbg, "In range '{range} value isn't supported: {:?}", val)),
            };
            ranges.push(
                LinearStep::try_new(range, val).map_err(|err| err_pass!(dbg, err))?
            );
        }
        Ok(Self { id: dbg, lines: ranges })
    }
    ///
    /// ### Evaluates the piecewise step function at point `x`.
    ///
    /// Returns the value of the step which range contains `x`.
    /// If `x` doesn't belong to any range, returns `None` (gap of the function).
    ///
    /// **Examples**
    /// ```ignore
    /// let yaml = "..0.5: 'low'\n0.5..: 'high'";
    /// let conf: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
    /// let func = PiecewiseStep::try_new("test", 1, &conf).unwrap();
    /// // func.eval(0.2) -> 'low', func.eval(0.7) -> 'high'
    /// ```
    pub fn eval(&self, x: f64) -> Option<&Point> {
        log::trace!("{}.eval | value: {:?}", self.id, x);
        self.lines.iter().find_map(|step| step.eval(x))
    }
}
///
/// Basic Tests
#[cfg(test)]
mod piecewise_step_tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use sal_sync::services::entity::{Cot, PointHlr, Status};

    const STEPS_YAML: &str = "
        ..0.05: '-0_05'
        0.05..0.15: '0_05-0_15'
        0.15..0.25: '0_15-0_25'
        0.25..: '0_25-'
    ";
    const CLOSED_YAML: &str = "
        0.05..0.15: '0_05-0_15'
        0.15..0.25: '0_15-0_25'
    ";

    fn setup_func(yaml: &str) -> PiecewiseStep {
        let conf: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        PiecewiseStep::try_new("test_parent", 1, &conf).unwrap()
    }
    ///
    /// Возвращает строковое значение результата `eval`
    fn eval_string(func: &PiecewiseStep, x: f64) -> Option<String> {
        match func.eval(x) {
            Some(Point::String(val)) => Some(val.value.clone()),
            Some(other) => panic!("Expected Point::String, got: {:?}", other),
            None => None,
        }
    }

    #[test]
    fn test_open_left_range() {
        let func = setup_func(STEPS_YAML);
        assert_eq!(eval_string(&func, -100.0), Some("-0_05".to_string()));
        assert_eq!(eval_string(&func, 0.0), Some("-0_05".to_string()));
        assert_eq!(eval_string(&func, 0.049), Some("-0_05".to_string()));
    }
    #[test]
    fn test_range_boundaries() {
        let func = setup_func(STEPS_YAML);
        // Левая граница включена, правая исключена: 0.05 <= x < 0.15
        assert_eq!(eval_string(&func, 0.05), Some("0_05-0_15".to_string()));
        assert_eq!(eval_string(&func, 0.149), Some("0_05-0_15".to_string()));
        // Значение правой границы попадает уже в следующий диапазон
        assert_eq!(eval_string(&func, 0.15), Some("0_15-0_25".to_string()));
    }
    #[test]
    fn test_open_right_range() {
        let func = setup_func(STEPS_YAML);
        assert_eq!(eval_string(&func, 0.25), Some("0_25-".to_string()));
        assert_eq!(eval_string(&func, 100.0), Some("0_25-".to_string()));
    }
    #[test]
    fn test_out_of_ranges_returns_none() {
        // Все диапазоны закрытые - вне их функция не определена (разрыв ветки вычислений)
        let func = setup_func(CLOSED_YAML);
        assert_eq!(func.eval(-1.0), None);
        assert_eq!(func.eval(0.5), None);
        assert_eq!(func.eval(100.0), None);
    }
    #[test]
    fn test_numeric_values() {
        let yaml = "
            0.0..10.0: 5
            10.0..20.0: 1.5
        ";
        let func = setup_func(yaml);
        match func.eval(5.0) {
            Some(Point::Int(val)) => assert_eq!(val.value, 5),
            other => panic!("Expected Point::Int(5), got: {:?}", other),
        }
        match func.eval(15.0) {
            Some(Point::Double(val)) => assert_eq!(val.value, 1.5),
            other => panic!("Expected Point::Double(1.5), got: {:?}", other),
        }
    }
    #[test]
    fn test_invalid_range_format_rejected() {
        let yaml = "
            0.0..1.0: 'ok'
            abc: 'bad'
        ";
        let conf: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        assert!(PiecewiseStep::try_new("test", 1, &conf).is_err());
    }
    #[test]
    fn test_unbounded_range_rejected() {
        let yaml = "
            ..: 'bad'
        ";
        let conf: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        assert!(PiecewiseStep::try_new("test", 1, &conf).is_err());
    }
    #[test]
    fn test_equal_bounds_rejected() {
        let yaml = "
            0.15..0.15: 'bad'
        ";
        let conf: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        assert!(PiecewiseStep::try_new("test", 1, &conf).is_err());
    }
    #[test]
    fn test_null_value_rejected() {
        let yaml = "
            0.0..1.0:
        ";
        let conf: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        assert!(PiecewiseStep::try_new("test", 1, &conf).is_err());
    }

    ///
    /// Мок вышестоящего узла для управления выдачей данных в поток
    #[derive(Debug)]
    struct MockOut {
        next_value: Option<FnResult<FnFlow, String>>,
    }
    impl FnOut for MockOut {
        fn id(&self) -> String { "mock".to_string() }
        fn kind(&self) -> FnKind { FnKind::Fn }
        fn inputs(&self) -> Vec<String> { vec![] }
        fn out(&mut self) -> FnResult<FnFlow, String> {
            self.next_value.clone().unwrap_or(Ok(None))
        }
        fn hard_reset(&mut self) {}
        fn reset(&mut self) {}
    }

    fn setup_fn(yaml: &str, input: Point) -> FnPiecewiseStep {
        let mock = Rc::new(RefCell::new(MockOut { next_value: Some(Ok(Some(FnFlow::New(input)))) }));
        let conf: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        FnPiecewiseStep::new("test_parent", 1, mock, &conf).unwrap()
    }

    #[test]
    fn test_out_maps_value_and_renames_point() {
        let input = Point::Real(PointHlr::new(1, "AI_01", 0.10, Status::Ok, Cot::Inf, chrono::Utc::now()));
        let mut f = setup_fn(STEPS_YAML, input);
        let res = f.out().unwrap().unwrap();
        match res {
            FnFlow::New(Point::String(hlr)) => {
                assert_eq!(hlr.value, "0_05-0_15");
                // Имя выходной точки = id функции (как у FnPiecewiseLinear)
                assert!(hlr.name.contains("/FnPiecewiseStep"));
            }
            other => panic!("Expected FnFlow::New(Point::String), got: {:?}", other),
        }
    }
    #[test]
    fn test_out_parses_string_input() {
        let input = Point::String(PointHlr::new(1, "AI_01", "0.20".to_string(), Status::Ok, Cot::Inf, chrono::Utc::now()));
        let mut f = setup_fn(STEPS_YAML, input);
        let res = f.out().unwrap().unwrap();
        match res {
            FnFlow::New(Point::String(hlr)) => assert_eq!(hlr.value, "0_15-0_25"),
            other => panic!("Expected FnFlow::New(Point::String), got: {:?}", other),
        }
    }
    #[test]
    fn test_out_returns_none_in_gap() {
        let input = Point::Real(PointHlr::new(1, "AI_01", 0.5, Status::Ok, Cot::Inf, chrono::Utc::now()));
        let mut f = setup_fn(CLOSED_YAML, input);
        assert!(f.out().unwrap().is_none());
    }
    #[test]
    fn test_out_nan_input_error() {
        let input = Point::Double(PointHlr::new(1, "AI_01", f64::NAN, Status::Ok, Cot::Inf, chrono::Utc::now()));
        let mut f = setup_fn(STEPS_YAML, input);
        assert!(f.out().is_err());
    }
    #[test]
    fn test_out_invalid_string_input_error() {
        let input = Point::String(PointHlr::new(1, "AI_01", "abc".to_string(), Status::Ok, Cot::Inf, chrono::Utc::now()));
        let mut f = setup_fn(STEPS_YAML, input);
        assert!(f.out().is_err());
    }
}