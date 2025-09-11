use chrono::{DateTime, Utc};
use sal_sync::services::entity::{
    Cot, Point, PointConf, PointConfAddress, PointConfType, PointHlr, Status,
};
use crate::{domain::filter::filter::{Filter, FilterEmpty}, services::modbus_tcp::modbus::ParsePoint};
///
/// Used for parsing configured point from slice of bytes read from device
#[derive(Debug)]
pub struct ModbusParseReal {
    id: String,
    pub type_: PointConfType,
    pub txid: usize,
    pub name: String,
    pub value: Box<dyn Filter<Item = f32> + Send>,
    pub status: Box<dyn Filter<Item = Status> + Send>,
    pub offset: Option<u32>,
    // pub history: PointConfHistory,
    // pub alarm: Option<u8>,
    // pub comment: Option<String>,
    pub timestamp: DateTime<Utc>,
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
            status: Box::new(FilterEmpty::<2, Status>::new(Some(Status::Invalid))),
            name,
            offset: config.clone().address.unwrap_or(PointConfAddress::empty()).offset,
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
                self.txid,
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
                if self.is_changed() {
                    self.timestamp = timestamp;
                }
            }
            Err(e) => {
                self.status.add(Status::Invalid);
                log::warn!("{}.add_raw | convertion error: {:?}", self.id, e);
            }
        }
    }
}
//
//
impl ParsePoint for ModbusParseReal {
    //
    //
    fn type_(&self) -> PointConfType {
        self.type_.clone()
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
                let message = format!("{}.write | Point of type 'Real / Double' expected, but found '{:?}' in the parse point: {:#?}", self.id, point.type_(), self.name);
                log::warn!("{}", message);
                Err(message)
            }
        }
    }    
}
