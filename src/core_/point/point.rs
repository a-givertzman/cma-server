use std::fmt::Debug;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use testing::entities::test_value::Value;
use crate::{
    conf::point_config::point_config_type::PointConfigType,
    core_::{cot::cot::Cot, status::status::Status, types::{bool::Bool, type_of::TypeOf}},
    services::multi_queue::subscription_criteria::SubscriptionCriteria,
};
use super::point_hlr::PointHlr;
///
///
pub trait ToPoint {
    fn to_point(&self, tx_id: usize, name: &str) -> Point;
}

impl ToPoint for bool {
    fn to_point(&self, tx_id: usize, name: &str) -> Point {
        Point::Bool(PointHlr::new_bool(tx_id, name, *self))
    }
}
impl ToPoint for i64 {
    fn to_point(&self, tx_id: usize, name: &str) -> Point {
        Point::Int(PointHlr::new_int(tx_id, name, *self))
    }
}
impl ToPoint for f32 {
    fn to_point(&self, tx_id: usize, name: &str) -> Point {
        Point::Real(PointHlr::new_real(tx_id, name, *self))
    }
}
impl ToPoint for f64 {
    fn to_point(&self, tx_id: usize, name: &str) -> Point {
        Point::Double(PointHlr::new_double(tx_id, name, *self))
    }
}
impl ToPoint for &str {
    fn to_point(&self, tx_id: usize, name: &str) -> Point {
        Point::String(PointHlr::new_string(tx_id, name, *self))
    }
}
impl ToPoint for String {
    fn to_point(&self, tx_id: usize, name: &str) -> Point {
        Point::String(PointHlr::new_string(tx_id, name, self))
    }
}

///
/// enum container for Point<T>
/// - supported types: Bool, Int, Real, Double, String
#[derive(Debug, Clone, PartialEq)]
pub enum Point {
    Bool(PointHlr<Bool>),
    Int(PointHlr<i64>),
    Real(PointHlr<f32>),
    Double(PointHlr<f64>),
    String(PointHlr<String>)
}
//
//
impl Point {
    ///
    /// Creates instance of PointType
    ///  - tx_id - identifier of the producer - service
    ///  - name - the name of the Point
    ///  - value - current value stored in the Point
    pub fn new<T: ToPoint>(tx_id: usize, name: &str, value: T) -> Self {
        value.to_point(tx_id, name)
    }
    ///
    /// Returns transmitter ID of the containing Point
    pub fn tx_id(&self) -> usize {
        match self {
            Point::Bool(point) => point.tx_id,
            Point::Int(point) => point.tx_id,
            Point::Real(point) => point.tx_id,
            Point::Double(point) => point.tx_id,
            Point::String(point) => point.tx_id,
        }
    }
    ///
    /// Returns type of the containing Point
    pub fn type_(&self) -> PointConfigType {
        match self {
            Point::Bool(_) => PointConfigType::Bool,
            Point::Int(_) => PointConfigType::Int,
            Point::Real(_) => PointConfigType::Real,
            Point::Double(_) => PointConfigType::Double,
            Point::String(_) => PointConfigType::String,
        }
    }
    ///
    /// Returns name of the containing Point
    pub fn name(&self) -> String {
        match self {
            Point::Bool(point) => point.name.clone(),
            Point::Int(point) => point.name.clone(),
            Point::Real(point) => point.name.clone(),
            Point::Double(point) => point.name.clone(),
            Point::String(point) => point.name.clone(),
        }
    }
    ///
    /// Returns destination of the containing Point
    pub fn dest(&self) -> String {
        match self {
            Point::Bool(point) => SubscriptionCriteria::dest(&point.cot, &point.name),    //concat_string!(point.cot, point.name),
            Point::Int(point) => SubscriptionCriteria::dest(&point.cot, &point.name),    //concat_string!(point.cot, point.name),
            Point::Real(point) => SubscriptionCriteria::dest(&point.cot, &point.name),    //concat_string!(point.cot, point.name),
            Point::Double(point) => SubscriptionCriteria::dest(&point.cot, &point.name),    //concat_string!(point.cot, point.name),
            Point::String(point) => SubscriptionCriteria::dest(&point.cot, &point.name),    //concat_string!(point.cot, point.name),
        }
    }
    ///
    /// Returns point.value wraped into the enum Value
    pub fn value(&self) -> Value {
        match self {
            Point::Bool(point) => Value::Bool(point.value.0),
            Point::Int(point) => Value::Int(point.value),
            Point::Real(point) => Value::Real(point.value),
            Point::Double(point) => Value::Double(point.value),
            Point::String(point) => Value::String(point.value.clone()),
        }
    }
    ///
    /// Returns containing Point<bool>
    pub fn as_bool(&self) -> PointHlr<Bool> {
        match self {
            Point::Bool(point) => point.clone(),
            _ => panic!("PointType.as_bool | Expected type 'Bool', but found '{:?}' point: '{}'", self.type_(), self.name()),
        }
    }
    ///
    /// Returns containing Point<bool>
    pub fn try_as_bool(&self) -> Result<PointHlr<Bool>, String> {
        match self {
            Point::Bool(point) => Ok(point.clone()),
            _ => Err(format!("PointType.try_as_bool | Expected type 'Bool', but found '{:?}' point: '{}'", self.type_(), self.name())),
        }
    }
    ///
    /// Returns containing Point<i64>
    pub fn as_int(&self) -> PointHlr<i64> {
        match self {
            Point::Int(point) => point.clone(),
            _ => panic!("PointType.as_int | Expected type 'Int', but found '{:?}' point: '{}'", self.type_(), self.name()),
        }
    }
    ///
    /// Returns containing Point<i64>
    pub fn try_as_int(&self) -> Result<PointHlr<i64>, String> {
        match self {
            Point::Int(point) => Ok(point.clone()),
            _ => Err(format!("PointType.try_as_int | Expected type 'Int', but found '{}' point: {}", self.type_of(), self.name())),
        }
    }
    ///
    /// Returns containing Point<f32>
    pub fn as_real(&self) -> PointHlr<f32> {
        match self {
            Point::Real(point) => point.clone(),
            _ => panic!("PointType.as_real | Expected type 'Real', but found '{:?}' point: '{}'", self.type_(), self.name()),
        }
    }
    ///
    /// Returns containing Point<f32>
    pub fn try_as_real(&self) -> Result<PointHlr<f32>, String> {
        match self {
            Point::Real(point) => Ok(point.clone()),
            _ => Err(format!("PointType.try_as_real | Expected type 'Real', but found '{:?}' point: '{}'", self.type_(), self.name())),
        }
    }
    ///
    /// Returns containing Point<f64>
    pub fn as_double(&self) -> PointHlr<f64> {
        match self {
            Point::Double(point) => point.clone(),
            _ => panic!("PointType.as_double | Expected type 'Double', but found '{:?}' point: '{}'", self.type_(), self.name()),
        }
    }
    ///
    /// Returns containing Point<f64>
    pub fn try_as_double(&self) -> Result<PointHlr<f64>, String> {
        match self {
            Point::Double(point) => Ok(point.clone()),
            _ => Err(format!("PointType.try_as_double | Expected type 'Double', but found '{:?}' point: '{}'", self.type_(), self.name())),
        }
    }
    ///
    /// Returns containing Point<String>
    pub fn as_string(&self) -> PointHlr<String> {
        match self {
            Point::String(point) => point.clone(),
            _ => panic!("PointType.as_string | Expected type 'String', but found '{:?}' point: '{}'", self.type_(), self.name()),
        }
    }
    ///
    /// Returns containing Point<String>
    pub fn try_as_string(&self) -> Result<PointHlr<String>, String> {
        match self {
            Point::String(point) => Ok(point.clone()),
            _ => Err(format!("PointType.try_as_string | Expected type 'String', but found '{:?}' point: '{}'", self.type_(), self.name())),
        }
    }
    ///
    /// Returns status of the containing Point
    pub fn status(&self) -> Status {
        match self {
            Point::Bool(point) => point.status,
            Point::Int(point) => point.status,
            Point::Real(point) => point.status,
            Point::Double(point) => point.status,
            Point::String(point) => point.status,
        }
    }
    ///
    /// Returns the cause & direction of the containing Point
    pub fn cot(&self) -> Cot {
        match self {
            Point::Bool(point) => point.cot,
            Point::Int(point) => point.cot,
            Point::Real(point) => point.cot,
            Point::Double(point) => point.cot,
            Point::String(point) => point.cot,
        }
    }
    ///
    /// Returns timestamp of the containing Point
    pub fn timestamp(&self) -> DateTime<chrono::Utc> {
        match self {
            Point::Bool(point) => point.timestamp,
            Point::Int(point) => point.timestamp,
            Point::Real(point) => point.timestamp,
            Point::Double(point) => point.timestamp,
            Point::String(point) => point.timestamp,
        }
    }
    ///
    /// Returns true if other.value == self.value
    pub fn cmp_value(&self, other: &Point) -> bool {
        match self {
            Point::Bool(point) => point.value == other.as_bool().value,
            Point::Int(point) => point.value == other.as_int().value,
            Point::Real(point) => point.value == other.as_real().value,
            Point::Double(point) => point.value == other.as_double().value,
            Point::String(point) => point.value == other.as_string().value,
        }
    }
    ///
    /// Returns Point converted to the Bool
    pub fn to_bool(&self) -> Self {
        let value = match self {
            Point::Bool(p) => p.value.0,
            Point::Int(p) => p.value > 0,
            Point::Real(p) => p.value > 0.0,
            Point::Double(p) => p.value > 0.0,
            // PointType::String(point) => panic!("{}.to_bool | Conversion to Bool for 'String' - is not supported", point.name),
            Point::String(p) => {
                match p.value.parse() {
                    Ok(value) => value,
                    Err(err) => {
                        panic!("PointType({}).to_bool | Error conversion into<bool> value: '{:?}'\n\terror: {:#?}", self.name(), self.value(), err);
                    }
                }
            }
            // _ => panic!("{}.to_bool | Conversion to Bool for '{}' - is not supported", self.name(),  self.type_of()),
        };
        Point::Bool(PointHlr::new(
            self.tx_id(),
            &self.name(),
            Bool(value),
            self.status(),
            self.cot(),
            self.timestamp(),
        ))
    }
    ///
    /// Returns Point converted to the Int
    pub fn to_int(&self) -> Self {
        let value = match self {
            Point::Bool(p) => if p.value.0 {1} else {0},
            Point::Int(p) => p.value,
            Point::Real(p) => p.value.round() as i64 ,
            Point::Double(p) => p.value.round() as i64,
            // PointType::String(p) => panic!("{}.to_int | Conversion to Int for 'String' - is not supported", p.name),
            Point::String(p) => match p.value.parse() {
                Ok(value) => value,
                Err(err) => {
                    panic!("PointType({}).to_int | Error conversion into<i64> value: {:?}\n\terror: {:#?}", self.name(), self.value(), err);
                }
            }
            // _ => panic!("{}.to_int | Conversion to Int for '{}' - is not supported", self.name(),  self.type_of()),
        };
        Point::Int(PointHlr::new(
            self.tx_id(),
            &self.name(),
            value,
            self.status(),
            self.cot(),
            self.timestamp(),
        ))
    }
    ///
    /// Returns Point converted to the Real
    pub fn to_real(&self) -> Self {
        let value = match self {
            Point::Bool(p) => if p.value.0 {1.0} else {0.0},
            Point::Int(p) => p.value as f32,
            Point::Real(p) => p.value,
            Point::Double(p) => p.value as f32,
            // PointType::String(p) => panic!("{}.to_real | Conversion to Real for 'String' - is not supported", p.name),
            Point::String(p) => match p.value.parse() {
                Ok(value) => value,
                Err(err) => {
                    panic!("PointType({}).to_real | Error conversion into<f32> value: {:?}\n\terror: {:#?}", self.name(), self.value(), err);
                }
            }
            // _ => panic!("{}.to_real | Conversion to Real for '{}' - is not supported", self.name(),  self.type_of()),
        };
        Point::Real(PointHlr::new(
            self.tx_id(),
            &self.name(),
            value,
            self.status(),
            self.cot(),
            self.timestamp(),
        ))
    }
    ///
    /// Returns Point converted to the Double
    pub fn to_double(&self) -> Self {
        let value = match self {
            Point::Bool(p) => if p.value.0 {1.0} else {0.0},
            Point::Int(p) => p.value as f64,
            Point::Real(p) => p.value as f64,
            Point::Double(p) => p.value,
            // PointType::String(p) => panic!("{}.to_double | Conversion to Double for 'String' - is not supported", p.name),
            Point::String(p) => match p.value.parse() {
                Ok(value) => value,
                Err(err) => {
                    panic!("PointType({}).to_double | Error conversion into<f64> value: {:?}\n\terror: {:#?}", self.name(), self.value(), err);
                }
            }
            // _ => panic!("{}.to_double | Conversion to Double for '{}' - is not supported", self.name(),  self.type_of()),
        };
        Point::Double(PointHlr::new(
            self.tx_id(),
            &self.name(),
            value,
            self.status(),
            self.cot(),
            self.timestamp(),
        ))
    }
    ///
    /// Returns Point converted to the String
    pub fn to_string(&self) -> Self {
        let value = match self {
            Point::Bool(p) => p.value.0.to_string(),
            Point::Int(p) => p.value.to_string(),
            Point::Real(p) => p.value.to_string(),
            Point::Double(p) => p.value.to_string(),
            Point::String(p) => p.value.to_owned(),
            // _ => panic!("{}.to_double | Conversion to Double for '{}' - is not supported", self.name(),  self.type_of()),
        };
        Point::String(PointHlr::new(
            self.tx_id(),
            &self.name(),
            value,
            self.status(),
            self.cot(),
            self.timestamp(),
        ))
    }
    ///
    /// Raises self to a [factor] power.
    pub fn pow(&self, exp: Self) -> Self {
        match &self {
            Point::Int(self_point) => {
                match exp {
                    Point::Int(exp) => Point::Int(self_point.pow(exp)),
                    Point::Real(exp) => Point::Int(self_point.to_real().pow(exp).to_int()),
                    Point::Double(exp) => Point::Int(self_point.to_double().pow(exp).to_int()),
                    _ => panic!("PointType.pow | Pow is not supported for 'exp' of type '{:?}'", self.type_()),
                }
            }
            Point::Real(self_point) => {
                match exp {
                    Point::Int(exp) => Point::Real(self_point.pow(exp.to_real())),
                    Point::Real(exp) => Point::Real(self_point.pow(exp)),
                    Point::Double(exp) => Point::Real(self_point.pow(exp.to_real())),
                    _ => panic!("PointType.pow | Pow is not supported for 'exp' of type '{:?}'", self.type_()),
                }
            }
            Point::Double(self_point) => {
                match exp {
                    Point::Int(exp) => Point::Double(self_point.pow(exp.to_double())),
                    Point::Real(exp) => Point::Double(self_point.pow(exp.to_double())),
                    Point::Double(exp) => Point::Double(self_point.pow(exp)),
                    _ => panic!("PointType.pow | Pow is not supported for 'exp' of type '{:?}'", self.type_()),
                }
            }
            _ => panic!("PointType.pow | Pow is not supported for type '{:?}'", self.type_()),
        }
    }
}
//
//
impl Serialize for Point {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        #[derive(Debug, Serialize)]
        struct PointSerialize<'a, T> {
            #[serde(rename = "type")]
            type_: &'a str,
            value: T,
            name: &'a str,
            status: u32,
            cot: Cot,
            timestamp: String,
        }
        match self {
            Point::Bool(point) => {
                PointSerialize {
                    type_: "Bool",
                    value: if point.value.0 {1} else {0},
                    name: &point.name,
                    status: Into::<u32>::into(point.status),
                    cot: point.cot,
                    timestamp: point.timestamp.to_rfc3339(),
                }.serialize(serializer)
            }
            Point::Int(point) => {
                PointSerialize {
                    type_: "Int",
                    value: &point.value,
                    name: &point.name,
                    status: Into::<u32>::into(point.status),
                    cot: point.cot,
                    timestamp: point.timestamp.to_rfc3339(),
                }.serialize(serializer)
            }
            Point::Real(point) => {
                PointSerialize {
                    type_: "Real",
                    value: point.value,
                    name: &point.name,
                    status: Into::<u32>::into(point.status),
                    cot: point.cot,
                    timestamp: point.timestamp.to_rfc3339(),
                }.serialize(serializer)
            }
            Point::Double(point) => {
                PointSerialize {
                    type_: "Double",
                    value: &point.value,
                    name: &point.name,
                    status: Into::<u32>::into(point.status),
                    cot: point.cot,
                    timestamp: point.timestamp.to_rfc3339(),
                }.serialize(serializer)
            }
            Point::String(point) => {
                PointSerialize {
                    type_: "String",
                    value: &point.value,
                    name: &point.name,
                    status: Into::<u32>::into(point.status),
                    cot: point.cot,
                    timestamp: point.timestamp.to_rfc3339(),
                }.serialize(serializer)
            }
        }
    }
}
//
//
impl<'de> Deserialize<'de> for Point {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        #[derive(Debug, Deserialize)]
        struct PointDeserialize {
            #[serde(alias = "type")]
            pub type_: PointConfigType,
            pub value: serde_json::Value,
            pub name: String,
            pub status: i64,  // Status,
            pub cot: Cot,
            pub timestamp: String    //DateTime<chrono::Utc>,
        }
        let tx_id = 0;
        let visitor = PointDeserialize::deserialize(deserializer)?;
        fn value_parsing_error<'de, D>(type_: &str, visitor: &PointDeserialize, err: impl Debug) -> D::Error where D: serde::Deserializer<'de>{
            serde::de::Error::custom(format!("PointType.deserialize | Error parsing {} value from {:#?}, \n\terror: {:#?}", type_, visitor, err))
        }
        fn timestamp_parsing_error<'de, D>(type_: &str, visitor: &PointDeserialize, err: impl Debug) -> D::Error where D: serde::Deserializer<'de>{
            serde::de::Error::custom(format!("PointType.deserialize | Error parsing {} timestamp from {:#?}, \n\terror: {:#?}", type_, visitor, err))
        }
        match visitor.type_ {
            PointConfigType::Bool => {
                let value = visitor.value.as_i64().ok_or_else(|| value_parsing_error::<D>("Point<Bool>", &visitor, "err"))?;
                Ok(Point::Bool(PointHlr::new(
                    tx_id,
                    &visitor.name,
                    Bool(value > 0),
                    Status::from(visitor.status),
                    visitor.cot,
                    visitor.timestamp.parse().map_err(|err| timestamp_parsing_error::<D>("Point<Bool>", &visitor, err))?,
                )))
            }
            PointConfigType::Int => {
                let value = visitor.value.as_i64().ok_or_else(|| value_parsing_error::<D>("Point<Int>", &visitor, "err"))?;
                Ok(Point::Int(PointHlr::new(
                    tx_id,
                    &visitor.name,
                    value,
                    Status::from(visitor.status),
                    visitor.cot,
                    visitor.timestamp.parse().map_err(|err| timestamp_parsing_error::<D>("Point<Int>", &visitor, err))?,
                )))
            }
            PointConfigType::Real => {
                let value = visitor.value.as_f64().ok_or_else(|| value_parsing_error::<D>("Point<Real>", &visitor, "err"))?;
                Ok(Point::Real(PointHlr::new(
                    tx_id,
                    &visitor.name,
                    value as f32,
                    Status::from(visitor.status),
                    visitor.cot,
                    visitor.timestamp.parse().map_err(|err| timestamp_parsing_error::<D>("Point<Real>", &visitor, err))?,
                )))
            }
            PointConfigType::Double => {
                let value = visitor.value.as_f64().ok_or_else(|| value_parsing_error::<D>("Point<Double>", &visitor, "err"))?;
                Ok(Point::Double(PointHlr::new(
                    tx_id,
                    &visitor.name,
                    value,
                    Status::from(visitor.status),
                    visitor.cot,
                    visitor.timestamp.parse().map_err(|err| timestamp_parsing_error::<D>("Point<Double>", &visitor, err))?,
                )))
            }
            PointConfigType::String => {
                Ok(Point::String(PointHlr::new(
                    tx_id,
                    &visitor.name,
                    visitor.value.as_str().unwrap().to_owned(),
                    Status::from(visitor.status),
                    visitor.cot,
                    visitor.timestamp.parse().map_err(|err| timestamp_parsing_error::<D>("Point<String>", &visitor, err))?,
                )))
            }
            PointConfigType::Json => {
                Err(serde::de::Error::custom("PointType.deserialize | Error parsing Point<Json> - Not implemented yet"))
                // Ok(PointType::String(Point::new(
                //     tx_id,
                //     &visitor.name,
                //     visitor.value.clone(),
                //     Status::from(visitor.status),
                //     visitor.cot,
                //     visitor.timestamp.parse().map_err(|err| value_parsing_timestamp::<D>("Point<Json>", &visitor, err))?,
                // )))
            }
        }
    }
}
//
//
impl std::ops::Add for Point {
    type Output = Point;
    fn add(self, rhs: Self) -> Self::Output {
        assert_eq!(self.type_(), rhs.type_(), "PointType.add | Incopitable types self: '{:?}' and other: '{:?}'\tin '{}'", self.type_(), rhs.type_(), self.name());
        match self {
            Point::Bool(self_point) => {
                Point::Bool(self_point + rhs.as_bool())
            }
            Point::Int(self_point) => {
                Point::Int(self_point + rhs.as_int())
            }
            Point::Real(self_point) => {
                Point::Real(self_point + rhs.as_real())
            }
            Point::Double(self_point) => {
                Point::Double(self_point + rhs.as_double())
            }
            _ => panic!("PointType.add | Add is not supported for type '{:?}'", self.type_()),
        }
    }
}
//
//
impl std::ops::Sub for Point {
    type Output = Point;
    fn sub(self, rhs: Self) -> Self::Output {
        assert_eq!(self.type_(), rhs.type_(), "PointType.sub | Incopitable types self: '{:?}' and other: '{:?}'\tin '{}'", self.type_(), rhs.type_(), self.name());
        match self {
            Point::Int(self_point) => {
                Point::Int(self_point - rhs.as_int())
            }
            Point::Real(self_point) => {
                Point::Real(self_point - rhs.as_real())
            }
            Point::Double(self_point) => {
                Point::Double(self_point - rhs.as_double())
            }
            _ => panic!("PointType.sub | Sub is not supported for type '{:?}'", self.type_()),
        }
    }
}
//
//
impl std::ops::Mul for Point {
    type Output = Point;
    fn mul(self, rhs: Self) -> Self::Output {
        assert_eq!(self.type_(), rhs.type_(), "PointType.mul | Incopitable types self: '{:?}' and other: '{:?}'\tin '{}'", self.type_(), rhs.type_(), self.name());
        match self {
            Point::Bool(self_point) => {
                Point::Bool(self_point * rhs.as_bool())
            }
            Point::Int(self_point) => {
                Point::Int(self_point * rhs.as_int())
            }
            Point::Real(self_point) => {
                Point::Real(self_point * rhs.as_real())
            }
            Point::Double(self_point) => {
                Point::Double(self_point * rhs.as_double())
            }
            _ => panic!("PointType.mul | Mul is not supported for type '{:?}'", self.type_()),
        }
    }
}
//
//
impl std::ops::Div for Point {
    type Output = Point;
    fn div(self, rhs: Self) -> Self::Output {
        assert_eq!(self.type_(), rhs.type_(), "PointType.div | Incopitable types self: '{:?}' and other: '{:?}'\tin '{}'", self.type_(), rhs.type_(), self.name());
        match self {
            Point::Int(self_point) => {
                Point::Int(self_point / rhs.as_int())
            }
            Point::Real(self_point) => {
                Point::Real(self_point / rhs.as_real())
            }
            Point::Double(self_point) => {
                Point::Double(self_point / rhs.as_double())
            }
            _ => panic!("PointType.div | Div is not supported for type '{:?}'", self.type_()),
        }
    }
}
//
//
impl std::cmp::PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        assert_eq!(self.type_(), other.type_(), "PointType.partial_cmp | Incopitable types self: '{:?}' and other: '{:?}'\tin '{}'", self.type_(), other.type_(), self.name());
        match self {
            Point::Bool(self_point) => {
                self_point.partial_cmp(&other.as_bool())
            }
            Point::Int(self_point) => {
                self_point.partial_cmp(&other.as_int())
            }
            Point::Real(self_point) => {
                self_point.partial_cmp(&other.as_real())
            }
            Point::Double(self_point) => {
                self_point.partial_cmp(&other.as_double())
            }
            Point::String(self_point) => {
                self_point.partial_cmp(&other.as_string())
            }
            // _ => panic!("PointType.partial_cmp | Not supported for type '{:?}'", self.type_of()),
        }
    }
}
