use log::{trace, warn};
use chrono::{DateTime, Utc};
use crate::{
    conf::point_config::{point_config::PointConfig, point_config_address::PointConfigAddress, point_config_history::PointConfigHistory, point_config_type::PointConfigType},
    core_::{cot::cot::Cot, filter::filter::Filter, point::{point_hlr::PointHlr, point::Point}, status::status::Status},
    services::slmp_client::parse_point::ParsePoint,
};
///
/// Used for parsing configured point from slice of bytes read from device
#[derive(Debug)]
pub struct SlmpParseReal {
    id: String,
    pub type_: PointConfigType,
    pub tx_id: usize,
    pub name: String,
    pub value: Box<dyn Filter<Item = f32> + Sync + Send>,
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
        filter: Box<dyn Filter<Item = f32> + Sync + Send>,
    ) -> SlmpParseReal {
        SlmpParseReal {
            id: format!("SlmpParseReal"),
            type_: config.type_.clone(),
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
