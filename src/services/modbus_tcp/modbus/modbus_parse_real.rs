use chrono::{DateTime, Utc};
use sal_sync::services::entity::{
    Cot, Point, PointConf, PointConfAddress, PointType, PointHlr, Status,
};
use crate::{domain::filter::filter::{Filter, FilterEmpty}, services::modbus_tcp::modbus::ParsePoint};
///
/// Used for parsing configured point from slice of bytes read from device
#[derive(Debug)]
pub struct ModbusParseReal {
    id: String,
    pub type_: PointType,
    pub txid: usize,
    pub name: String,
    pub value: Box<dyn Filter<Item = f32> + Send>,
    pub status: Box<dyn Filter<Item = Status> + Send>,
    pub offset: Option<u32>,
    // pub history: PointConfHistory,
    // pub alarm: Option<u8>,
    // pub comment: Option<String>,
}
//
//
impl ModbusParseReal {
    ///
    /// Size in the bytes in the Device address area
    const SIZE: usize = 4;
    ///
    ///
    pub fn new(
        txid: usize,
        name: String,
        config: &PointConf,
        filter: Box<dyn Filter<Item = f32> + Send>,
    ) -> ModbusParseReal {
        ModbusParseReal {
            id: format!("ModbusParseReal"),
            type_: config.type_.clone(),
            txid,
            value: filter,
            status: Box::new(FilterEmpty::<Status>::new(Some(Status::Invalid))),
            name,
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
    ) -> Result<f32, String> {
        if bytes.len() > start + Self::SIZE {
            log::trace!("{}.convert | start: {},  end: {:?}", self.id, start, start + Self::SIZE);
            log::trace!("{}.convert | raw: {:02X?}", self.id, &bytes[start..(start + Self::SIZE)]);
            log::trace!("{}.convert | converted f32: {:?}", self.id, f32::from_le_bytes(bytes[start..(start + Self::SIZE)].try_into().unwrap()));
            match bytes[start..(start + Self::SIZE)].try_into() {
                Ok(v) => Ok(f32::from_le_bytes(v)),
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
    fn to_point(&mut self, value: Option<f32>, status: Status, timestamp: DateTime<Utc>) -> Option<Point> {
        let value = value.map_or(self.value.last(), |v| self.value.add(v)) ;
        let status = self.status.add(status);
        match (value, status) {
            (None, None) => None,
            (value, status) => {
                Some(Point::Real(PointHlr::new(
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
            Ok(value) => self.to_point(Some(value), Status::Ok, timestamp),
            Err(e) => {
                log::warn!("{}.add_raw | convertion error: {:?}", self.id, e);
                self.to_point(None, Status::Invalid, timestamp)
            }
        }
    }
}
//
//
impl ParsePoint for ModbusParseReal {
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
        match point {
            Point::Real(point) => Ok(point.value.to_le_bytes().to_vec()),
            Point::Double(_) => Ok(point.to_real().as_real().value.to_le_bytes().to_vec()),
            _ => {
                let message = format!("{}.write | Point of type 'Real / Double' expected, but found '{:?}' in the parse point: {:#?}", self.id, point.typ(), self.name);
                log::warn!("{}", message);
                Err(message)
            }
        }
    }    
}
