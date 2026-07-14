use chrono::{DateTime, Utc};
use sal_sync::services::
    entity::{Point, PointConf, PointConfAddress, PointType, Status}
;
use crate::{domain::filter::filter::{Filter, FilterEmpty}, services::modbus_tcp::modbus::ParsePoint};
///
/// Used for parsing configured point from slice of bytes read from device
#[derive(Debug)]
pub struct ModbusParseBool {
    id: String,
    pub type_: PointType,
    pub txid: usize,
    pub name: String,
    pub value: Box<dyn Filter<Item = bool> + Send>,
    pub status: Box<dyn Filter<Item = Status> + Send>,
    pub offset: Option<u32>,
    pub bit: Option<u8>,
    // pub history: PointConfHistory,
    // pub alarm: Option<u8>,
    // pub comment: Option<String>,
    pub timestamp: DateTime<Utc>,
    is_changed: bool,
}
impl ModbusParseBool {
    ///
    /// Size in the bytes in the Device address area
    const SIZE: usize = 2;
    ///
    /// Creates new instance of the SlmpPArseBool
    pub fn new(
        txid: usize,
        name: String,
        config: &PointConf,
        filter: Box<dyn Filter<Item = bool> + Send>,
    ) -> ModbusParseBool {
        ModbusParseBool {
            id: format!("ModbusParseBool"),
            type_: config.type_.clone(),
            txid,
            name,
            value: filter,
            status: Box::new(FilterEmpty::<Status>::new(Some(Status::Invalid))),
            is_changed: false,
            offset: config.clone().address.unwrap_or(PointConfAddress::empty()).offset,
            bit: config.clone().address.unwrap_or(PointConfAddress::empty()).bit,
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
    fn to_point(&self,value: Option<i64>, status: Status, timestamp: DateTime<Utc>) -> Option<Point> {
        todo!()
        // let value = value.map_or(self.value.last(), |v| self.value.add(v)) ;
        // let status = self.status.add(status);
        // match (value, status) {
        //     (None, None) => None,
        //     (value, status) => {
        //         let value = value.or_else(|| self.value.last())?;
        //         Some(Point::Bool(PointHlr::new(
        //             self.txid,
        //             &self.name,
        //             Bool(self.get_bit(value, self.bit.unwrap() as usize)),
        //             status.or_else(|| self.status.last()).unwrap_or(Status::Ok),
        //             Cot::Inf,
        //             timestamp,
        //         )))
        //     }
        // }

        // if self.is_changed {
        //     Some(Point::Bool(PointHlr::new(
        //         self.txid,
        //         &self.name,
        //         Bool(self.get_bit(self.value, self.bit.unwrap() as usize)),
        //         self.status,
        //         Cot::Inf,
        //         self.timestamp,
        //     )))
        //     // debug!("{} point Bool: {:?}", self.id, dsPoint.value);
        // } else {
        //     None
        // }
    }
    //
    //
    fn add_raw(&mut self, bytes: &[u8], timestamp: DateTime<Utc>) -> Option<Point> {
        todo!()
        // let result = self.convert(
        //     bytes,
        //     self.offset.unwrap() as usize,
        //     self.bit.unwrap() as usize,
        // );
        // match result {
        //     Ok(new_val) => {
        //         let status = Status::Ok;
        //         let self_value = self.get_bit(self.value, self.bit.unwrap() as usize);
        //         if new_val != self_value || self.status != status {
        //             self.value = self.change_bit(self.value, new_val, self.bit.unwrap() as usize);
        //             self.status = status;
        //             self.timestamp = timestamp;
        //             self.is_changed = true;
        //         }
        //     }
        //     Err(e) => {
        //         self.status = Status::Invalid;
        //         log::warn!("{}.add_raw | convertion error: {:?}", self.id, e);
        //     }
        // }
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
        log::debug!("{}.set_bit | Set bit operation: \n\t{} => \n\t{}", self.id, value, result);
        result
    }
    ///
    /// Resets single bit to '0' in the integer [value]
    fn reset_bit(&self, value: i64, bit: usize) -> i64 {
        let result = value & !(1 << bit);
        log::debug!("{}.set_bit | Reset bit operation: \n\t{} => \n\t{}", self.id, value, result);
        result
    }
}
///
impl ParsePoint for ModbusParseBool {
    //
    //
    fn type_(&self) -> PointType {
        self.type_.clone()
    }
    //
    //
    fn next(&mut self, bytes: &[u8], timestamp: DateTime<Utc>) -> Option<Point> {
        self.add_raw(bytes, timestamp)
    }
    //
    //
    fn next_status(&mut self, status: Status) -> Option<Point> {
        self.to_point(None, status, Utc::now())
    }
    //
    //
    fn address(&self) -> PointConfAddress {
        PointConfAddress { offset: self.offset, bit: self.bit }
    }
    //
    //
    fn size(&self) -> usize {
        Self::SIZE
    }
    //
    //
    fn to_bytes(&self, point: &Point) -> Result<Vec<u8>, String> {
        todo!()
        // match point.try_as_bool() {
        //     Ok(point) => {
        //         let value = self.change_bit(self.value, point.value.0, self.bit.unwrap() as usize);
        //         log::debug!("{}.write | converting '{}' into i16...", self.id, point.value);
        //         match i16::try_from(value) {
        //             Ok(value) => {
        //                 Ok(value.to_le_bytes().to_vec())
        //             }
        //             Err(err) => {
        //                 let message = format!("{}.write | '{}' to i16 conversion error: {:#?} in the parse point: {:#?}", self.id, point.value, err, self.name);
        //                 log::warn!("{}", message);
        //                 Err(message)
        //             }
        //         }
        //     }
        //     Err(_) => {
        //         let message = format!("{}.write | Point of type 'Bool' expected, but found '{:?}' in the parse point: {:#?}", self.id, point.typ(), self.name);
        //         log::warn!("{}", message);
        //         Err(message)
        //     }
        // }
    }
}
