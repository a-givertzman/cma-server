#![allow(non_snake_case)]

use super::jds_serialize::JdsSerialize;


pub const JDS_END_OF_TRANSMISSION: u8 = 0x4;

///
/// Converts json string into the bytes
/// adds Jds.endOfTransmission = 4 at the end of message
/// returns Result<Vec, Err>
pub struct JdsEncodeMessage {
    id: String,
    stream: JdsSerialize,
}
///
/// 
impl JdsEncodeMessage {
    ///
    /// Creates new instance of the JdsEncodeMessage
    pub fn new(parent: impl Into<String>, stream: JdsSerialize) -> Self {
        Self {
            id: format!("{}/JdsMessage", parent.into()),
            stream,
        }
    }
    ///
    /// Returns sequence of bytes representing encoded single PointType, ends with Jds.endOfTransmission = 4
    pub fn read(&mut self) -> Result<Vec<u8>, String> {
        let mut bytes = Vec::new();
        match self.stream.read() {
            Ok(value) => {
                match serde_json::to_writer(&mut bytes, &value) {
                    Ok(_) => {
                        Ok(bytes)
                    },
                    Err(err) => Err(format!("{}.read | error: {:?}", self.id, err)),
                }
            },
            Err(err) => Err(err),
        }
    }
}
