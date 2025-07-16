use std::io::{BufReader, Read};
use chrono::{DateTime, Utc};
use concat_string::concat_string;
use log::{warn, trace, LevelFilter};
use sal_sync::services::{
    entity::{
        cot::Cot, name::Name, object::Object, 
        point::{point::Point, point_hlr::PointHlr, point_tx_id::PointTxId},
        status::status::Status,
    }, 
    types::bool::Bool
};
use crate::{
    core_::net::connection_status::ConnectionStatus, tcp::{steam_read::TcpStreamRead, tcp_stream_write::OpResult}
};
use super::jds_decode_message::JdsDecodeMessage;
///
/// Converts squence of bytes into the PointType
/// useng bytes -> JSON -> Point<type> PointType conversion
#[derive(Debug)]
pub struct JdsDeserialize {
    id: String,
    name: Name,
    tx_id: usize,
    stream: JdsDecodeMessage,
}
//
// 
impl JdsDeserialize {
    ///
    /// Creates new instance of the JdsDeserialize
    pub fn new(parent: impl Into<String>, stream: JdsDecodeMessage) -> Self {
        let me = Name::new(parent, "JdsDeserialize");
        Self {
            tx_id: PointTxId::from_str(&me.join()),
            id: me.join(),
            name: me,
            stream,
        }
    }
    ///
    /// Reads single point from TcpStream
    pub fn read(&mut self, tcp_stream: impl Read) -> ConnectionStatus<OpResult<Point, String>, String> {
        match self.stream.read(tcp_stream) {
            ConnectionStatus::Active(result) => {
                match result {
                    OpResult::Ok(bytes) => {
                        match Self::deserialize(&self.id, self.tx_id, bytes) {
                            Ok(point) => {
                                ConnectionStatus::Active(OpResult::Ok(point))
                            }
                            Err(err) => {
                                if log::max_level() == LevelFilter::Debug {
                                    warn!("{}", err);
                                }
                                ConnectionStatus::Active(OpResult::Err(err))
                            }
                        }
                    }
                    OpResult::Err(err) => ConnectionStatus::Active(OpResult::Err(err)),
                    OpResult::Timeout() => ConnectionStatus::Active(OpResult::Timeout())
                }
            }
            ConnectionStatus::Closed(err) => {
                ConnectionStatus::Closed(err)
            }
        }
    }
    ///
    /// Returns Cot parsed from the json::Map by it's key "cot" 
    fn parse_cot(self_id: &str, name: &str, obj: &serde_json::Map<String, serde_json::Value>) -> Cot {
        trace!("{}.parse_cot | obj: {:#?}", self_id, obj);
        match obj.get("cot") {
            Some(value) => {
                match serde_json::from_value(value.clone()) {
                    Ok(direction) => direction,
                    Err(err) => {
                        let message = concat_string!(self_id, ".parse_cot | Deserialize Point.cot error: \n\t", err.to_string(), "\n\t in the: ", name, ": ", value.to_string());
                        warn!("{}", message);
                        Cot::default()
                    }
                }
            }
            None => Cot::default(),
        }
    }
    ///
    /// Deserialize point from JSON string
    pub fn deserialize(self_id: &str, tx_id: usize, bytes: Vec<u8>) -> Result<Point, String> {
        match serde_json::from_slice(&bytes) {
            Ok(value) => {
                let value: serde_json::Value = value;
                match value.as_object() {
                    Some(obj) => {
                        match obj.get("type") {
                            Some(type_) => {
                                match type_.as_str() {
                                    Some("bool") | Some("Bool") => {
                                        let name = obj.get("name").unwrap().as_str().unwrap();
                                        let value = obj.get("value").unwrap().as_u64().unwrap();
                                        let status = obj.get("status").unwrap().as_i64().unwrap();
                                        let direction = Self::parse_cot(self_id, name, obj);
                                        let timestamp = obj.get("timestamp").unwrap().as_str().unwrap();
                                        let timestamp: DateTime<Utc> = chrono::DateTime::parse_from_rfc3339(timestamp).unwrap().with_timezone(&Utc);
                                        Ok(Point::Bool(PointHlr::new(
                                            tx_id,
                                            name,
                                            Bool(value > 0),
                                            Status::from(status),
                                            direction,
                                            timestamp,
                                        )))
                                    }
                                    Some("int") | Some("Int") => {
                                        let name = obj.get("name").unwrap().as_str().unwrap();
                                        let value = obj.get("value").unwrap().as_i64().unwrap();
                                        let status = obj.get("status").unwrap().as_i64().unwrap();
                                        let direction = Self::parse_cot(self_id, name, obj);
                                        let timestamp = obj.get("timestamp").unwrap().as_str().unwrap();
                                        let timestamp: DateTime<Utc> = chrono::DateTime::parse_from_rfc3339(timestamp).unwrap().with_timezone(&Utc);
                                        Ok(Point::Int(PointHlr::new(
                                            tx_id,
                                            name,
                                            value,
                                            Status::from(status),
                                            direction,
                                            timestamp,
                                        )))
                                    }
                                    Some("real") | Some("Real") => {
                                        let name = obj.get("name").unwrap().as_str().unwrap();
                                        let value = obj.get("value").unwrap().as_f64().unwrap();
                                        let status = obj.get("status").unwrap().as_i64().unwrap();
                                        let direction = Self::parse_cot(self_id, name, obj);
                                        let timestamp = obj.get("timestamp").unwrap().as_str().unwrap();
                                        let timestamp: DateTime<Utc> = chrono::DateTime::parse_from_rfc3339(timestamp).unwrap().with_timezone(&Utc);
                                        Ok(Point::Real(PointHlr::new(
                                            tx_id,
                                            name,
                                            value as f32,
                                            Status::from(status),
                                            direction,
                                            timestamp,
                                        )))
                                    }
                                    Some("double") | Some("Double") => {
                                        let name = obj.get("name").unwrap().as_str().unwrap();
                                        let value = obj.get("value").unwrap().as_f64().unwrap();
                                        let status = obj.get("status").unwrap().as_i64().unwrap();
                                        let direction = Self::parse_cot(self_id, name, obj);
                                        let timestamp = obj.get("timestamp").unwrap().as_str().unwrap();
                                        let timestamp: DateTime<Utc> = chrono::DateTime::parse_from_rfc3339(timestamp).unwrap().with_timezone(&Utc);
                                        Ok(Point::Double(PointHlr::new(
                                            tx_id,
                                            name,
                                            value,
                                            Status::from(status),
                                            direction,
                                            timestamp,
                                        )))
                                    }
                                    Some("string") | Some("String") => {
                                        let name = obj.get("name").unwrap().as_str().unwrap();
                                        let value = obj.get("value").unwrap().as_str().unwrap();
                                        let status = obj.get("status").unwrap().as_i64().unwrap();
                                        let direction = Self::parse_cot(self_id, name, obj);
                                        let timestamp = obj.get("timestamp").unwrap().as_str().unwrap();
                                        let timestamp: DateTime<Utc> = chrono::DateTime::parse_from_rfc3339(timestamp).unwrap().with_timezone(&Utc);
                                        Ok(Point::String(PointHlr::new(
                                            tx_id,
                                            name,
                                            value.to_owned(),
                                            Status::from(status),
                                            direction,
                                            timestamp,
                                        )))
                                    }
                                    _ => {
                                        let message = format!("{}.parse | Unknown point type: {}", self_id, type_);
                                        trace!("{}", message);
                                        Err(message)
                                    }
                                }
                            }
                            None => {
                                let message = format!("{}.parse | JSON convertion error: mapping not found in the JSON: {}", self_id, value);
                                trace!("{}", message);
                                Err(message)        
                            }
                        }
                    }
                    None => {
                        let message = format!("{}.parse | JSON convertion error: mapping not found in the JSON: {}", self_id, value);
                        trace!("{}", message);
                        Err(message)
                    }
                }
            }
            Err(err) => {
                let message = format!("JdsDeserialize.parse | JSON convertion error: {:?}", err);
                trace!("{}", message);
                Err(message)        
            }
        }
    }    
}
//
// 
impl Object for JdsDeserialize {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl TcpStreamRead for JdsDeserialize {
    fn read(&mut self, tcp_stream: &mut BufReader<std::net::TcpStream>) -> ConnectionStatus<OpResult<Point, String>, String> {
        self.read(tcp_stream)
    }
}