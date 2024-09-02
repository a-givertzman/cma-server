use chrono::{DateTime, Utc};
use sal_sync::services::entity::{
    cot::Cot,
    point::{
        point::Point, point_config::PointConfig, point_config_address::PointConfigAddress, 
        point_config_history::PointConfigHistory, point_config_type::PointConfigType, point_hlr::PointHlr,
    },
    status::status::Status,
};
use crate::services::udp_client::parse_point::ParsePoint;
///
/// Used for parsing configured point from slice of bytes read from device
#[derive(Debug)]
pub struct UdpcParseI16 {
    id: String,
    pub type_: PointConfigType,
    pub tx_id: usize,
    pub name: String,
    pub values: Vec<Option<i16>>,
    pub status: Status,
    pub size: usize,
    pub history: PointConfigHistory,
    pub alarm: Option<u8>,
    pub comment: Option<String>,
    pub timestamp: DateTime<Utc>,
    is_changed: bool,
    prev: i16,
}
//
//
impl UdpcParseI16 {
    ///
    /// Size in the bytes in the Device address area
    const SIZE: usize = 2;
    ///
    /// - `size` - length of the array caming from the assoscieated channel
    pub fn new(
        tx_id: usize,
        name: String,
        size: usize,
        config: &PointConfig,
    ) -> UdpcParseI16 {
        UdpcParseI16 {
            id: format!("UdpcParseI16({})", name),
            type_: config.type_.clone(),
            tx_id,
            name,
            values: vec![None; size],
            status: Status::Invalid,
            is_changed: false,
            size,
            history: config.history.clone(),
            alarm: config.alarm,
            comment: config.comment.clone(),
            timestamp: Utc::now(),
            prev: 0,
        }
    }
    ///
    /// Returns Ok(is_changed) or Err(String)
    fn convert(
        &mut self,
        bytes: &[u8],
    ) -> Result<bool, String> {
        log::trace!("{}.run | bytes: {:?}", self.id, bytes);
        let mut is_changed = false;
        if ! bytes.is_empty() {
            let words = bytes.chunks(2);
            self.values = vec![];
            log::debug!("{}.run | words: {:?}", self.id, words.len());
            for (index, word) in words.enumerate() {
                // log::debug!("{}.run | index: {}  |  word: {:?}", self.id, index, word);
                match word.try_into() {
                    Ok(v) => {
                        log::debug!("{}.run | index: {}  |  word: {:?}", self.id, index, word);
                        is_changed = true;
                        self.values.push(Some(i16::from_be_bytes(v)));
                    }
                    Err(err) => {
                        log::warn!("{}.convert | Error: {} \n\t on index {}, bytes: {:?}", self.id, err, index, word);
                    }
                }
            }
            log::debug!("{}.convert | is_changed: {}  |  values: {:?}", self.id, is_changed, self.values);
            Ok(is_changed)
        } else {
            Err(format!("{}.convert | Size {} out of range for slice of length {}", self.id, self.size, bytes.len()))
        }
    }
    ///
    ///
    fn to_point(&mut self) -> Option<Point> {
        match self.values.pop() {
            Some(value) => {
                let (status, value) = match value {
                    Some(value) => {
                        self.prev = value;
                        (Status::Ok, value)
                    }
                    None => (Status::Invalid, self.prev),
                };
                Some(Point::Int(PointHlr::new(
                    self.tx_id,
                    &self.name,
                    value as i64,
                    if status > self.status {status} else {self.status},
                    Cot::Inf,
                    self.timestamp,
                )))
            }
            None => None,
        }
        // debug!("{} point Bool: {:?}", self.id, dsPoint.value);
    }
    //
    //
    fn add(&mut self, bytes: &[u8], status: Status, timestamp: DateTime<Utc>) {
        match self.convert(bytes) {
            Ok(is_changed) => {
                if is_changed || status != self.status {
                    self.is_changed = true;
                    self.status = status;
                    self.timestamp = timestamp;
                }
            }
            Err(err) => {
                log::warn!("{}.add_raw | convertion error: {:?}", self.id, err);
            }
        }
    }
}
//
//
impl ParsePoint for UdpcParseI16 {
    //
    //
    fn type_(&self) -> PointConfigType {
        self.type_.clone()
    }
    //
    //
    fn add(&mut self, bytes: &[u8], status: Status, timestamp: DateTime<Utc>) {
        self.add(bytes, status, timestamp)
    }
    //
    //
    fn next(&mut self) -> Option<Point> {
        self.to_point()
    }
    //
    //
    fn is_changed(&self) -> bool {
        self.is_changed
    }
    //
    //
    fn address(&self) -> PointConfigAddress {
        PointConfigAddress { offset: None, bit: None }
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
