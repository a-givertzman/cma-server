use log::warn;
use std::array::TryFromSliceError;
use chrono::{DateTime, Utc};
use crate::{
    conf::point_config::{point_config::PointConfig, point_config_address::PointConfigAddress, point_config_history::PointConfigHistory},
    core_::{cot::cot::Cot, point::{point::Point, point_type::PointType}, status::status::Status, types::bool::Bool},
    services::profinet_client::parse_point::ParsePoint,
};

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
    pub history: PointConfigHistory,
    pub alarm: Option<u8>,
    pub comment: Option<String>,
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
    fn to_point(&self) -> Option<PointType> {
        if self.is_changed {
            Some(PointType::Bool(Point::new(
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
    fn add_raw_simple(&mut self, bytes: &[u8]) {
        self.add_raw(bytes, Utc::now())
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
                if new_val != self.value {
                    self.value = new_val;
                    self.status = Status::Ok;
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
    fn next_simple(&mut self, bytes: &[u8]) -> Option<PointType> {
        self.add_raw_simple(bytes);
        self.to_point()
    }
    //
    //
    fn next(&mut self, bytes: &[u8], timestamp: DateTime<Utc>) -> Option<PointType> {
        self.add_raw(bytes, timestamp);
        self.to_point().map(|point| {
            self.is_changed = false;
            point
        })
    }
    //
    //
    fn next_status(&mut self, status: Status) -> Option<PointType> {
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
