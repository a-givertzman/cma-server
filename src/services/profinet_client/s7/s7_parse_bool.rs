use sal_sync::services::{
    entity::{Cot, Point, PointConf, PointConfAddress, PointHlr, Status},
    types::Bool,
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
    // pub history: PointConfHistory,
    // pub alarm: Option<u8>,
    // pub comment: Option<String>,
    is_changed: bool,
}
impl S7ParseBool {
    ///
    ///
    pub fn new(
        tx_id: usize,
        name: String,
        config: &PointConf,
        // filter: Filter<T>,
    ) -> S7ParseBool {
        S7ParseBool {
            tx_id,
            name,
            value: false,
            status: Status::Invalid,
            is_changed: false,
            offset: config.clone().address.unwrap_or(PointConfAddress::empty()).offset,
            bit: config.clone().address.unwrap_or(PointConfAddress::empty()).bit,
            // history: config.history.clone(),
            // alarm: config.alarm,
            // comment: config.comment.clone(),
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
                log::warn!("S7ParseBool.convert | error: {}", e);
                Err(e)
            }
        }
    }
    ///
    ///
    fn to_point(&self, timestamp: DateTime<Utc>) -> Option<Point> {
        if self.is_changed {
            Some(Point::Bool(PointHlr::new(
                self.tx_id,
                &self.name,
                Bool(self.value),
                self.status,
                Cot::Inf,
                timestamp,
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
                    self.is_changed = true;
                }
            }
            Err(e) => {
                self.status = Status::Invalid;
                log::warn!("S7ParseBool.addRaw | convertion error: {:?}", e);
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
        self.to_point(timestamp).map(|point| {
            self.is_changed = false;
            point
        })
    }
    //
    //
    fn next_status(&mut self, status: Status) -> Option<Point> {
        if self.status != status {
            self.status = status;
            self.is_changed = true;
        }
        self.to_point(Utc::now()).map(|point| {
            self.is_changed = false;
            point
        })
    }
    //
    //
    fn address(&self) -> PointConfAddress {
        PointConfAddress { offset: self.offset, bit: self.bit }
    }
}
