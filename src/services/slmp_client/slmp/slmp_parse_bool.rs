use log::{debug, warn};
use chrono::{DateTime, Utc};
use sal_sync::services::{
    entity::{
        cot::Cot,
        point::{
            point::Point, point_config::PointConfig, point_config_address::PointConfigAddress, 
            point_config_type::PointConfigType, point_hlr::PointHlr,
        },
        status::status::Status,
    },
    types::bool::Bool,
};
use crate::services::slmp_client::parse_point::ParsePoint;
///
/// Used for parsing configured point from slice of bytes read from device
#[derive(Debug, Clone)]
pub struct SlmpParseBool {
    id: String,
    pub type_: PointConfigType,
    pub tx_id: usize,
    pub name: String,
    pub value: i64,
    pub status: Status,
    pub offset: Option<u32>,
    pub bit: Option<u8>,
    // pub history: PointConfigHistory,
    // pub alarm: Option<u8>,
    // pub comment: Option<String>,
    pub timestamp: DateTime<Utc>,
    is_changed: bool,
}
impl SlmpParseBool {
    ///
    /// Size in the bytes in the Device address area
    const SIZE: usize = 2;
    ///
    /// Creates new instance of the SlmpPArseBool
    pub fn new(
        tx_id: usize,
        name: String,
        config: &PointConfig,
        // filter: Filter<T>,
    ) -> SlmpParseBool {
        SlmpParseBool {
            id: format!("SlmpParseBool"),
            type_: config.type_.clone(),
            tx_id,
            name,
            value: 0i64,
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
    ) -> Result<bool, String> {
        if bytes.len() >= start + Self::SIZE {
            match bytes[start..(start + Self::SIZE)].try_into() {
                Ok(v) => {
                    let value = i16::from_le_bytes(v);
                    Ok(self.get_bit(value as i64, bit))
                }
                Err(e) => {
                    // warn!("{}.convert | error: {}", self.id, e);
                    Err(format!("{}.convert | Error: {}", self.id, e))
                }
            }
        } else {
            Err(format!("{}.convert | Index {} + size {} out of range for slice of length {}", self.id, start, Self::SIZE, bytes.len()))
        }
    }
    ///
    ///
    fn to_point(&self) -> Option<Point> {
        if self.is_changed {
            Some(Point::Bool(PointHlr::new(
                self.tx_id,
                &self.name,
                Bool(self.get_bit(self.value, self.bit.unwrap() as usize)),
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
                let self_value = self.get_bit(self.value, self.bit.unwrap() as usize);
                if new_val != self_value || self.status != status {
                    self.value = self.change_bit(self.value, new_val, self.bit.unwrap() as usize);
                    self.status = status;
                    self.timestamp = timestamp;
                    self.is_changed = true;
                }
            }
            Err(e) => {
                self.status = Status::Invalid;
                warn!("{}.add_raw | convertion error: {:?}", self.id, e);
            }
        }
    }
    ///
    /// 
    fn get_bit(&self, value: i64, bit: usize) -> bool {
        let b = (value >> bit) & 1;
        b > 0
    }
    ///
    /// 
    fn change_bit(&self, value: i64, bit_value: bool, bit: usize) -> i64 {
        match bit_value {
            true  => self.set_bit(value, bit),
            false => self.reset_bit(value, bit),
        }
    }
    ///
    /// Sets single bit to '1' in the integer  [value]
    fn set_bit(&self, value: i64, bit: usize) -> i64 {
        let result = value | (1 << bit);
        debug!("{}.set_bit | Set bit operation: \n\t{} => \n\t{}", self.id, value, result);
        result
    }
    ///
    /// Resets single bit to '0' in the integer [value]
    fn reset_bit(&self, value: i64, bit: usize) -> i64 {
        let result = value & !(1 << bit);
        debug!("{}.set_bit | Reset bit operation: \n\t{} => \n\t{}", self.id, value, result);
        result
    }
}
///
impl ParsePoint for SlmpParseBool {
    //
    //
    fn type_(&self) -> PointConfigType {
        self.type_.clone()
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
        PointConfigAddress { offset: self.offset, bit: self.bit }
    }
    //
    //
    fn size(&self) -> usize {
        Self::SIZE
    }
    //
    //
    fn to_bytes(&self, point: &Point) -> Result<Vec<u8>, String> {
        match point.try_as_bool() {
            Ok(point) => {
                let value = self.change_bit(self.value, point.value.0, self.bit.unwrap() as usize);
                debug!("{}.write | converting '{}' into i16...", self.id, point.value);
                match i16::try_from(value) {
                    Ok(value) => {
                        Ok(value.to_le_bytes().to_vec())
                    }
                    Err(err) => {
                        let message = format!("{}.write | '{}' to i16 conversion error: {:#?} in the parse point: {:#?}", self.id, point.value, err, self.name);
                        warn!("{}", message);
                        Err(message)
                    }
                }
            }
            Err(_) => {
                let message = format!("{}.write | Point of type 'Bool' expected, but found '{:?}' in the parse point: {:#?}", self.id, point.type_(), self.name);
                warn!("{}", message);
                Err(message)
            }
        }
    }
}
