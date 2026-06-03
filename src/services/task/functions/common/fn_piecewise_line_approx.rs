use sal_core::{dbg::Dbg, error::Error};
use sal_sync::services::{entity::{Point, PointHlr, PointType}, types::Bool};
use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use crate::{
    domain::FnOutRef,
    services::task::{FlowContext, FnFlow, functions::{
        FnKind, FnOut, FnResult
    }}
};
///
/// ### Function | Piecewise Linear Approximation (кусочно-линейная аппроксимация)
/// 
///  - bool: true -> 1, false -> 0
///  - real: 0.1 -> 0 | 0.5 -> 1 | 0.9 -> 1 | 1.1 -> 1
///  - string: try to parse int
#[derive(Debug)]
pub struct FnPiecewiseLineApprox {
    id: String,
    kind: FnKind,
    input: FnOutRef,
    piecewise_linear: PiecewiseLinear,
}
// 
impl FnPiecewiseLineApprox {
    ///
    /// Creates new instance of the FnPiecewiseLineApprox
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnOutRef, piecewise_linear: PiecewiseLinear) -> Self {
        let self_id = format!("{}/FnPiecewiseLineApprox{}", parent.into(), COUNT.fetch_add(1, Ordering::SeqCst));
        Self { 
            id: self_id,
            kind: FnKind::Fn,
            input,
            piecewise_linear,
        }
    }
    ///
    /// Возвращает `PointHlr` с обновленными `name` и `value`
    #[inline]
    fn point_with<T>(p: &Point, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(p.txid(), name, value, p.status(), p.cot(), p.timestamp())
    }
    ///
    /// Возвращает `Point` с обновленными `name` и `value` сохраняя тип
    #[inline]
    fn point(id: &str, input: &Point, val: f64) -> Result<Point, String> {
        match input.type_() {
            PointType::Bool => Ok(Point::Bool(Self::point_with(input, id, Bool(val != 0.0)))),
            PointType::Int => Ok(Point::Int(Self::point_with(input, id, val.round() as i64))),
            PointType::Real => Ok(Point::Real(Self::point_with(input, id, val as f32))),
            PointType::Double => Ok(Point::Double(Self::point_with(input, id, val))),
            PointType::String => Ok(Point::String(Self::point_with(input, id, val.to_string()))),
            _ => Err(concat_string!(id, ".out | Invalid input type '", input.type_().to_string(), "'")),
        }
    }
}
//
// 
impl FnOut for FnPiecewiseLineApprox { 
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
            _ => return Err(concat_string!(self.id, ".out | Invalid input type '", input.type_().to_string(), "'")),
        };
        let Some(val) = self.piecewise_linear.eval(value) else { return Ok(None) };
            // .ok_or(concat_string!(self.id, ".out | Invalid input type '", input.type_().to_string(), "'"))?;
        let out = Self::point(&self.id, &input, val)?;
        log::trace!("{}.out | out: {:?}", self.id, &out);
        flow.wrap(out)
    }
    //
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
    }
}
///
/// Global static counter of FnPiecewiseLineApprox instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Contains x & y of the point on the line
#[derive(Debug, Copy, Clone)]
struct LinePoint {
    x: f64, y: f64
}
impl LinePoint {
    #[inline]
    fn new(x: f64, y: f64) -> Self {
        Self {x, y}
    }
}
///
/// Linear approximation between two points
#[derive(Debug)]
struct LinearApprox {
    left: LinePoint,
    right: LinePoint,
    k: f64,
    b: f64,
}
impl LinearApprox {
    ///
    /// Creates new instance of the LinearApprox based on two points
    /// - `is_last` - Means the piece is last in the piecwise collection
    /// - Returns Err if points are vertical (has same `x`)
    fn try_new(p1: &LinePoint, p2: &LinePoint) -> Result<Self, Error> {
        if (p2.x - p1.x).abs() <= f64::MIN_POSITIVE {
            return Err(Error::new("LinearApprox", "try_new")
                .err(format!("Can't create linear approximation: vertical line (x1 {} and x2 {} coordinates are equal)", p1.x, p2.x)));
        }
        let k = (p2.y - p1.y) / (p2.x - p1.x);
        if k.is_nan() || k.is_infinite() {
            return Err(Error::new("LinearApprox", "try_new")
                .err(format!("Can't create linear approximation, invalid factor k: {k}")));
        }
        let b = p1.y - p1.x * k;
        if b.is_nan() || b.is_infinite() {
            return Err(Error::new("LinearApprox", "try_new")
                .err(format!("Can't create linear approximation, invalid number b: {b}")));
        }
        Ok(Self {
            left: *p1,
            right: *p2,
            k,
            b,
        })
    }
    ///
    /// Returns linear approximation of the x between two points
    #[inline]
    fn eval(&self, x: f64) -> f64 {
        self.k * x + self.b
    }
    ///
    /// Returns `true` if `x` is out of the left
    #[inline]
    fn is_out_of_left(&self, x: f64) -> bool {
        x < self.left.x
    }
    ///
    /// Returns `true` if `x` is out of the right
    #[inline]
    fn is_out_of_right(&self, x: f64) -> bool {
        x > self.right.x
    }
}
///
/// The collection og the LinearApprox
#[derive(Debug)]
pub struct PiecewiseLinear {
    id: Dbg,
    lines: Vec<LinearApprox>,
}
impl PiecewiseLinear {
    const IS_EMPTY_MSG: &'static str = "Piecewise function must contains at least two points";
    ///
    /// Creates new instance of the [Linears]
    /// - `parent` - Parent entity identifier
    /// - `pairs` - Points of piecewise-line function
    pub fn try_new(parent: impl Into<String>, mut pairs: Vec<LinePoint>) -> Result<Self, Error> {
        let dbg = Dbg::new(parent, "Linears");
        let error = Error::new(&dbg, "try_new");
        if pairs.len() < 2 {
            return Err(error.err(Self::IS_EMPTY_MSG));
        }
        for p in &pairs {
            if p.x.is_nan() || p.y.is_nan() {
                return Err(error.err("Piecewise points contains NAN instead of numbers"));
            } 
        }
        pairs.sort_by(|a, b| a.x.total_cmp(&b.x));
        let has_duplicates = pairs.windows(2).any(|w| (w[0].x - w[1].x).abs() < f64::EPSILON);
        if has_duplicates {
            return Err(error.err(format!("Piecewise points contains duplicates in `x`")));
        }
        let mut lines = Vec::with_capacity(pairs.len() - 1);
        for pp in pairs.windows(2) {
            lines.push(
                LinearApprox::try_new(&pp[0], &pp[1])
                    .map_err(|err| error.pass(err))?
            );
        }
        Ok(Self { id: dbg, lines })
    }
    ///
    /// ### Returns `Linears` parsed from YAML
    /// 
    /// Expected following YAML format:
    /// ```yaml
    /// x1: y1
    /// x2: y2
    /// x3: y3
    /// ...
    /// ```
    pub fn from_yaml(parent: impl Into<String>, points: serde_yaml::Value) -> Result<Self, Error> {
        let parent = parent.into();
        let dbg = Dbg::new(&parent, "Linears");
        let error = Error::new(&dbg, "from_yaml");
        let points: serde_yaml::Mapping = serde_yaml::from_value(points.clone())
            .map_err(|err| error.pass_with(format!("Can't parse points from {:?}", points), err.to_string()))?;
        // let points: Vec<(serde_yaml::Value, serde_yaml::Value)> = points.into_iter().collect();
        if points.len() < 2 {
            return Err(error.err(Self::IS_EMPTY_MSG));
        }
        let mut pairs = Vec::with_capacity(points.len());
        for (x, y) in points {
            let x = x.as_f64()
                .ok_or(error.err(format!("Can't parse {:?} as f64", x)))?;
            let y = y.as_f64()
                .ok_or(error.err(format!("Can't parse {:?} as f64", y)))?;
        pairs.push(LinePoint::new(x, y));
        }
        Self::try_new(parent, pairs)
    }
    ///
    /// ### Evaluates the piecewise linear function at point `x`.
    ///
    /// If `x` is less than the first point's `x`, returns the first point's `y`.
    /// If `x` is greater than the last point's `x`, returns the last point's `y`.
    /// Otherwise, finds the segment containing `x` and performs linear interpolation.
    ///
    /// **Examples**
    /// ```ignore
    /// let points = vec![LinePoint::new(0.0, 0.0), LinePoint::new(1.0, 1.0)];
    /// let func = PiecewiseLinear::try_new("test", points).unwrap();
    /// assert_eq!(func.eval(0.5), Some(0.5));
    /// assert_eq!(func.eval(-1.0), Some(0.0)); // extrapolation
    /// ```
    pub fn eval(&self, x: f64) -> Option<f64> {
        log::trace!("{}.line_approx | value: {:?}", self.id, x);
        // Проверка выхода за левую границу (экстраполяция/const)
        if let Some(first) = self.lines.first() {
            if first.is_out_of_left(x) {
                log::trace!("{}.line_approx | less then first: {:#?}", self.id, first);
                return Some(first.left.y);
            }
        }
        // Проверка выхода за правую границу (экстраполяция/const)
        if let Some(last) = self.lines.last() {
            if last.is_out_of_right(x) {
                log::trace!("{}.line_approx | Greater then last: {:#?}", self.id, last);
                return Some(last.right.y);
            }
        }
        self.lines.iter().find_map(|line| {
            if x >= line.left.x && x <= line.right.x {
                return Some(line.eval(x))
            }
            None
        })
        // let res = self.lines.binary_search_by(|probe| {
        //     if x < probe.left.x {
        //         std::cmp::Ordering::Greater
        //     } else if x >= probe.right.x {
        //         std::cmp::Ordering::Less
        //     } else {
        //         std::cmp::Ordering::Equal
        //     }
        // });
        // match res {
        //     Ok(i) => Some(self.lines[i].eval(x)),
        //     Err(_) => None
        // }
    }
}
///
/// `PiecewiseLinear` Basic Tests
#[cfg(test)]
mod piecewise_linear_tests {
    use super::*;
    // Вспомогательная функция для создания базовой функции: f(x) = x
    fn setup_simple_func() -> PiecewiseLinear {
        let points = vec![
            LinePoint::new(0.0, 0.0),
            LinePoint::new(1.0, 1.0),
            LinePoint::new(2.0, 2.0),
        ];
        PiecewiseLinear::try_new("test_parent", points).unwrap()
    }
    #[test]
    fn test_successful_interpolation() {
        let func = setup_simple_func();
        // Точки строго внутри интервалов
        assert_eq!(func.eval(0.5), Some(0.5));
        assert_eq!(func.eval(1.5), Some(1.5));
    }
    #[test]
    fn test_boundaries_and_edges() {
        let func = setup_simple_func();
        // Точные совпадения с узловыми точками (тест на баг бинарного поиска)
        assert_eq!(func.eval(0.0), Some(0.0));
        assert_eq!(func.eval(1.0), Some(1.0));
        assert_eq!(func.eval(2.0), Some(2.0));
    }
    #[test]
    fn test_extrapolation_limits() {
        let func = setup_simple_func();
        // Выход за левую границу -> возвращает y первой точки
        assert_eq!(func.eval(-10.0), Some(0.0));
        assert_eq!(func.eval(-0.0001), Some(0.0));
        // Выход за правую границу -> возвращает y последней точки
        assert_eq!(func.eval(10.0), Some(2.0));
        assert_eq!(func.eval(2.0001), Some(2.0));
    }
    #[test]
    fn test_insufficient_points() {
        // Меньше 2 точек — должна быть ошибка
        let points = vec![LinePoint::new(1.0, 1.0)];
        let result = PiecewiseLinear::try_new("test", points);
        assert!(result.is_err());
    }
    #[test]
    fn test_nan_coordinates_rejected() {
        // Проверка защиты от NaN в try_new
        let points = vec![LinePoint::new(0.0, 0.0), LinePoint::new(f64::NAN, 1.0)];
        assert!(PiecewiseLinear::try_new("test", points).is_err());
        let points = vec![LinePoint::new(0.0, f64::NAN), LinePoint::new(1.0, 1.0)];
        assert!(PiecewiseLinear::try_new("test", points).is_err());
    }
    #[test]
    fn test_duplicate_x_coordinates() {
        // Две точки с одинаковым X (вертикальная линия / дубликат)
        let points = vec![
            LinePoint::new(1.0, 5.0),
            LinePoint::new(1.0, 10.0),
        ];
        let result = PiecewiseLinear::try_new("test", points);
        assert!(result.is_err());
    }
    #[test]
    fn test_sorting_order_independence() {
        // Передаем точки не по порядку, проверяем автоматическую сортировку
        let points = vec![
            LinePoint::new(2.0, 20.0),
            LinePoint::new(0.0, 0.0),
            LinePoint::new(1.0, 10.0),
        ];
        let func = PiecewiseLinear::try_new("test", points).unwrap();
        // Проверяем, что интерполяция между отсортированными точками корректна
        assert_eq!(func.eval(0.5), Some(5.0));
        assert_eq!(func.eval(1.5), Some(15.0));
    }
    #[test]
    fn test_from_yaml_parsing() {
        // Имитируем структуру IndexMap из serde_yaml
        let yaml_str = "
          0.0: 0.0
          1.0: 10.0
          2.0: 20.0
        ";
        let points: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let func = PiecewiseLinear::from_yaml("test_yaml", points);
        assert!(func.is_ok()); // Должно распарситься успешно
        let valid_func = func.unwrap();
        assert_eq!(valid_func.eval(0.5), Some(5.0));
    }
    #[test]
    fn test_from_yaml_invalid_types() {
        // Ошибка: вместо числа передана строка в качестве Y
        let yaml_str = "
          0.0: 0.0
          1.0: 'invalid_value'
        ";
        let points: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let func = PiecewiseLinear::from_yaml("test_yaml", points);
        assert!(func.is_err());
    }
}
