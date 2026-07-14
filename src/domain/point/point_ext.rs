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
        match self {
            Point::Bool(p) => Ok(p.value.0),
            Point::Int(p) => Ok(p.value != 0),
            Point::Real(p) => Ok(p.value.is_finite() && p.value != 0.0),
            Point::Double(p) => Ok(p.value.is_finite() && p.value != 0.0),
            _ => return Err(sal_core::error::Error::new("Point", "try_to<bool>").err(concat_string::concat_string!("Invalid type '", self.typ().to_string(), "'"))),
        }
    }
}
impl TryTo<i64> for &sal_sync::services::entity::Point {
    type Error = sal_core::error::Error;
    fn try_to(self) -> Result<i64, Self::Error> {
        match self {
            Point::Bool(p) => Ok(if p.value.0 {1} else {0}),
            Point::Int(p) => Ok(p.value),
            Point::Real(p) => Ok(p.value.round() as i64),
            Point::Double(p) => Ok(p.value.round() as i64),
            _ => return Err(sal_core::error::Error::new("Point", "try_to<i64>").err(concat_string::concat_string!("Invalid type '", self.typ().to_string(), "'"))),
        }
    }
}
impl TryTo<f32> for &sal_sync::services::entity::Point {
    type Error = sal_core::error::Error;
    fn try_to(self) -> Result<f32, Self::Error> {
        match self {
            Point::Bool(p) => Ok(if p.value.0 {1.0} else {0.0}),
            Point::Int(p) => Ok(p.value as f32),
            Point::Real(p) => Ok(p.value),
            Point::Double(p) => Ok(p.value as f32),
            _ => return Err(sal_core::error::Error::new("Point", "try_to<f32>").err(concat_string::concat_string!("Invalid type '", self.typ().to_string(), "'"))),
        }
    }
}
impl TryTo<f64> for &sal_sync::services::entity::Point {
    type Error = sal_core::error::Error;
    fn try_to(self) -> Result<f64, Self::Error> {
        match self {
            Point::Bool(p) => Ok(if p.value.0 {1.0} else {0.0}),
            Point::Int(p) => Ok(p.value as f64),
            Point::Real(p) => Ok(p.value as f64),
            Point::Double(p) => Ok(p.value),
            _ => return Err(sal_core::error::Error::new("Point", "try_to<f64>").err(concat_string::concat_string!("Invalid type '", self.typ().to_string(), "'"))),
        }
    }
}
impl TryTo<String> for &sal_sync::services::entity::Point {
    type Error = sal_core::error::Error;
    fn try_to(self) -> Result<String, Self::Error> {
        let value = match self {
            Point::Bool(p) => p.value.0.to_string(),
            Point::Int(p) => p.value.to_string(),
            Point::Real(p) => p.value.to_string(),
            Point::Double(p) => p.value.to_string(),
            Point::String(p) => p.value.clone(),
            _ => return Err(sal_core::error::Error::new("Point", "try_to<f64>").err(concat_string::concat_string!("Invalid type '", self.typ().to_string(), "'"))),
        };
        Ok(value)
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
            ts: p.ts(),
        }
    }
    ///
    /// Returns `PointMeta` updated with passed `Point`: `status`, `cot`, `timestamp` only if it has latest `timestamp`
    pub fn update_latest(self, p: &Point) -> Self {
        if p.ts() > self.ts {
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
