use sal_sync::services::entity::{Cot, Point, Status};


pub(crate) trait TryTo<T>: Sized {
    /// The type returned in the event of a conversion error.
    type Error;
    /// Performs the conversion.
    fn try_to(self) -> Result<T, Self::Error>;
}

impl TryTo<bool> for &sal_sync::services::entity::Point {
    type Error = sal_core::error::Error;
    fn try_to(self) -> Result<bool, Self::Error> {
        match self.type_() {
            sal_sync::services::entity::PointType::Bool |
            sal_sync::services::entity::PointType::Int |
            sal_sync::services::entity::PointType::Real |
            sal_sync::services::entity::PointType::Double => Ok(self.to_bool().as_bool().value.0),
            _ => return Err(sal_core::error::Error::new("Point", "try_to<bool>").err(concat_string::concat_string!("Invalid type '", self.type_().to_string(), "'"))),
        }
    }
}
impl TryTo<i64> for &sal_sync::services::entity::Point {
    type Error = sal_core::error::Error;
    fn try_to(self) -> Result<i64, Self::Error> {
        match self.type_() {
            sal_sync::services::entity::PointType::Bool |
            sal_sync::services::entity::PointType::Int |
            sal_sync::services::entity::PointType::Real |
            sal_sync::services::entity::PointType::Double => Ok(self.to_int().as_int().value),
            _ => return Err(sal_core::error::Error::new("Point", "try_to<i64>").err(concat_string::concat_string!("Invalid type '", self.type_().to_string(), "'"))),
        }
    }
}
impl TryTo<f32> for &sal_sync::services::entity::Point {
    type Error = sal_core::error::Error;
    fn try_to(self) -> Result<f32, Self::Error> {
        match self.type_() {
            sal_sync::services::entity::PointType::Bool |
            sal_sync::services::entity::PointType::Int |
            sal_sync::services::entity::PointType::Real |
            sal_sync::services::entity::PointType::Double => Ok(self.to_real().as_real().value),
            _ => return Err(sal_core::error::Error::new("Point", "try_to<f32>").err(concat_string::concat_string!("Invalid type '", self.type_().to_string(), "'"))),
        }
    }
}
impl TryTo<f64> for &sal_sync::services::entity::Point {
    type Error = sal_core::error::Error;
    fn try_to(self) -> Result<f64, Self::Error> {
        match self.type_() {
            sal_sync::services::entity::PointType::Bool |
            sal_sync::services::entity::PointType::Int |
            sal_sync::services::entity::PointType::Real |
            sal_sync::services::entity::PointType::Double => Ok(self.to_double().as_double().value),
            _ => return Err(sal_core::error::Error::new("Point", "try_to<f64>").err(concat_string::concat_string!("Invalid type '", self.type_().to_string(), "'"))),
        }
    }
}
///
/// Container for `Point` meta
#[derive(Debug, Clone, Copy)]
pub struct PointMeta {
    pub status: Status,
    pub cot: Cot,
    pub ts: chrono::DateTime<chrono::Utc>,
}
//
impl PointMeta {
    ///
    /// Returns `PointMeta` updated with passed `Point`: `status`, `cot`, `timestamp`
    pub fn update(self, p: &Point) -> Self {
        Self {
            status: p.status(),
            cot: p.cot(),
            ts: p.timestamp(),
        }
    }
    ///
    /// Returns `PointMeta` updated with passed `Point`: `status`, `cot`, `timestamp` only if it has latest `timestamp`
    pub fn update_latest(self, p: &Point) -> Self {
        if p.timestamp() > self.ts {
            self.update(p)
        } else {
            self
        }
    }
    ///
    /// Returns `PointMeta` updated with most bad status
    pub fn update_status(mut self, p: &Point) -> Self {
        if p.status() > self.status {
            self.status = p.status();
        }
        self
    }
}
//
impl Default for PointMeta {
    fn default() -> Self {
        Self { status: Status::Ok, cot: Cot::Inf, ts: chrono::DateTime::<chrono::Utc>::MIN_UTC }
    }
}
///
/// Halper `Value` to simplify the math operations
#[derive(Debug, Clone, Copy)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Real(f32),
    Double(f64),
}
//
impl Value {
    pub fn is_nan(&self) -> bool {
        match self {
            Value::Bool(_) => false,
            Value::Int(_) => false,
            Value::Real(v) => v.is_nan(),
            Value::Double(v) => v.is_nan(),
        }
    }
    pub fn is_zero(&self) -> bool {
        match self {
            Value::Bool(v) => !v,
            Value::Int(v) => *v == 0,
            Value::Real(v) => *v == 0.0,
            Value::Double(v) => *v == 0.0,
        }
    }
}
//
impl std::ops::Sub for Value {
    type Output = Result<Value, String>;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.is_nan() | rhs.is_nan() {
            return Err(format!("Value.sub | Invalid input: `{:?} - {:?}`", self, rhs));
        }
        match (self, rhs) {
            (Value::Bool(v1), Value::Bool(v2)) => Ok(Value::Int(v1 as i64 - v2 as i64)),
            (Value::Bool(v1), Value::Int(v2)) => Ok(Value::Int(v1 as i64 - v2)),
            (Value::Bool(v1), Value::Real(v2)) => Ok(Value::Real((v1 as u8) as f32 - v2)),
            (Value::Bool(v1), Value::Double(v2)) => Ok(Value::Double((v1 as u8) as f64 - v2)),
            (Value::Int(v1), Value::Bool(v2)) => Ok(Value::Int(v1 - v2 as i64)),
            (Value::Int(v1), Value::Int(v2)) => Ok(Value::Int(v1 - v2)),
            (Value::Int(v1), Value::Real(v2)) => Ok(Value::Real(v1 as f32 - v2)),
            (Value::Int(v1), Value::Double(v2)) => Ok(Value::Double(v1 as f64 - v2)),
            (Value::Real(v1), Value::Bool(v2)) => Ok(Value::Real(v1 - (v2 as u8) as f32)),
            (Value::Real(v1), Value::Int(v2)) => Ok(Value::Real(v1 - v2 as f32)),
            (Value::Real(v1), Value::Real(v2)) => Ok(Value::Real(v1 - v2)),
            (Value::Real(v1), Value::Double(v2)) => Ok(Value::Double(v1 as f64 - v2)),
            (Value::Double(v1), Value::Bool(v2)) => Ok(Value::Double(v1 - (v2 as u8) as f64)),
            (Value::Double(v1), Value::Int(v2)) => Ok(Value::Double(v1 - v2 as f64)),
            (Value::Double(v1), Value::Real(v2)) => Ok(Value::Double(v1 - v2 as f64)),
            (Value::Double(v1), Value::Double(v2)) => Ok(Value::Double(v1 - v2)),
        }
    }
}
//
impl std::ops::Div for Value {
    type Output = Result<Value, String>;

    fn div(self, rhs: Self) -> Self::Output {
        if rhs.is_zero() {
            return Err(format!("Value.div | Divizion dy zero: `{:?} / {:?}`", self, rhs));
        }
        if self.is_nan() | rhs.is_nan() {
            return Err(format!("Value.div | Invalid input: `{:?} / {:?}`", self, rhs));
        }
        match (self, rhs) {
            (Value::Bool(v1), Value::Bool(v2)) => Ok(Value::Int(v1 as i64 / v2 as i64)),
            (Value::Bool(v1), Value::Int(v2)) => Ok(Value::Int(v1 as i64 / v2)),
            (Value::Bool(v1), Value::Real(v2)) => Ok(Value::Real((v1 as u8) as f32 / v2)),
            (Value::Bool(v1), Value::Double(v2)) => Ok(Value::Double((v1 as u8) as f64 / v2)),
            (Value::Int(v1), Value::Bool(v2)) => Ok(Value::Int(v1 / v2 as i64)),
            (Value::Int(v1), Value::Int(v2)) => Ok(Value::Int(v1 / v2)),
            (Value::Int(v1), Value::Real(v2)) => Ok(Value::Real(v1 as f32 / v2)),
            (Value::Int(v1), Value::Double(v2)) => Ok(Value::Double(v1 as f64 / v2)),
            (Value::Real(v1), Value::Bool(v2)) => Ok(Value::Real(v1 / (v2 as u8) as f32)),
            (Value::Real(v1), Value::Int(v2)) => Ok(Value::Real(v1 / v2 as f32)),
            (Value::Real(v1), Value::Real(v2)) => Ok(Value::Real(v1 / v2)),
            (Value::Real(v1), Value::Double(v2)) => Ok(Value::Double(v1 as f64 / v2)),
            (Value::Double(v1), Value::Bool(v2)) => Ok(Value::Double(v1 / (v2 as u8) as f64)),
            (Value::Double(v1), Value::Int(v2)) => Ok(Value::Double(v1 / v2 as f64)),
            (Value::Double(v1), Value::Real(v2)) => Ok(Value::Double(v1 / v2 as f64)),
            (Value::Double(v1), Value::Double(v2)) => Ok(Value::Double(v1 / v2)),
        }
    }
}
