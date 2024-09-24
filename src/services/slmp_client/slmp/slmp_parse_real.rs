use log::{trace, warn};
use chrono::{DateTime, Utc};
use sal_sync::services::entity::{
    cot::Cot,
    point::{
        point::Point, point_config::PointConfig, point_config_address::PointConfigAddress, 
        point_config_history::PointConfigHistory, point_config_type::PointConfigType, point_hlr::PointHlr,
    },
    status::status::Status,
};
use crate::{core_::filter::filter::{Filter, FilterEmpty}, services::slmp_client::parse_point::ParsePoint};
///
/// Used for parsing configured point from slice of bytes read from device
#[derive(Debug)]
pub struct SlmpParseReal {
    id: String,
    pub type_: PointConfigType,
    pub tx_id: usize,
    pub name: String,
    pub value: Box<dyn Filter<Item = f32> + Send>,
    pub status: Box<dyn Filter<Item = Status> + Send>,
    pub offset: Option<u32>,
    pub history: PointConfigHistory,
    pub alarm: Option<u8>,
    pub comment: Option<String>,
    pub timestamp: DateTime<Utc>,
}
//
//
impl SlmpParseReal {
    ///
    /// Size in the bytes in the Device address area
    const SIZE: usize = 4;
    ///
    ///
    pub fn new(
        tx_id: usize,
        name: String,
        config: &PointConfig,
        filter: Box<dyn Filter<Item = f32> + Send>,
    ) -> SlmpParseReal {
        SlmpParseReal {
            id: format!("SlmpParseReal"),
            type_: config.type_.clone(),
            tx_id,
            value: filter,
            status: Box::new(FilterEmpty::<2, Status>::new(Some(Status::Invalid))),
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
    ) -> Result<f32, String> {
        if bytes.len() > start + Self::SIZE {
            trace!("{}.convert | start: {},  end: {:?}", self.id, start, start + Self::SIZE);
            trace!("{}.convert | raw: {:02X?}", self.id, &bytes[start..(start + Self::SIZE)]);
            trace!("{}.convert | converted f32: {:?}", self.id, f32::from_le_bytes(bytes[start..(start + Self::SIZE)].try_into().unwrap()));
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
        if let Some(value) = self.value.pop() {
            Some(Point::Real(PointHlr::new(
                self.tx_id,
                &self.name,
                value,
                self.status.pop().unwrap_or(Status::Invalid),
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
                self.value.add(new_val);
                self.status.add(Status::Ok);
                if self.is_changed() {
                    self.timestamp = timestamp;
                }
            }
            Err(e) => {
                self.status.add(Status::Invalid);
                warn!("{}.add_raw | convertion error: {:?}", self.id, e);
            }
        }
    }
}
//
//
impl ParsePoint for SlmpParseReal {
    //
    //
    fn type_(&self) -> PointConfigType {
        self.type_.clone()
    }
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
                warn!("{}", message);
                Err(message)
            }
        }
    }    
}
