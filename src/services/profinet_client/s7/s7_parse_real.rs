use log::warn;
use sal_sync::services::entity::{
    cot::Cot, point::{point::Point, point_config::PointConfig, point_config_address::PointConfigAddress, point_config_history::PointConfigHistory, point_hlr::PointHlr},
    status::status::Status
};
use std::array::TryFromSliceError;
use chrono::{DateTime, Utc};
use crate::{core_::filter::filter::Filter, services::profinet_client::parse_point::ParsePoint};

///
///
#[derive(Debug)]
pub struct S7ParseReal {
    pub tx_id: usize,
    pub name: String,
    pub value: Box<dyn Filter<Item = f32>>,
    pub status: Status,
    pub offset: Option<u32>,
    pub history: PointConfigHistory,
    pub alarm: Option<u8>,
    pub comment: Option<String>,
    pub timestamp: DateTime<Utc>,
    is_changed: bool,
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
            status: Status::Invalid,
            is_changed: false,
            name,
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
    fn to_point(&self) -> Option<Point> {
        if self.is_changed {
            Some(Point::Real(PointHlr::new(
                self.tx_id,
                &self.name,
                self.value.value(),
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
        let result = self.convert(bytes, self.offset.unwrap() as usize, 0);
        match result {
            Ok(new_val) => {
                let status = Status::Ok;
                if (new_val) != self.value.value() || self.status != status {
                    self.value.add(new_val);
                    self.status = status;
                    self.timestamp = timestamp;
                    self.is_changed = true;
                }
            }
            Err(e) => {
                self.status = Status::Invalid;
                warn!("S7ParseReal.addRaw | convertion error: {:?}", e);
            }
        }
    }
}
//
//
impl ParsePoint for S7ParseReal {
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
        PointConfigAddress { offset: self.offset, bit: None }
    }
}
