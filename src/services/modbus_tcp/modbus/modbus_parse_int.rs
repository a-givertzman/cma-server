use chrono::{DateTime, Utc};
use sal_sync::services::entity::{
    Cot, Point, PointConf, PointConfAddress, PointConfType, PointHlr, Status,
};
use crate::{domain::filter::filter::{Filter, FilterEmpty}, services::modbus_tcp::modbus::ParsePoint};
///
/// Used for parsing configured point from slice of bytes read from device
#[derive(Debug)]
pub struct ModbusParseInt {
    id: String,
    pub type_: PointConfType,
    pub txid: usize,
    pub name: String,
    pub value: Box<dyn Filter<Item = i64> + Send>,
    pub status: Box<dyn Filter<Item = Status> + Send>,
    pub offset: Option<u32>,
    // pub history: PointConfHistory,
    // pub alarm: Option<u8>,
    // pub comment: Option<String>,
}
//
//
impl ModbusParseInt {
    ///
    /// Size in the bytes in the Device address area
    const SIZE: usize = 2;
    ///
    ///
    pub fn new(
        txid: usize,
        name: String,
        config: &PointConf,
        filter: Box<dyn Filter<Item = i64> + Send>,
    ) -> ModbusParseInt {
        ModbusParseInt {
            id: format!("ModbusParseInt({})", name),
            type_: config.type_.clone(),
            txid,
            name,
            value: filter,
            status: Box::new(FilterEmpty::<Status>::new(Some(Status::Invalid))),
            offset: config.clone().address.unwrap_or(PointConfAddress::empty()).offset,
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
        _bit: usize,
    ) -> Result<i16, String> {
        if bytes.len() >= start + Self::SIZE {
            log::trace!("{}.convert | start: {},  end: {:?}", self.id, start, start + Self::SIZE);
            log::trace!("{}.convert | raw: {:02X?}", self.id, &bytes[start..(start + Self::SIZE)]);
            log::trace!("{}.convert | converted i16: {:?}", self.id, i16::from_le_bytes(bytes[start..(start + Self::SIZE)].try_into().unwrap()));
            match bytes[start..(start + Self::SIZE)].try_into() {
                Ok(v) => Ok(i16::from_le_bytes(v)),
                Err(e) => {
                    // log::warn!("{}.convert | error: {}", self.id, e);
                    Err(format!("{}.convert | Error: {}", self.id, e))
                }
            }
        } else {
            Err(format!("{}.convert | Index {} + size {} out of range for slice of length {}", self.id, start, Self::SIZE, bytes.len()))
        }
    }
    ///
    ///
    fn to_point(&mut self, value: Option<i64>, status: Status, timestamp: DateTime<Utc>) -> Option<Point> {
        let value = value.map_or(self.value.last(), |v| self.value.add(v)) ;
        let status = self.status.add(status);
        match (value, status) {
            (None, None) => None,
            (value, status) => {
                Some(Point::Int(PointHlr::new(
                    self.txid,
                    &self.name,
                    value.or_else(|| self.value.last())?,
                    status.or_else(|| self.status.last()).unwrap_or(Status::Ok),
                    Cot::Inf,
                    timestamp,
                )))
            }
        }
    }
    //
    //
    fn add_raw(&mut self, bytes: &[u8], timestamp: DateTime<Utc>) -> Option<Point> {
        let result = self.convert(bytes, self.offset.unwrap() as usize, 0);
        match result {
            Ok(value) => self.to_point(Some(value as i64), Status::Ok, timestamp),
            Err(e) => {
                log::warn!("{}.add_raw | convertion error: {:?}", self.id, e);
                self.to_point(None, Status::Invalid, timestamp)
            }
        }
    }
}
//
//
impl ParsePoint for ModbusParseInt {
    //
    //
    fn type_(&self) -> PointConfType {
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
        PointConfAddress { offset: self.offset, bit: None }
    }
    //
    //
    fn size(&self) -> usize {
        Self::SIZE
    }
    //
    //
    fn to_bytes(&self, point: &Point) -> Result<Vec<u8>, String> {
        match point.try_as_int() {
            Ok(point) => {
                log::debug!("{}.write | converting '{}' into i16...", self.id, point.value);
                match i16::try_from(point.value) {
                    Ok(value) => {
                        Ok(value.to_le_bytes().to_vec())
                    }
                    Err(err) => {
                        let message = format!("{}.write | '{}' to i16 conversion error: {:#?} in the parse point: {:#?}", self.id, point.value, err, self.name);
                        log::warn!("{}", message);
                        Err(message)
                    }
                }
            }
            Err(_) => {
                let message = format!("{}.write | Point of type 'Int' expected, but found '{:?}' in the parse point: {:#?}", self.id, point.type_(), self.name);
                log::warn!("{}", message);
                Err(message)
            }
        }
    }
}
