use chrono::Utc;
use crate::{
    conf::point_config::point_config::PointConfig,
    core_::{
        cot::cot::Cot,
        point::{point_hlr::PointHlr, point_type::PointType},
        status::status::Status,
    }
};
///
/// Provides the state for diagnosis Point's
pub struct DiagPoint {
    tx_id: usize,
    conf: PointConfig,
    value: Status,
}
//
//
impl DiagPoint {
    ///
    /// Creates new instance of the DiagPoint
    pub fn new(tx_id: usize, conf: PointConfig) -> Self {
        Self {
            tx_id,
            conf,
            value: Status::Unknown(-1),
        }
    }
    ///
    /// Returns diagnostic Point from value
    ///  - the value is represents the [Status]
    fn point(&self, value: Status) -> PointType {
        PointType::Int(PointHlr::new(
            self.tx_id,
            &self.conf.name,
            i64::from(value),
            Status::Ok,
            Cot::Inf,
            Utc::now(),
        ))
    }
    ///
    /// Returns updated point with
    pub fn next(&mut self, value: Status) -> Option<PointType> {
        if value != self.value {
            self.value = value;
            Some(self.point(value))
        } else {
            None
        }
    }
}