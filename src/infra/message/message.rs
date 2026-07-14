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
use crate::infra::message::{Field, FieldConf};
///
/// 
pub type Bytes = Vec<u8>;
///
/// Parse Message structure from bytes Interface 
pub trait MessageParse<FieldIn, FieldOut, Out> {
    ///
    /// Extracting some pattern from input `bytes`
    fn parse(&mut self, bytes: Bytes) -> Result<(FieldIn, FieldOut, Bytes), Error>;
}
///
/// Socket Message
pub struct Message<FieldIn, FieldOut> {
    build: Vec<FieldConf>,
    parse: Box<dyn MessageParse<FieldIn, FieldOut, Bytes>>,
    remainder: Bytes,
    dbg: Dbg,
}

//
//
impl<FieldIn, FieldOut> std::fmt::Debug for Message<FieldIn, FieldOut> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Message")
            .field("dbg", &self.dbg)
            .field("build", &self.build)
            .finish()
    }
}
//
//
impl<FieldIn, FieldOut> Message<FieldIn, FieldOut> {
    ///
    /// Returns `Message` new instance 
    pub fn new(
        parent: impl Into<String>,
        build: Vec<FieldConf>,
        parse: impl MessageParse<FieldIn, FieldOut, Bytes> + 'static
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
    pub fn build(&mut self, data: &[Field]) -> Vec<u8> {
        let mut message = vec![];
        for (i, field) in self.build.iter().enumerate() {
            match field {
                FieldConf::Const(bytes) => message.extend(bytes),
                FieldConf::U16Be => match data.get(i) {
                    Some(field) => message.extend(field.as_u16().to_be_bytes()),
                    None => log::error!("{}.build | 'Field::U16Be' configured with index [{i}], but input fields contains only {} elements", self.dbg, data.len()),
                },
                FieldConf::U16Le => match data.get(i) {
                    Some(field) => message.extend(field.as_u16().to_le_bytes()),
                    None => log::error!("{}.build | 'Field::U16Be' configured with index [{i}], but input fields contains only {} elements", self.dbg, data.len()),
                },
                FieldConf::U32Be => match data.get(i) {
                    Some(field) => message.extend(field.as_u32().to_be_bytes()),
                    None => log::error!("{}.build | 'Field::U32Be' configured with index [{i}], but input fields contains only {} elements", self.dbg, data.len()),
                },
                FieldConf::U32Le => match data.get(i) {
                    Some(field) => message.extend(field.as_u32().to_le_bytes()),
                    None => log::error!("{}.build | 'Field::U32Le' configured with index [{i}], but input fields contains only {} elements", self.dbg, data.len()),
                },
                FieldConf::U64Be => match data.get(i) {
                    Some(field) => message.extend(field.as_u64().to_be_bytes()),
                    None => log::error!("{}.build | 'Field::U64Be' configured with index [{i}], but input fields contains only {} elements", self.dbg, data.len()),
                },
                FieldConf::U64Le => match data.get(i) {
                    Some(field) => message.extend(field.as_u64().to_le_bytes()),
                    None => log::error!("{}.build | 'Field::U64Le' configured with index [{i}], but input fields contains only {} elements", self.dbg, data.len()),
                },
                FieldConf::U128Be => match data.get(i) {
                    Some(field) => message.extend(field.as_u128().to_be_bytes()),
                    None => log::error!("{}.build | 'Field::U128Be' configured with index [{i}], but input fields contains only {} elements", self.dbg, data.len()),
                },
                FieldConf::U128Le => match data.get(i) {
                    Some(field) => message.extend(field.as_u128().to_le_bytes()),
                    None => log::error!("{}.build | 'Field::U128Le' configured with index [{i}], but input fields contains only {} elements", self.dbg, data.len()),
                },
                FieldConf::I8Be => match data.get(i) {
                    Some(field) => message.extend(field.as_i8().to_be_bytes()),
                    None => log::error!("{}.build | 'Field::I8Be' configured with index [{i}], but input fields contains only {} elements", self.dbg, data.len()),
                },
                FieldConf::I8Le => match data.get(i) {
                    Some(field) => message.extend(field.as_i8().to_le_bytes()),
                    None => log::error!("{}.build | 'Field::I8Le' configured with index [{i}], but input fields contains only {} elements", self.dbg, data.len()),
                },
                FieldConf::I16Be => match data.get(i) {
                    Some(field) => message.extend(field.as_i16().to_be_bytes()),
                    None => log::error!("{}.build | 'Field::I16Be' configured with index [{i}], but input fields contains only {} elements", self.dbg, data.len()),
                },
                FieldConf::I16Le => match data.get(i) {
                    Some(field) => message.extend(field.as_i16().to_le_bytes()),
                    None => log::error!("{}.build | 'Field::I16Le' configured with index [{i}], but input fields contains only {} elements", self.dbg, data.len()),
                },
                FieldConf::I32Be => match data.get(i) {
                    Some(field) => message.extend(field.as_i32().to_be_bytes()),
                    None => log::error!("{}.build | 'Field::I32Be' configured with index [{i}], but input fields contains only {} elements", self.dbg, data.len()),
                },
                FieldConf::I32Le => match data.get(i) {
                    Some(field) => message.extend(field.as_i32().to_le_bytes()),
                    None => log::error!("{}.build | 'Field::I32Le' configured with index [{i}], but input fields contains only {} elements", self.dbg, data.len()),
                },
                FieldConf::I64Be => match data.get(i) {
                    Some(field) => message.extend(field.as_i64().to_be_bytes()),
                    None => log::error!("{}.build | 'Field::I64Be' configured with index [{i}], but input fields contains only {} elements", self.dbg, data.len()),
                },
                FieldConf::I64Le => match data.get(i) {
                    Some(field) => message.extend(field.as_i64().to_le_bytes()),
                    None => log::error!("{}.build | 'Field::I64Le' configured with index [{i}], but input fields contains only {} elements", self.dbg, data.len()),
                },
                FieldConf::I128Be => match data.get(i) {
                    Some(field) => message.extend(field.as_i128().to_be_bytes()),
                    None => log::error!("{}.build | 'Field::I128Be' configured with index [{i}], but input fields contains only {} elements", self.dbg, data.len()),
                },
                FieldConf::I128Le => match data.get(i) {
                    Some(field) => message.extend(field.as_i128().to_le_bytes()),
                    None => log::error!("{}.build | 'Field::I128Le' configured with index [{i}], but input fields contains only {} elements", self.dbg, data.len()),
                },
                FieldConf::F32Be => match data.get(i) {
                    Some(field) => message.extend(field.as_f32().to_be_bytes()),
                    None => log::error!("{}.build | 'Field::F32Be' configured with index [{i}], but input fields contains only {} elements", self.dbg, data.len()),
                },
                FieldConf::F32Le => match data.get(i) {
                    Some(field) => message.extend(field.as_f32().to_le_bytes()),
                    None => log::error!("{}.build | 'Field::F32Le' configured with index [{i}], but input fields contains only {} elements", self.dbg, data.len()),
                },
                FieldConf::F64Be => match data.get(i) {
                    Some(field) => message.extend(field.as_f64().to_be_bytes()),
                    None => log::error!("{}.build | 'Field::F64Be' configured with index [{i}], but input fields contains only {} elements", self.dbg, data.len()),
                },
                FieldConf::F64Le => match data.get(i) {
                    Some(field) => message.extend(field.as_f64().to_le_bytes()),
                    None => log::error!("{}.build | 'Field::F64Le' configured with index [{i}], but input fields contains only {} elements", self.dbg, data.len()),
                },
                FieldConf::String => match data.get(i) {
                    Some(field) => message.extend(field.as_string().as_bytes()),
                    None => log::error!("{}.build | 'Field::String' configured with index [{i}], but input fields contains only {} elements", self.dbg, data.len()),
                },
                FieldConf::Byte => match data.get(i) {
                    Some(field) => message.push(*field.as_byte()),
                    None => log::error!("{}.build | 'Field::Byte' configured with index [{i}], but input fields contains only {} elements", self.dbg, data.len()),
                },
                FieldConf::Bytes => match data.get(i) {
                    Some(field) => message.extend(field.as_bytes()),
                    None => log::error!("{}.build | 'Field::Byte' configured with index [{i}], but input fields contains only {} elements", self.dbg, data.len()),
                },
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
