//!
//! # Messages transmitted over socket.
//! 
//! - Data can be encoded using varius data `Kind`, `Size` and payload Data
//! 
//! - Message format
//!     Field name | Start | Kind |  Size  | Data |
//!     ---       |  ---  | ---  |  ---   | ---  |
//!     Data type |  u8   | u8   | u32    | [u8; Size] |
//!     Value     |  22   | StringValue | xxx    | [..., ...]  |
//!     
//!     - Start - Each message starts with SYN (22)
//!     - Kind - The `Kind` of the data stored in the `Data` field, refer to
//!     - Size - The length of the `Data` field in bytes
//!     - Data - Data structured depending on it `Kind`
//! 
//! - `Kind` of data
//!     - 00, Any
//!     - 01, Empty
//!     - 02, Bytes
//!     - 08, Bool
//!     - 16, UInt16
//!     - 17, UInt32
//!     - 18, UInt64
//!     - 24, Int16
//!     - 25, Int32
//!     - 26, Int64
//!     - 32, F32
//!     - 33, F64
//!     - 40, String
//!     - 48, Timestamp
//!     - 49, Duration
//!     - .., ...
//! 
use sal_core::{dbg::Dbg, error::Error};
use super::fields::{FieldData, FieldId, FieldSize, FieldSyn};
///
/// 
pub type Bytes = Vec<u8>;
///
/// Parse Message structure from bytes Interface 
pub trait MessageParse<'a, FieldIn, FieldOut, Out> {
    ///
    /// Extracting some pattern from input `bytes`
    fn parse(&mut self, bytes: Bytes) -> Result<(FieldIn, FieldOut, Bytes), Error>;
}
/// 
/// 
#[derive(Debug, Clone, PartialEq)]
pub enum MessageField {
    Syn(FieldSyn),
    Id(FieldId),
    Size(FieldSize),
    Data(FieldData),
}
///
/// Socket Message
pub struct Message<'a, FieldIn, FieldOut> {
    build: Vec<MessageField>, 
    parse: Box<dyn MessageParse<'a, FieldIn, FieldOut, Bytes>>,
    remainder: Bytes,
    dbg: Dbg,
}

//
//
impl<'a, FieldIn, FieldOut> std::fmt::Debug for Message<'a, FieldIn, FieldOut> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Message")
            .field("dbg", &self.dbg)
            .field("build", &self.build)
            .finish()
    }
}
//
//
impl<'a, FieldIn, FieldOut> Message<'a, FieldIn, FieldOut> {
    ///
    /// Returns `Message` new instance 
    pub fn new(
        parent: impl Into<String>,
        build: Vec<MessageField>,
        parse: impl MessageParse<'a, FieldIn, FieldOut, Bytes> + 'static
    ) -> Self {
        Self {
            dbg: Dbg::new(parent.into(), "Message"),
            build,
            parse: Box::new(parse),
            remainder: vec![],
        }
    }
    ///
    /// Returns message built according to specified fields and passed `bytes`
    pub fn build(&mut self, bytes: &[u8], id: u32) -> Vec<u8> {
        let mut message = vec![];
        for field in &mut self.build {
            match field {
                MessageField::Syn(field_syn) => message.push(field_syn.0),
                MessageField::Id(_) => message.extend(FieldId(id).to_be_bytes()),
                // MessageField::Kind(field_kind) => message.extend(field_kind.to_bytes()),
                MessageField::Size(field_size) => message.extend(field_size.to_be_bytes(bytes.len() as u32)),
                MessageField::Data(_) => {
                    message.extend_from_slice(bytes);
                }
            }
        }
        message
    }
    ///
    /// Extracting [Message] fields from the input bytes
    /// - returns `Id`, `Kind`, `Size` & `Bytes` following by the `Size`
    /// - call this method multiple times, until the end of message
    pub fn parse(&mut self, bytes: Bytes) -> Result<(FieldIn, FieldOut), Error> {
        let bytes = [std::mem::take(&mut self.remainder), bytes].concat();
        match self.parse.parse(bytes) {
            Ok((din, dout, remainder)) => {
                self.remainder = remainder;
                Ok((din, dout))
            }
            Err(err) => Err(Error::new(&self.dbg, "parse").pass(err)),
        }
    }
}
