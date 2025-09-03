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
use std::collections::VecDeque;

use sal_core::{dbg::Dbg, error::Error};
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
pub enum BuildField {
    /// u8 to be converted to byte
    U8(u8),
    /// u16 value to be converted to Big-ending bytes
    BeU16(u16),
    /// u16 value to be converted to Little-ending bytes
    LeU16(u16),
    /// u32 value to be converted to Big-ending bytes
    BeU32(u32),
    /// u32 value to be converted to Little-ending bytes
    LeU32(u32),
    /// u64 value to be converted to Big-ending bytes
    BeU64(u64),
    /// u64 value to be converted to Little-ending bytes
    LeU64(u64),
    /// u128 value to be converted to Big-ending bytes
    BeU128(u128),
    /// u128 value to be converted to Little-ending bytes
    LeU128(u128),

    /// i8 to be converted to Big-ending bytes
    BeI8(i8),
    /// i8 to be converted to Little-ending bytes
    LeI8(i8),
    /// i16 value to be converted to Big-ending bytes
    BeI16(i16),
    /// i16 value to be converted to Little-ending bytes
    LeI16(i16),
    /// i32 value to be converted to Big-ending bytes
    BeI32(i32),
    /// i32 value to be converted to Little-ending bytes
    LeI32(i32),
    /// i64 value to be converted to Big-ending bytes
    BeI64(i64),
    /// i64 value to be converted to Little-ending bytes
    LeI64(i64),
    /// i128 value to be converted to Big-ending bytes
    BeI128(i128),
    /// i128 value to be converted to Little-ending bytes
    LeI128(i128),

    /// f32 value to be converted to Big-ending bytes
    BeF32(f32),
    /// f32 value to be converted to Little-ending bytes
    LeF32(f32),
    /// f64 value to be converted to Big-ending bytes
    BeF64(f64),
    /// f64 value to be converted to Little-ending bytes
    LeF64(f64),
    /// Value passed by already converted to the bytes
    Data,
}
///
/// Socket Message
pub struct Message<'a, FieldIn, FieldOut> {
    build: Vec<BuildField>,
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
        build: Vec<BuildField>,
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
    pub fn build(&mut self, mut data: VecDeque<Vec<u8>>) -> Vec<u8> {
        let data_len = data.len();
        let mut message = vec![];
        for field in &self.build {
            match field {
                BuildField::U8(val) => message.push(*val),
                BuildField::BeU16(val) => message.extend(val.to_be_bytes()),
                BuildField::LeU16(val) => message.extend(val.to_le_bytes()),
                BuildField::BeU32(val) => message.extend(val.to_be_bytes()),
                BuildField::LeU32(val) => message.extend(val.to_le_bytes()),
                BuildField::BeU64(val) => message.extend(val.to_be_bytes()),
                BuildField::LeU64(val) => message.extend(val.to_le_bytes()),
                BuildField::BeU128(val) => message.extend(val.to_be_bytes()),
                BuildField::LeU128(val) => message.extend(val.to_le_bytes()),
                BuildField::BeI8(val) => message.extend(val.to_be_bytes()),
                BuildField::LeI8(val) => message.extend(val.to_le_bytes()),
                BuildField::BeI16(val) => message.extend(val.to_be_bytes()),
                BuildField::LeI16(val) => message.extend(val.to_le_bytes()),
                BuildField::BeI32(val) => message.extend(val.to_be_bytes()),
                BuildField::LeI32(val) => message.extend(val.to_le_bytes()),
                BuildField::BeI64(val) => message.extend(val.to_be_bytes()),
                BuildField::LeI64(val) => message.extend(val.to_le_bytes()),
                BuildField::BeI128(val) => message.extend(val.to_be_bytes()),
                BuildField::LeI128(val) => message.extend(val.to_le_bytes()),
                BuildField::BeF32(val) => message.extend(val.to_be_bytes()),
                BuildField::LeF32(val) => message.extend(val.to_le_bytes()),
                BuildField::BeF64(val) => message.extend(val.to_be_bytes()),
                BuildField::LeF64(val) => message.extend(val.to_le_bytes()),
                BuildField::Data => {
                    match data.pop_front() {
                        Some(bytes) => message.extend(bytes),
                        None => log::warn!(
                            "{}.build | Argument `data` expected length {} but found {} elements",
                            self.dbg,
                            self.build.iter().filter(|field| **field == BuildField::Data).count(),
                            data_len,
                        ),
                    }
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
