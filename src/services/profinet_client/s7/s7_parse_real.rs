use log::warn;
use sal_sync::services::entity::{
    cot::Cot, point::{point::Point, point_config::PointConfig, point_config_address::PointConfigAddress, point_hlr::PointHlr},
    status::status::Status
};
use std::array::TryFromSliceError;
use chrono::{DateTime, Utc};
use crate::{core_::filter::filter::{Filter, FilterEmpty}, services::profinet_client::parse_point::ParsePoint};
///
///
#[derive(Debug)]
pub struct S7ParseReal {
    pub tx_id: usize,
    pub name: String,
    pub value: Box<dyn Filter<Item = f32>>,
    pub status: Box<dyn Filter<Item = Status>>,
    pub offset: Option<u32>,
    // pub history: PointConfigHistory,
    // pub alarm: Option<u8>,
    // pub comment: Option<String>,
    pub timestamp: DateTime<Utc>,

}
//
//
impl S7ParseReal {
    ///
    ///
    pub fn new(
        tx_id: usize,
        name: String,
        config: &PointConfig,
        filter: Box<dyn Filter<Item = f32>>,
    ) -> S7ParseReal {
        S7ParseReal {
            tx_id,
            value: filter,
            status: Box::new(FilterEmpty::<2, Status>::new(Some(Status::Invalid))),
            name,
            offset: config.clone().address.unwrap_or(PointConfigAddress::empty()).offset,
            // history: config.history.clone(),
            // alarm: config.alarm,
            // comment: config.comment.clone(),
            timestamp: Utc::now(),
        }
    }
    //
    //
    fn convert(
        &self,
        bytes: &[u8],
        start: usize,
        _bit: usize,
    ) -> Result<f32, TryFromSliceError> {
        match bytes[start..(start + 4)].try_into() {
            Ok(v) => Ok(f32::from_be_bytes(v)),
            Err(e) => {
                warn!("S7ParseReal.convert | error: {}", e);
                Err(e)
            }
        }
    }
    ///
    ///
    fn to_point(&mut self) -> Option<Point> {
        let value_status = match (self.value.pop(), self.status.pop()) {
            (None, None) => None,
            (None, Some(status)) => match self.value.last() {
                Some(value) => Some((value, Some(status))),
                None => None,
            }
            (Some(value), None) => Some((value, self.status.last())),
            (Some(value), Some(status)) => Some((value, Some(status))),
        };
        if let Some((value, status)) = value_status {
            Some(Point::Real(PointHlr::new(
                self.tx_id,
                &self.name,
                value,
                status.unwrap_or(Status::Invalid),
                Cot::Inf,
                self.timestamp,
            )))
            // debug!("{} point Bool: {:?}", self.id, dsPoint.value);
        } else {
            None
        }
    }
    //
    //
    fn add_raw(&mut self, bytes: &[u8], timestamp: DateTime<Utc>) {
        let result = self.convert(bytes, self.offset.unwrap() as usize, 0);
        match result {
            Ok(new_val) => {
                self.value.add(new_val);
                self.status.add(Status::Ok);
            }
            Err(e) => {
                self.status.add(Status::Invalid);
                warn!("S7ParseReal.addRaw | convertion error: {:?}", e);
            }
        }
        if self.is_changed() {
            self.timestamp = timestamp;
        }
    }
}
//
//
impl ParsePoint for S7ParseReal {
    //
    //
    fn next(&mut self, bytes: &[u8], timestamp: DateTime<Utc>) -> Option<Point> {
        self.add_raw(bytes, timestamp);
        self.to_point()
    }
    //
    //
    fn next_status(&mut self, status: Status) -> Option<Point> {
        self.status.add(status);
        if self.is_changed() {
            self.timestamp = Utc::now();
        }
        self.to_point()
    }
    //
    //
    fn is_changed(&self) -> bool {
        self.value.is_changed() || self.status.is_changed()
    }
    //
    //
    fn address(&self) -> PointConfigAddress {
        PointConfigAddress { offset: self.offset, bit: None }
    }
}
