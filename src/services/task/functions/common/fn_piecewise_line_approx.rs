use indexmap::IndexMap;
use sal_core::error::Error;
use sal_sync::services::{entity::{Point, PointHlr, PointType}, types::{Bool, TypeOf}};
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
    pieces: Linears,
}
// 
impl FnPiecewiseLineApprox {
    ///
    /// Creates new instance of the FnPiecewiseLineApprox
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnOutRef, pieces: Linears) -> Self {
        let self_id = format!("{}/FnPiecewiseLineApprox{}", parent.into(), COUNT.fetch_add(1, Ordering::SeqCst));
        Self { 
            id: self_id,
            kind: FnKind::Fn,
            input,
            pieces,
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
        let val = self.pieces.line_approx(value)?;
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
    fn new(x: f64, y: f64) -> Self {
        Self {x, y}
    }
}
///
/// Contains linear approximation between two points
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
    fn new(p1: LinePoint, p2: LinePoint) -> Self {
        let k = (p2.y - p1.y) / (p2.x - p1.x);
        let b = - p1.x * k + p1.y;
        Self {
            left: p1,
            right: p2,
            k,
            b,
        }
    }
    ///
    /// Returns linear approximation of the x between two points
    fn linear_approx(&self, x: f64) -> f64 {
        self.k * x + self.b
    }
    ///
    /// Returns true if x in between points
    /// left <= x < x
    fn contains(&self, x: f64) -> bool {
        self.left.x <= x && x < self.right.x
    } 
    ///
    /// Returns true if x is on the left of the left point
    fn is_less(&self, x: f64) -> bool {
        x < self.left.x
    }
    // ///
    // /// Returns true if x is on the right of the rigjt point
    // fn is_greater(&self, x: f64) -> bool {
    //     x >= self.right.x
    // }
}
///
/// The collection og the LinearApprox
#[derive(Debug)]
struct Linears {
    id: String,
    pieces: Vec<LinearApprox>,
}
impl Linears {
    const IS_EMPTY_MSG: &'static str = "Piecewise function must contains at least two points";
    ///
    /// Creates new instance of the [Linears]
    fn new(parent: impl Into<String>, pieces: &IndexMap<serde_yaml::Value, serde_yaml::Value>) -> Result<Self, Error> {
        let self_id = format!("{}/Linears", parent.into());
        assert!(pieces.len() > 1, "{}.line_approx | {}", self_id, Self::IS_EMPTY_MSG);
        let mut pieces_iter = pieces.iter();
        let mut pieces = vec![];
        match pieces_iter.next() {
            Some((x, y)) => {
                let x = Self::serde_value_to_f64(&self_id, x)?;
                let y = Self::serde_value_to_f64(&self_id, y)?;
                let mut p1 = LinePoint::new(x, y);
                while let Some((x, y)) = pieces_iter.next() {
                    let x = Self::serde_value_to_f64(&self_id, x)?;
                    let y = Self::serde_value_to_f64(&self_id, y)?;
                    let p2 = LinePoint::new(x, y);
                    pieces.push(LinearApprox::new(p1, p2));
                    p1 = p2;
                }
            }
            None => panic!("{}.line_approx | {}", self_id, Self::IS_EMPTY_MSG),
        }
        Ok(Self { id: self_id, pieces })
    }
    ///
    /// 
    fn line_approx(&self, value: f64) -> Result<f64, String> {
        log::trace!("{}.line_approx | value: {:?}", self.id, value);
        let mut pieces = self.pieces.iter();
        match pieces.next() {
            Some(first) => if first.is_less(value) {
                log::trace!("{}.line_approx | less first: {:#?}", self.id, first);
                return Ok(first.left.y);
            } else {
                if first.contains(value) {
                    log::trace!("{}.line_approx | first: {:#?}", self.id, first);
                    return Ok(first.linear_approx(value));
                }
                while let Some(piece) = pieces.next() {
                    if piece.contains(value) {
                        log::trace!("{}.line_approx | piece: {:#?}", self.id, piece);
                        return Ok(piece.linear_approx(value));
                    }
                }
                match self.pieces.last() {
                    Some(last) => Ok(last.right.y),
                    None => Err(format!("{}.line_approx | {}", self.id, Self::IS_EMPTY_MSG)),
                }
            }
            None => Err(format!("{}.line_approx | {}", self.id, Self::IS_EMPTY_MSG)),
        }
    }
    ///
    /// Extracts containing number as f64
    fn serde_value_to_f64(self_id: &str, value: &serde_yaml::Value) -> Result<f64, String> {
        if value.is_number() {
            value.as_f64().ok_or(format!("{}.serde_value_to_f64 | Piecewise function point type '{:?}' - is not supported", self_id, value.type_of()))
        } else {
            Err(format!("{}.serde_value_to_f64 | Piecewise function point type '{:?}' - is not supported", self_id, value.type_of()))
        }
    }
}
