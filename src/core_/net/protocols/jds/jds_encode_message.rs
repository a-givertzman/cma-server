use crate::{conf::point_config::name::Name, core_::{failure::recv_error::RecvError, object::object::Object}, tcp::steam_read::StreamRead};
use super::{jds_serialize::JdsSerialize, jds_define::JDS_END_OF_TRANSMISSION};
///
/// Converts json string into the bytes
/// adds Jds.endOfTransmission = 4 at the end of message
/// returns Result<Vec, Err>
#[derive(Debug)]
pub struct JdsEncodeMessage {
    id: String,
    name: Name,
    stream: JdsSerialize,
}
//
// 
impl JdsEncodeMessage {
    ///
    /// Creates new instance of the JdsEncodeMessage
    pub fn new(parent: impl Into<String>, stream: JdsSerialize) -> Self {
        let me = Name::new(parent, "JdsEncodeMessage");
        Self {
            id: me.join(),
            name: me,
            stream,
        }
    }
}
//
// 
impl Object for JdsEncodeMessage {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl StreamRead<Vec<u8>, RecvError> for JdsEncodeMessage {
    ///
    /// Returns sequence of bytes representing encoded single PointType, ends with Jds.endOfTransmission = 4
    fn read(&mut self) -> Result<Vec<u8>, RecvError> {
        let mut bytes = Vec::new();
        match self.stream.read() {
            Ok(value) => {
                match serde_json::to_writer(&mut bytes, &value) {
                    Ok(_) => {
                        bytes.push(JDS_END_OF_TRANSMISSION);
                        Ok(bytes)
                    }
                    Err(err) => Err(RecvError::Error(format!("{}.read | error: {:?}", self.id, err))),
                }
            }
            Err(err) => Err(err),
        }
    }
}
///
/// 
unsafe impl Sync for JdsEncodeMessage {}