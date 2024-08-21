use std::fmt::Debug;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use testing::entities::test_value::Value;
use crate::{
    conf::point_config::point_config_type::PointConfigType,
    core_::{cot::cot::Cot, status::status::Status, types::{bool::Bool, type_of::TypeOf}},
    services::multi_queue::subscription_criteria::SubscriptionCriteria,
};
use super::point::Point;
///
///
pub trait ToPoint {
    fn to_point(&self, tx_id: usize, name: &str) -> PointType;
}

impl ToPoint for bool {
    fn to_point(&self, tx_id: usize, name: &str) -> PointType {
        PointType::Bool(Point::new_bool(tx_id, name, *self))
    }
}
impl ToPoint for i64 {
    fn to_point(&self, tx_id: usize, name: &str) -> PointType {
        PointType::Int(Point::new_int(tx_id, name, *self))
    }
}
impl ToPoint for f32 {
    fn to_point(&self, tx_id: usize, name: &str) -> PointType {
        PointType::Real(Point::new_real(tx_id, name, *self))
    }
}
impl ToPoint for f64 {
    fn to_point(&self, tx_id: usize, name: &str) -> PointType {
        PointType::Double(Point::new_double(tx_id, name, *self))
    }
}
impl ToPoint for &str {
    fn to_point(&self, tx_id: usize, name: &str) -> PointType {
        PointType::String(Point::new_string(tx_id, name, *self))
    }
}
impl ToPoint for String {
    fn to_point(&self, tx_id: usize, name: &str) -> PointType {
        PointType::String(Point::new_string(tx_id, name, self))
    }
}

///
/// enum container for Point<T>
/// - supported types: Bool, Int, Real, Double, String
#[derive(Debug, Clone, PartialEq)]
pub enum PointType {
    Bool(Point<Bool>),
    Int(Point<i64>),
    Real(Point<f32>),
    Double(Point<f64>),
    String(Point<String>)
}
//
//
impl PointType {
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
            PointType::Bool(point) => point.tx_id,
            PointType::Int(point) => point.tx_id,
            PointType::Real(point) => point.tx_id,
            PointType::Double(point) => point.tx_id,
            PointType::String(point) => point.tx_id,
        }
    }
    ///
    /// Returns type of the containing Point
    pub fn type_(&self) -> PointConfigType {
        match self {
            PointType::Bool(_) => PointConfigType::Bool,
            PointType::Int(_) => PointConfigType::Int,
            PointType::Real(_) => PointConfigType::Real,
            PointType::Double(_) => PointConfigType::Double,
            PointType::String(_) => PointConfigType::String,
        }
    }
    ///
    /// Returns name of the containing Point
    pub fn name(&self) -> String {
        match self {
            PointType::Bool(point) => point.name.clone(),
            PointType::Int(point) => point.name.clone(),
            PointType::Real(point) => point.name.clone(),
            PointType::Double(point) => point.name.clone(),
            PointType::String(point) => point.name.clone(),
        }
    }
    ///
    /// Returns destination of the containing Point
    pub fn dest(&self) -> String {
        match self {
            PointType::Bool(point) => SubscriptionCriteria::dest(&point.cot, &point.name),    //concat_string!(point.cot, point.name),
            PointType::Int(point) => SubscriptionCriteria::dest(&point.cot, &point.name),    //concat_string!(point.cot, point.name),
            PointType::Real(point) => SubscriptionCriteria::dest(&point.cot, &point.name),    //concat_string!(point.cot, point.name),
            PointType::Double(point) => SubscriptionCriteria::dest(&point.cot, &point.name),    //concat_string!(point.cot, point.name),
            PointType::String(point) => SubscriptionCriteria::dest(&point.cot, &point.name),    //concat_string!(point.cot, point.name),
        }
    }
    ///
    /// Returns point.value wraped into the enum Value
    pub fn value(&self) -> Value {
        match self {
            PointType::Bool(point) => Value::Bool(point.value.0),
            PointType::Int(point) => Value::Int(point.value),
            PointType::Real(point) => Value::Real(point.value),
            PointType::Double(point) => Value::Double(point.value),
            PointType::String(point) => Value::String(point.value.clone()),
        }
    }
    ///
    /// Returns containing Point<bool>
    pub fn as_bool(&self) -> Point<Bool> {
        match self {
            PointType::Bool(point) => point.clone(),
            _ => panic!("PointType.as_bool | Expected type 'Bool', but found '{:?}' point: '{}'", self.type_(), self.name()),
        }
    }
    ///
    /// Returns containing Point<bool>
    pub fn try_as_bool(&self) -> Result<Point<Bool>, String> {
        match self {
            PointType::Bool(point) => Ok(point.clone()),
            _ => Err(format!("PointType.try_as_bool | Expected type 'Bool', but found '{:?}' point: '{}'", self.type_(), self.name())),
        }
    }
    ///
    /// Returns containing Point<i64>
    pub fn as_int(&self) -> Point<i64> {
        match self {
            PointType::Int(point) => point.clone(),
            _ => panic!("PointType.as_int | Expected type 'Int', but found '{:?}' point: '{}'", self.type_(), self.name()),
        }
    }
    ///
    /// Returns containing Point<i64>
    pub fn try_as_int(&self) -> Result<Point<i64>, String> {
        match self {
            PointType::Int(point) => Ok(point.clone()),
            _ => Err(format!("PointType.try_as_int | Expected type 'Int', but found '{}' point: {}", self.type_of(), self.name())),
        }
    }
    ///
    /// Returns containing Point<f32>
    pub fn as_real(&self) -> Point<f32> {
        match self {
            PointType::Real(point) => point.clone(),
            _ => panic!("PointType.as_real | Expected type 'Real', but found '{:?}' point: '{}'", self.type_(), self.name()),
        }
    }
    ///
    /// Returns containing Point<f32>
    pub fn try_as_real(&self) -> Result<Point<f32>, String> {
        match self {
            PointType::Real(point) => Ok(point.clone()),
            _ => Err(format!("PointType.try_as_real | Expected type 'Real', but found '{:?}' point: '{}'", self.type_(), self.name())),
        }
    }
    ///
    /// Returns containing Point<f64>
    pub fn as_double(&self) -> Point<f64> {
        match self {
            PointType::Double(point) => point.clone(),
            _ => panic!("PointType.as_double | Expected type 'Double', but found '{:?}' point: '{}'", self.type_(), self.name()),
        }
    }
    ///
    /// Returns containing Point<f64>
    pub fn try_as_double(&self) -> Result<Point<f64>, String> {
        match self {
            PointType::Double(point) => Ok(point.clone()),
            _ => Err(format!("PointType.try_as_double | Expected type 'Double', but found '{:?}' point: '{}'", self.type_(), self.name())),
        }
    }
    ///
    /// Returns containing Point<String>
    pub fn as_string(&self) -> Point<String> {
        match self {
            PointType::String(point) => point.clone(),
            _ => panic!("PointType.as_string | Expected type 'String', but found '{:?}' point: '{}'", self.type_(), self.name()),
        }
    }
    ///
    /// Returns containing Point<String>
    pub fn try_as_string(&self) -> Result<Point<String>, String> {
        match self {
            PointType::String(point) => Ok(point.clone()),
            _ => Err(format!("PointType.try_as_string | Expected type 'String', but found '{:?}' point: '{}'", self.type_(), self.name())),
        }
    }
    ///
    /// Returns status of the containing Point
    pub fn status(&self) -> Status {
        match self {
            PointType::Bool(point) => point.status,
            PointType::Int(point) => point.status,
            PointType::Real(point) => point.status,
            PointType::Double(point) => point.status,
            PointType::String(point) => point.status,
        }
    }
    ///
    /// Returns the cause & direction of the containing Point
    pub fn cot(&self) -> Cot {
        match self {
            PointType::Bool(point) => point.cot,
            PointType::Int(point) => point.cot,
            PointType::Real(point) => point.cot,
            PointType::Double(point) => point.cot,
            PointType::String(point) => point.cot,
        }
    }
    ///
    /// Returns timestamp of the containing Point
    pub fn timestamp(&self) -> DateTime<chrono::Utc> {
        match self {
            PointType::Bool(point) => point.timestamp,
            PointType::Int(point) => point.timestamp,
            PointType::Real(point) => point.timestamp,
            PointType::Double(point) => point.timestamp,
            PointType::String(point) => point.timestamp,
        }
    }
    ///
    /// Returns true if other.value == self.value
    pub fn cmp_value(&self, other: &PointType) -> bool {
        match self {
            PointType::Bool(point) => point.value == other.as_bool().value,
            PointType::Int(point) => point.value == other.as_int().value,
            PointType::Real(point) => point.value == other.as_real().value,
            PointType::Double(point) => point.value == other.as_double().value,
            PointType::String(point) => point.value == other.as_string().value,
        }
    }
    ///
    /// Returns Point converted to the Bool
    pub fn to_bool(&self) -> Self {
        let value = match self {
            PointType::Bool(p) => p.value.0,
            PointType::Int(p) => p.value > 0,
            PointType::Real(p) => p.value > 0.0,
            PointType::Double(p) => p.value > 0.0,
            // PointType::String(point) => panic!("{}.to_bool | Conversion to Bool for 'String' - is not supported", point.name),
            PointType::String(p) => {
                match p.value.parse() {
                    Ok(value) => value,
                    Err(err) => {
                        panic!("PointType({}).to_bool | Error conversion into<bool> value: '{:?}'\n\terror: {:#?}", self.name(), self.value(), err);
                    }
                }
            }
            // _ => panic!("{}.to_bool | Conversion to Bool for '{}' - is not supported", self.name(),  self.type_of()),
        };
        PointType::Bool(Point::new(
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
            PointType::Bool(p) => if p.value.0 {1} else {0},
            PointType::Int(p) => p.value,
            PointType::Real(p) => p.value.round() as i64 ,
            PointType::Double(p) => p.value.round() as i64,
            // PointType::String(p) => panic!("{}.to_int | Conversion to Int for 'String' - is not supported", p.name),
            PointType::String(p) => match p.value.parse() {
                Ok(value) => value,
                Err(err) => {
                    panic!("PointType({}).to_int | Error conversion into<i64> value: {:?}\n\terror: {:#?}", self.name(), self.value(), err);
                }
            }
            // _ => panic!("{}.to_int | Conversion to Int for '{}' - is not supported", self.name(),  self.type_of()),
        };
        PointType::Int(Point::new(
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
            PointType::Bool(p) => if p.value.0 {1.0} else {0.0},
            PointType::Int(p) => p.value as f32,
            PointType::Real(p) => p.value,
            PointType::Double(p) => p.value as f32,
            // PointType::String(p) => panic!("{}.to_real | Conversion to Real for 'String' - is not supported", p.name),
            PointType::String(p) => match p.value.parse() {
                Ok(value) => value,
                Err(err) => {
                    panic!("PointType({}).to_real | Error conversion into<f32> value: {:?}\n\terror: {:#?}", self.name(), self.value(), err);
                }
            }
            // _ => panic!("{}.to_real | Conversion to Real for '{}' - is not supported", self.name(),  self.type_of()),
        };
        PointType::Real(Point::new(
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
            PointType::Bool(p) => if p.value.0 {1.0} else {0.0},
            PointType::Int(p) => p.value as f64,
            PointType::Real(p) => p.value as f64,
            PointType::Double(p) => p.value,
            // PointType::String(p) => panic!("{}.to_double | Conversion to Double for 'String' - is not supported", p.name),
            PointType::String(p) => match p.value.parse() {
                Ok(value) => value,
                Err(err) => {
                    panic!("PointType({}).to_double | Error conversion into<f64> value: {:?}\n\terror: {:#?}", self.name(), self.value(), err);
                }
            }
            // _ => panic!("{}.to_double | Conversion to Double for '{}' - is not supported", self.name(),  self.type_of()),
        };
        PointType::Double(Point::new(
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
            PointType::Bool(p) => p.value.0.to_string(),
            PointType::Int(p) => p.value.to_string(),
            PointType::Real(p) => p.value.to_string(),
            PointType::Double(p) => p.value.to_string(),
            PointType::String(p) => p.value.to_owned(),
            // _ => panic!("{}.to_double | Conversion to Double for '{}' - is not supported", self.name(),  self.type_of()),
        };
        PointType::String(Point::new(
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
            PointType::Int(self_point) => {
                match exp {
                    PointType::Int(exp) => PointType::Int(self_point.pow(exp)),
                    PointType::Real(exp) => PointType::Int(self_point.to_real().pow(exp).to_int()),
                    PointType::Double(exp) => PointType::Int(self_point.to_double().pow(exp).to_int()),
                    _ => panic!("PointType.pow | Pow is not supported for 'exp' of type '{:?}'", self.type_()),
                }
            }
            PointType::Real(self_point) => {
                match exp {
                    PointType::Int(exp) => PointType::Real(self_point.pow(exp.to_real())),
                    PointType::Real(exp) => PointType::Real(self_point.pow(exp)),
                    PointType::Double(exp) => PointType::Real(self_point.pow(exp.to_real())),
                    _ => panic!("PointType.pow | Pow is not supported for 'exp' of type '{:?}'", self.type_()),
                }
            }
            PointType::Double(self_point) => {
                match exp {
                    PointType::Int(exp) => PointType::Double(self_point.pow(exp.to_double())),
                    PointType::Real(exp) => PointType::Double(self_point.pow(exp.to_double())),
                    PointType::Double(exp) => PointType::Double(self_point.pow(exp)),
                    _ => panic!("PointType.pow | Pow is not supported for 'exp' of type '{:?}'", self.type_()),
                }
            }
            _ => panic!("PointType.pow | Pow is not supported for type '{:?}'", self.type_()),
        }
    }
}
//
//
impl Serialize for PointType {
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
            PointType::Bool(point) => {
                PointSerialize {
                    type_: "Bool",
                    value: if point.value.0 {1} else {0},
                    name: &point.name,
                    status: Into::<u32>::into(point.status),
                    cot: point.cot,
                    timestamp: point.timestamp.to_rfc3339(),
                }.serialize(serializer)
            }
            PointType::Int(point) => {
                PointSerialize {
                    type_: "Int",
                    value: &point.value,
                    name: &point.name,
                    status: Into::<u32>::into(point.status),
                    cot: point.cot,
                    timestamp: point.timestamp.to_rfc3339(),
                }.serialize(serializer)
            }
            PointType::Real(point) => {
                PointSerialize {
                    type_: "Real",
                    value: point.value,
                    name: &point.name,
                    status: Into::<u32>::into(point.status),
                    cot: point.cot,
                    timestamp: point.timestamp.to_rfc3339(),
                }.serialize(serializer)
            }
            PointType::Double(point) => {
                PointSerialize {
                    type_: "Double",
                    value: &point.value,
                    name: &point.name,
                    status: Into::<u32>::into(point.status),
                    cot: point.cot,
                    timestamp: point.timestamp.to_rfc3339(),
                }.serialize(serializer)
            }
            PointType::String(point) => {
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
impl<'de> Deserialize<'de> for PointType {
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
                Ok(PointType::Bool(Point::new(
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
                Ok(PointType::Int(Point::new(
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
                Ok(PointType::Real(Point::new(
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
                Ok(PointType::Double(Point::new(
                    tx_id,
                    &visitor.name,
                    value,
                    Status::from(visitor.status),
                    visitor.cot,
                    visitor.timestamp.parse().map_err(|err| timestamp_parsing_error::<D>("Point<Double>", &visitor, err))?,
                )))
            }
            PointConfigType::String => {
                Ok(PointType::String(Point::new(
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
impl std::ops::Add for PointType {
    type Output = PointType;
    fn add(self, rhs: Self) -> Self::Output {
        assert_eq!(self.type_(), rhs.type_(), "PointType.add | Incopitable types self: '{:?}' and other: '{:?}'\tin '{}'", self.type_(), rhs.type_(), self.name());
        match self {
            PointType::Bool(self_point) => {
                PointType::Bool(self_point + rhs.as_bool())
            }
            PointType::Int(self_point) => {
                PointType::Int(self_point + rhs.as_int())
            }
            PointType::Real(self_point) => {
                PointType::Real(self_point + rhs.as_real())
            }
            PointType::Double(self_point) => {
                PointType::Double(self_point + rhs.as_double())
            }
            _ => panic!("PointType.add | Add is not supported for type '{:?}'", self.type_()),
        }
    }
}
//
//
impl std::ops::Sub for PointType {
    type Output = PointType;
    fn sub(self, rhs: Self) -> Self::Output {
        assert_eq!(self.type_(), rhs.type_(), "PointType.sub | Incopitable types self: '{:?}' and other: '{:?}'\tin '{}'", self.type_(), rhs.type_(), self.name());
        match self {
            PointType::Int(self_point) => {
                PointType::Int(self_point - rhs.as_int())
            }
            PointType::Real(self_point) => {
                PointType::Real(self_point - rhs.as_real())
            }
            PointType::Double(self_point) => {
                PointType::Double(self_point - rhs.as_double())
            }
            _ => panic!("PointType.sub | Sub is not supported for type '{:?}'", self.type_()),
        }
    }
}
//
//
impl std::ops::Mul for PointType {
    type Output = PointType;
    fn mul(self, rhs: Self) -> Self::Output {
        assert_eq!(self.type_(), rhs.type_(), "PointType.mul | Incopitable types self: '{:?}' and other: '{:?}'\tin '{}'", self.type_(), rhs.type_(), self.name());
        match self {
            PointType::Bool(self_point) => {
                PointType::Bool(self_point * rhs.as_bool())
            }
            PointType::Int(self_point) => {
                PointType::Int(self_point * rhs.as_int())
            }
            PointType::Real(self_point) => {
                PointType::Real(self_point * rhs.as_real())
            }
            PointType::Double(self_point) => {
                PointType::Double(self_point * rhs.as_double())
            }
            _ => panic!("PointType.mul | Mul is not supported for type '{:?}'", self.type_()),
        }
    }
}
//
//
impl std::ops::Div for PointType {
    type Output = PointType;
    fn div(self, rhs: Self) -> Self::Output {
        assert_eq!(self.type_(), rhs.type_(), "PointType.div | Incopitable types self: '{:?}' and other: '{:?}'\tin '{}'", self.type_(), rhs.type_(), self.name());
        match self {
            PointType::Int(self_point) => {
                PointType::Int(self_point / rhs.as_int())
            }
            PointType::Real(self_point) => {
                PointType::Real(self_point / rhs.as_real())
            }
            PointType::Double(self_point) => {
                PointType::Double(self_point / rhs.as_double())
            }
            _ => panic!("PointType.div | Div is not supported for type '{:?}'", self.type_()),
        }
    }
}
//
//
impl std::cmp::PartialOrd for PointType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        assert_eq!(self.type_(), other.type_(), "PointType.partial_cmp | Incopitable types self: '{:?}' and other: '{:?}'\tin '{}'", self.type_(), other.type_(), self.name());
        match self {
            PointType::Bool(self_point) => {
                self_point.partial_cmp(&other.as_bool())
            }
            PointType::Int(self_point) => {
                self_point.partial_cmp(&other.as_int())
            }
            PointType::Real(self_point) => {
                self_point.partial_cmp(&other.as_real())
            }
            PointType::Double(self_point) => {
                self_point.partial_cmp(&other.as_double())
            }
            PointType::String(self_point) => {
                self_point.partial_cmp(&other.as_string())
            }
            // _ => panic!("PointType.partial_cmp | Not supported for type '{:?}'", self.type_of()),
        }
    }
}
