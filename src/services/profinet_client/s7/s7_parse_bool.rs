use log::warn;
use sal_sync::services::{
    entity::{
        cot::Cot, point::{point::Point, point_config::PointConfig, point_config_address::PointConfigAddress, point_hlr::PointHlr},
        status::status::Status
    },
    types::bool::Bool,
};
use std::array::TryFromSliceError;
use chrono::{DateTime, Utc};
use crate::services::profinet_client::parse_point::ParsePoint;

///
///
#[derive(Debug, Clone)]
pub struct S7ParseBool {
    pub tx_id: usize,
    pub name: String,
    pub value: bool,
    pub status: Status,
    pub offset: Option<u32>,
    pub bit: Option<u8>,
    // pub history: PointConfigHistory,
    // pub alarm: Option<u8>,
    // pub comment: Option<String>,
    pub timestamp: DateTime<Utc>,
    is_changed: bool,
}
impl S7ParseBool {
    ///
    ///
    pub fn new(
        tx_id: usize,
        name: String,
        config: &PointConfig,
        // filter: Filter<T>,
    ) -> S7ParseBool {
        S7ParseBool {
            tx_id,
            name,
            value: false,
            status: Status::Invalid,
            is_changed: false,
            offset: config.clone().address.unwrap_or(PointConfigAddress::empty()).offset,
            bit: config.clone().address.unwrap_or(PointConfigAddress::empty()).bit,
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
        bit: usize,
    ) -> Result<bool, TryFromSliceError> {
        match bytes[start..(start + 2)].try_into() {
            Ok(v) => {
                let i = i16::from_le_bytes(v);
                let b: i16 = i >> bit & 1;
                Ok(b > 0)
            }
            Err(e) => {
                warn!("S7ParseBool.convert | error: {}", e);
                Err(e)
            }
        }
    }
    ///
    ///
    fn to_point(&self) -> Option<Point> {
        if self.is_changed {
            Some(Point::Bool(PointHlr::new(
                self.tx_id,
                &self.name,
                Bool(self.value),
                self.status,
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
        let result = self.convert(
            bytes,
            self.offset.unwrap() as usize,
            self.bit.unwrap() as usize,
        );
        match result {
            Ok(new_val) => {
                let status = Status::Ok;
                if new_val != self.value || self.status != status {
                    self.value = new_val;
                    self.status = status;
                    self.timestamp = timestamp;
                    self.is_changed = true;
                }
            }
            Err(e) => {
                self.status = Status::Invalid;
                warn!("S7ParseBool.addRaw | convertion error: {:?}", e);
            }
        }
    }
}
///
impl ParsePoint for S7ParseBool {
    //
    //
    fn next(&mut self, bytes: &[u8], timestamp: DateTime<Utc>) -> Option<Point> {
        self.add_raw(bytes, timestamp);
        self.to_point().map(|point| {
            self.is_changed = false;
            point
        })
    }
    //
    //
    fn next_status(&mut self, status: Status) -> Option<Point> {
        if self.status != status {
            self.status = status;
            self.timestamp = Utc::now();
            self.is_changed = true;
        }
        self.to_point().map(|point| {
            self.is_changed = false;
            point
        })
    }
    //
    //
    fn is_changed(&self) -> bool {
        self.is_changed
    }
    //
    //
    fn address(&self) -> PointConfigAddress {
        PointConfigAddress { offset: self.offset, bit: self.bit }
    }
}
