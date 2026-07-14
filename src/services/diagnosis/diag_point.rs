use std::sync::{Arc, atomic::{AtomicU32, Ordering}};

use chrono::Utc;
use sal_sync::services::entity::{Cot, PointHlr, Point, PointConf, Status};
///
/// Provides the state for diagnosis Point's
pub struct DiagPoint {
    txid: usize,
    conf: PointConf,
    value: AtomicU32,
}
//
//
impl DiagPoint {
    ///
    /// Creates new instance of the DiagPoint
    pub fn new(txid: usize, conf: PointConf) -> Self {
        Self {
            txid,
            conf,
            value: AtomicU32::new(u32::from(Status::Unknown(-1))),
        }
    }
    ///
    /// Returns diagnostic Point from value
    ///  - the value is represents the [Status]
    fn point(&self, value: Status) -> Point {
        Point::Int(PointHlr::new(
            self.txid,
            &self.conf.name,
            i64::from(value),
            Status::Ok,
            Cot::Inf,
            Utc::now(),
        ))
    }
    ///
    /// Returns updated point with
    pub fn next(&self, value: Status) -> Option<Point> {
        let val = u32::from(value);
        if val != self.value.swap(val, Ordering::AcqRel) {
            Some(self.point(value))
        } else {
            None
        }
    }
}