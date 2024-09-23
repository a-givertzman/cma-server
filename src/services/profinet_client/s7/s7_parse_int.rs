use sal_sync::services::entity::{
    cot::Cot, point::{point::Point, point_config::PointConfig, point_config_address::PointConfigAddress, point_config_history::PointConfigHistory, point_hlr::PointHlr},
    status::status::Status
};
use std::array::TryFromSliceError;
use chrono::{DateTime, Utc};
use crate::{core_::filter::filter::{Filter, FilterEmpty}, services::profinet_client::parse_point::ParsePoint};
///
///
#[derive(Debug)]
pub struct S7ParseInt {
    pub tx_id: usize,
    pub name: String,
    pub value: Box<dyn Filter<Item = i64>>,
    pub status: Box<dyn Filter<Item = Status>>,
    pub offset: Option<u32>,
    pub history: PointConfigHistory,
    pub alarm: Option<u8>,
    pub comment: Option<String>,
    pub timestamp: DateTime<Utc>,
}
//
//
impl S7ParseInt {
    ///
    ///
    pub fn new(
        tx_id: usize,
        name: String,
        config: &PointConfig,
        filter: Box<dyn Filter<Item = i64>>,
    ) -> S7ParseInt {
        S7ParseInt {
            tx_id,
            name,
            value: filter,
            status: Box::new(FilterEmpty::new(Some(Status::Invalid))),
            offset: config.clone().address.unwrap_or(PointConfigAddress::empty()).offset,
            history: config.history.clone(),
            alarm: config.alarm,
            comment: config.comment.clone(),
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
    ) -> Result<i16, TryFromSliceError> {
        // debug!("S7ParseInt.convert | start: {},  end: {:?}", start, start + 2);
        // let raw: [u8; 2] = (bytes[start..(start + 2)]).try_into().unwrap();
        // debug!("S7ParseInt.convert | raw: {:?}", raw);
        match bytes[start..(start + 2)].try_into() {
            Ok(v) => Ok(i16::from_be_bytes(v)),
            Err(e) => {
                log::warn!("S7ParseInt.convert | error: {}", e);
                Err(e)
            }
        }
    }
    ///
    ///
    fn to_point(&mut self) -> Option<Point> {
        if let Some(value) = self.value.value() {
            Some(Point::Int(PointHlr::new(
                self.tx_id,
                &self.name,
                value,
                self.status.value().unwrap_or(Status::Invalid),
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
    fn add_raw_simple(&mut self, bytes: &[u8]) {
        self.add_raw(bytes, Utc::now())
    }
    //
    //
    fn add_raw(&mut self, bytes: &[u8], timestamp: DateTime<Utc>) {
        let result = self.convert(bytes, self.offset.unwrap() as usize, 0);
        match result {
            Ok(new_val) => {
                self.value.add(new_val as i64);
                self.status.add(Status::Ok);
            }
            Err(e) => {
                self.status.add(Status::Invalid);
                log::warn!("S7ParseInt.addRaw | convertion error: {:?}", e);
            }
        }
        if self.is_changed() {
            self.timestamp = timestamp;
        }
    }
}
//
//
impl ParsePoint for S7ParseInt {
    //
    //
    fn next_simple(&mut self, bytes: &[u8]) -> Option<Point> {
        self.add_raw_simple(bytes);
        self.to_point()
    }
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
