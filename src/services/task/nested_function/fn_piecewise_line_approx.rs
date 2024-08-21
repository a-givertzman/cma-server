use indexmap::IndexMap;
use log::trace;
use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use crate::{
    conf::point_config::point_config_type::PointConfigType,
    core_::{point::{point::Point, point_type::PointType},
    types::{fn_in_out_ref::FnInOutRef, type_of::TypeOf}},
    services::task::nested_function::{
        fn_::{FnIn, FnInOut, FnOut},
        fn_kind::FnKind,
    }
};

use super::fn_result::FnResult;
///
/// Function | Piecewise Linear Approximation (кусочно-линейная аппроксимация)
///  - bool: true -> 1, false -> 0
///  - real: 0.1 -> 0 | 0.5 -> 1 | 0.9 -> 1 | 1.1 -> 1
///  - string: try to parse int
#[derive(Debug)]
pub struct FnPiecewiseLineApprox {
    id: String,
    kind: FnKind,
    input: FnInOutRef,
    pieces: Linears,
}
//
// 
impl FnPiecewiseLineApprox {
    ///
    /// Creates new instance of the FnPiecewiseLineApprox
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnInOutRef, pieces: IndexMap<serde_yaml::Value, serde_yaml::Value>) -> Self {
        let self_id = format!("{}/FnPiecewiseLineApprox{}", parent.into(), COUNT.fetch_add(1, Ordering::SeqCst));
        let pieces = Linears::new(&self_id, &pieces);
        Self { 
            id: self_id,
            kind: FnKind::Fn,
            input,
            pieces,
        }
    }
    ///
    /// Build an out Point deppending on the input type
    fn build_point(&self, input: &PointType, value: f64) -> PointType {
        match input.type_() {
            PointConfigType::Int => PointType::Int(
                Point::new(
                    input.tx_id(),
                    &concat_string!(self.id, ".out"),
                    value.round() as i64,
                    input.status(),
                    input.cot(),
                    input.timestamp(),
                )
            ),
            PointConfigType::Real => PointType::Real(
                Point::new(
                    input.tx_id(),
                    &concat_string!(self.id, ".out"),
                    value as f32,
                    input.status(),
                    input.cot(),
                    input.timestamp(),
                )
            ),
            PointConfigType::Double => PointType::Double(
                Point::new(
                    input.tx_id(),
                    &concat_string!(self.id, ".out"),
                    value,
                    input.status(),
                    input.cot(),
                    input.timestamp(),
                )
            ),
            _ => panic!("{}.line_approx | Input type '{:?}' - is not supported", self.id, input.type_()),
        }
    }
}
//
// 
impl FnIn for FnPiecewiseLineApprox {}
//
// 
impl FnOut for FnPiecewiseLineApprox { 
    //
    fn id(&self) -> String {
        self.id.clone()
    }
    //
    fn kind(&self) -> &FnKind {
        &self.kind
    }
    //
    fn inputs(&self) -> Vec<String> {
        self.input.borrow().inputs()
    }
    //
    //
    fn out(&mut self) -> FnResult<PointType, String> {
        let input = self.input.borrow_mut().out();
        trace!("{}.out | input: {:?}", self.id, input);
        match input {
            FnResult::Ok(input) => {
                let value = self.pieces.line_approx(input.to_double().as_double().value);
                let out = self.build_point(&input, value);
                trace!("{}.out | out: {:?}", self.id, &out);
                FnResult::Ok(out)
            }
            FnResult::None => FnResult::None,
            FnResult::Err(err) => FnResult::Err(err),
        }
    }
    //
    //
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
    }
}
//
// 
impl FnInOut for FnPiecewiseLineApprox {}
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
    fn new(parent: impl Into<String>, pieces: &IndexMap<serde_yaml::Value, serde_yaml::Value>) -> Self {
        let self_id = format!("{}/Linears", parent.into());
        assert!(pieces.len() > 1, "{}.line_approx | {}", self_id, Self::IS_EMPTY_MSG);
        let mut pieces_iter = pieces.iter();
        let mut pieces = vec![];
        match pieces_iter.next() {
            Some((x, y)) => {
                let x = Self::serde_value_to_f64(&self_id, x);
                let y = Self::serde_value_to_f64(&self_id, y);
                let mut p1 = LinePoint::new(x, y);
                while let Some((x, y)) = pieces_iter.next() {
                    let x = Self::serde_value_to_f64(&self_id, x);
                    let y = Self::serde_value_to_f64(&self_id, y);
                    let p2 = LinePoint::new(x, y);
                    pieces.push(LinearApprox::new(p1, p2));
                    p1 = p2;
                }
            }
            None => panic!("{}.line_approx | {}", self_id, Self::IS_EMPTY_MSG),
        }
        Self { id: self_id, pieces }
    }
    ///
    /// 
    fn line_approx(&self, value: f64) -> f64 {
        trace!("{}.line_approx | value: {:?}", self.id, value);
        let mut pieces = self.pieces.iter();
        match pieces.next() {
            Some(first) => if first.is_less(value) {
                trace!("{}.line_approx | less first: {:#?}", self.id, first);
                return first.left.y
            } else {
                if first.contains(value) {
                    trace!("{}.line_approx | first: {:#?}", self.id, first);
                    return first.linear_approx(value);
                }
                while let Some(piece) = pieces.next() {
                    if piece.contains(value) {
                        trace!("{}.line_approx | piece: {:#?}", self.id, piece);
                        return piece.linear_approx(value);
                    }
                }
                match self.pieces.last() {
                    Some(last) => {
                        return last.right.y
                    }
                    None => panic!("{}.line_approx | {}", self.id, Self::IS_EMPTY_MSG),
                }
            }
            None => panic!("{}.line_approx | {}", self.id, Self::IS_EMPTY_MSG),
        }
    }
    ///
    /// Extracts containing number as f64
    fn serde_value_to_f64(self_id: &str, value: &serde_yaml::Value) -> f64 {
        if value.is_number() {
            value.as_f64().unwrap_or_else(|| panic!("{}.serde_value_to_f64 | Piecewise function point type '{:?}' - is not supported", self_id, value.type_of()))
        } else {
            panic!("{}.serde_value_to_f64 | Piecewise function point type '{:?}' - is not supported", self_id, value.type_of())
        }
    }
}