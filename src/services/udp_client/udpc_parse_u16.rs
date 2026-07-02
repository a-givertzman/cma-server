use chrono::{DateTime, Utc};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::services::entity::{
    Cot, Point, PointConf, PointHlr, PointType, Status,
};
use crate::services::udp_client::parse_point::ParsePoint;
///
/// Used for parsing configured point from slice of bytes read from device
#[derive(Debug)]
pub struct UdpcParseU16 {
    pub txid: usize,
    pub typ: PointType,
    pub name: String,
    pub status: Status,
    dbg: Dbg,
}
//
//
impl UdpcParseU16 {
    ///
    /// Size in the bytes in the single value of the Device address area
    const SIZE: usize = 2;
    ///
    /// - `size` - `Values<i16>` in the array coming from the associated channel
    pub fn new(
        txid: usize,
        parent: impl Into<String>,
        conf: &PointConf,
    ) -> UdpcParseU16 {
        let dbg =  Dbg::new(parent, format!("UdpcParseU16({})", conf.name));
        UdpcParseU16 {
            txid,
            typ: conf.type_.clone(),
            name: conf.name.clone(),
            status: Status::Invalid,
            dbg,
        }
    }
    ///
    /// Returns u16 values converted from butes or `Err`
    fn convert(&mut self, bytes: &[u8]) -> Result<impl Iterator<Item = u16>, Error> {
        log::trace!("{}.convert | bytes: {:?}", self.dbg, bytes);
        if !bytes.is_empty() {
            let (words, remainder) = bytes.as_chunks::<{ Self::SIZE }>();
            log::trace!("{}.convert | words: {:?}", self.dbg, words.len());
            if remainder.len() > 0 {
                Err(Error::new(&self.name, "convert").err(format!("Wrong input len {}, must be divisible by 2", remainder.len())))
            } else {
                let values = words.iter().enumerate().map(|(index, word)| {
                    // log::debug!("{}.convert | index: {}  |  word: {:?}", self.id, index, word);
                    log::trace!("{}.convert | index: {}  |  word: {:?}", self.dbg, index, word);
                    u16::from_be_bytes(*word)
                });
                log::trace!("{}.convert | values: {:?}", self.dbg, values);
                Ok(values)
            }
        } else {
            Err(Error::new(&self.name, "convert").err("Input is empty"))
        }
    }
    ///
    /// Returns [Point]'s of type `Int` parsed from specified bytes
    fn add(&mut self, bytes: &[u8], status: Status, timestamp: DateTime<Utc>) -> Result<Vec<Point>, Error> {
        let dbg = self.dbg.clone();
        self.status = status;
        let name = self.name.clone();
        let txid = self.txid;
        match self.convert(bytes) {
            Ok(values) => {
                Ok(values.map(move |value| Point::Int(PointHlr::new(
                    txid,
                    &name,
                    value as i64,
                    status,
                    Cot::Inf,
                    timestamp,
                ))).collect())
            }
            Err(err) => Err(Error::new(dbg, "add").pass(err))
        }
    }
}
//
//
impl ParsePoint for UdpcParseU16 {
    //
    //
    fn typ(&self) -> PointType {
        self.typ.clone()
    }
    //
    //
    fn add(&mut self, bytes: &[u8], status: Status, timestamp: DateTime<Utc>) -> Result<Vec<Point>, Error> {
        self.add(bytes, status, timestamp)
    }
    //
    //
    fn name(&self) -> String {
        self.name.clone()
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
                log::trace!("{}.write | converting '{}' into i16...", self.dbg, point.value);
                match i16::try_from(point.value) {
                    Ok(value) => {
                        Ok(value.to_le_bytes().to_vec())
                    }
                    Err(err) => {
                        let message = format!("{}.write | '{}' to i16 conversion error: {:#?} in the parse point: {:#?}", self.dbg, point.value, err, self.name);
                        log::warn!("{}", message);
                        Err(message)
                    }
                }
            }
            Err(_) => {
                let message = format!("{}.write | Point of type 'Int' expected, but found '{:?}' in the parse point: {:#?}", self.dbg, point.typ(), self.name);
                log::warn!("{}", message);
                Err(message)
            }
        }
    }
}
