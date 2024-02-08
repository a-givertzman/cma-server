#![allow(non_snake_case)]

use std::io::Read;

use chrono::{DateTime, Utc};
use log::{warn, trace, LevelFilter};

use crate::core_::{net::connection_status::ConnectionStatus, point::{point::{Direction, Point}, point_tx_id::PointTxId, point_type::PointType}, status::status::Status, types::bool::Bool};

use super::jds_decode_message::JdsDecodeMessage;

///
/// Converts squence of bytes into the PointType
/// useng bytes -> JSON -> Point<type> PointType conversion
pub struct JdsDeserialize {
    id: String,
    txId: usize,
    stream: JdsDecodeMessage,
}
///
/// 
impl JdsDeserialize {
    ///
    /// Creates new instance of the JdsDeserialize
    pub fn new(parent: impl Into<String>, stream: JdsDecodeMessage) -> Self {
        let selfId = format!("{}/JdsDeserialize", parent.into());
        Self {
            txId: PointTxId::fromStr(&selfId),
            id: selfId,
            stream,
        }
    }
    ///
    /// Reads single point from TcpStream
    pub fn read(&mut self, tcpStream: impl Read) -> ConnectionStatus<Result<PointType, String>, String> {
        match self.stream.read(tcpStream) {
            ConnectionStatus::Active(result) => {
                match result {
                    Ok(bytes) => {
                        match Self::deserialize(&self.id, self.txId, bytes) {
                            Ok(point) => {
                                ConnectionStatus::Active(Ok(point))
                            },
                            Err(err) => {
                                if log::max_level() == LevelFilter::Debug {
                                    warn!("{}", err);
                                }
                                ConnectionStatus::Active(Err(err))
                            },
                        }
                    },
                    Err(err) => ConnectionStatus::Active(Err(err)),
                }
            },
            ConnectionStatus::Closed(err) => {
                ConnectionStatus::Closed(err)
            },
        }
    }
    ///
    /// 
    pub fn deserialize(selfId: &str, txId: usize, bytes: Vec<u8>) -> Result<PointType, String> {
        fn parseDirection(selfId: &str, name: &str, obj: &serde_json::Map<String, serde_json::Value>) -> Direction {
            match obj.get("direction") {
                Some(value) => {
                    match serde_json::from_value(value.clone()) {
                        Ok(direction) => direction,
                        Err(err) => {
                            let message = format!("{}.parse | Deserialize Point.direction error: {:?} in the: {}:{:?}", selfId, err, name, value);
                            warn!("{}", message);
                            Direction::Read
                        },
                    }
                },
                None => Direction::Read,
            }
        }
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
                                        let value = obj.get("value").unwrap().as_bool().unwrap();
                                        let status = obj.get("status").unwrap().as_i64().unwrap();
                                        let direction = parseDirection(selfId, name, obj);
                                        let timestamp = obj.get("timestamp").unwrap().as_str().unwrap();
                                        let timestamp: DateTime<Utc> = chrono::DateTime::parse_from_rfc3339(timestamp).unwrap().with_timezone(&Utc);
                                        Ok(PointType::Bool(Point::new(
                                            txId,
                                            name,
                                            Bool(value),
                                            Status::from(status),
                                            direction,
                                            timestamp,
                                        )))
                                    },
                                    Some("int") | Some("Int") => {
                                        let name = obj.get("name").unwrap().as_str().unwrap();
                                        let value = obj.get("value").unwrap().as_i64().unwrap();
                                        let status = obj.get("status").unwrap().as_i64().unwrap();
                                        let direction = parseDirection(selfId, name, obj);
                                        let timestamp = obj.get("timestamp").unwrap().as_str().unwrap();
                                        let timestamp: DateTime<Utc> = chrono::DateTime::parse_from_rfc3339(timestamp).unwrap().with_timezone(&Utc);
                                        Ok(PointType::Int(Point::new(
                                            txId,
                                            name,
                                            value,
                                            Status::from(status),
                                            direction,
                                            timestamp,
                                        )))
                                    },
                                    Some("float") | Some("Float") => {
                                        let name = obj.get("name").unwrap().as_str().unwrap();
                                        let value = obj.get("value").unwrap().as_f64().unwrap();
                                        let status = obj.get("status").unwrap().as_i64().unwrap();
                                        let direction = parseDirection(selfId, name, obj);
                                        let timestamp = obj.get("timestamp").unwrap().as_str().unwrap();
                                        let timestamp: DateTime<Utc> = chrono::DateTime::parse_from_rfc3339(timestamp).unwrap().with_timezone(&Utc);
                                        Ok(PointType::Float(Point::new(
                                            txId,
                                            name,
                                            value,
                                            Status::from(status),
                                            direction,
                                            timestamp,
                                        )))
                                    },
                                    Some("string") | Some("String") => {
                                        let name = obj.get("name").unwrap().as_str().unwrap();
                                        let value = obj.get("value").unwrap().as_str().unwrap();
                                        let status = obj.get("status").unwrap().as_i64().unwrap();
                                        let direction = parseDirection(selfId, name, obj);
                                        let timestamp = obj.get("timestamp").unwrap().as_str().unwrap();
                                        let timestamp: DateTime<Utc> = chrono::DateTime::parse_from_rfc3339(timestamp).unwrap().with_timezone(&Utc);
                                        Ok(PointType::String(Point::new(
                                            txId,
                                            name,
                                            value.to_owned(),
                                            Status::from(status),
                                            direction,
                                            timestamp,
                                        )))
                                    },
                                    _ => {
                                        let message = format!("{}.parse | Unknown point type: {}", selfId, type_);
                                        trace!("{}", message);
                                        Err(message)
                                    }
                                }
                            },
                            None => {
                                let message = format!("{}.parse | JSON convertion error: mapping not found in the JSON: {}", selfId, value);
                                trace!("{}", message);
                                Err(message)        
                            },
                        }
                    },
                    None => {
                        let message = format!("{}.parse | JSON convertion error: mapping not found in the JSON: {}", selfId, value);
                        trace!("{}", message);
                        Err(message)
                    },
                }
            },
            Err(err) => {
                let message = format!("JdsDeserialize.parse | JSON convertion error: {:?}", err);
                trace!("{}", message);
                Err(message)        
            },
        }
    }    
}