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
use std::{fmt::Debug, usize};
use sal_core::{dbg::Dbg, error::Error};
///
/// 
pub type Bytes = Vec<u8>;
///
/// 
pub trait ToBytes {
    fn to_be_bytes(&self) -> impl Iterator<Item = u8>;
    fn to_le_bytes(&self) -> impl Iterator<Item = u8>;
}
///
/// Parse Message structure from bytes Interface 
pub trait MessageParse<'a, FieldIn, FieldOut, Out> {
    ///
    /// Extracting some pattern from input `bytes`
    fn parse(&mut self, bytes: Bytes) -> Result<(FieldIn, FieldOut, Bytes), Error>;
}
/// 
/// Filed configuration for the [Message].build
/// 
/// Use such fields to specify a sequence of fields in the message built from values
/// 
/// ```ignire
/// vec![
///     ConstBe(64u16),    // First field (length 2 bytes) always contains value 64 as Big-ending bytes
///     ValueBe(0),        // Second field (length defines by input value type) coming feom input array in the index 0 contains value to be converted into Big-ending bytes
///     ConstBe(12u16),    // Therd field (length 2 bytes) always contains value 12 as Big-ending bytes
///     ValueBe(1),        // Forth field (length defines by input value type) coming feom input array in the index 0 contains value to be converted into Big-ending bytes
/// ]
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum ConfField {
    /// Const u8 to be converted to byte
    Const(Vec<u8>),
    /// Variable u8 to be converted to byte
    ValueBe(FieldParam),
    /// Variable u8 to be converted to byte
    ValueLe(FieldParam),
    /// Value passed by already converted to the bytes
    Byte(usize),
    /// Value passed by already converted to the bytes
    Bytes(usize),
}
#[derive(Debug, Clone, PartialEq)]
struct FieldParam {
    pub index: usize,
    pub length: usize,
    pub signed: bool,
}
impl FieldParam {
    pub fn to_be_bytes<'a>(&'a self, val: usize) -> Vec<u8> {
        match self.length {
            1 => match self.signed {
                true => (val as i8).to_be_bytes().to_vec(),
                false => (val as u8).to_be_bytes().to_vec(),
            }
            2 => match self.signed {
                true => (val as i16).to_be_bytes().to_vec(),
                false => (val as u16).to_be_bytes().to_vec(),
            }
            4 => match self.signed {
                true => (val as i32).to_be_bytes().to_vec(),
                false => (val as u32).to_be_bytes().to_vec(),
            }
            8 => match self.signed {
                true => (val as i64).to_be_bytes().to_vec(),
                false => (val as u64).to_be_bytes().to_vec(),
            }
            _ => panic!(),
        }
    }
    pub fn to_le_bytes(&self, val: usize) -> Vec<u8> {
        match self.length {
            1 => match self.signed {
                true => (val as i8).to_le_bytes().to_vec(),
                false => (val as u8).to_le_bytes().to_vec(),
            }
            2 => match self.signed {
                true => (val as i16).to_le_bytes().to_vec(),
                false => (val as u16).to_le_bytes().to_vec(),
            }
            4 => match self.signed {
                true => (val as i32).to_le_bytes().to_vec(),
                false => (val as u32).to_le_bytes().to_vec(),
            }
            8 => match self.signed {
                true => (val as i64).to_le_bytes().to_vec(),
                false => (val as u64).to_le_bytes().to_vec(),
            }
            _ => panic!(),
        }
    }
}
// impl FieldParam<4> {
//     pub fn to_be_bytes(&self, val: impl ToBytes) -> [u8; 4] {
//         val.to_be_bytes()
//     }
//     pub fn to_le_bytes(&self, val: impl ToBytes) -> [u8; 4] {
//         val.to_le_bytes()
//     }
// }
/// 
/// 
#[derive(Debug, Clone, PartialEq)]
pub enum Field<T> {
    /// Variable u8 to be converted to byte
    ValueBe(T),
    /// Variable u8 to be converted to byte
    ValueLe(T),
    /// Value passed by already converted to the bytes
    Byte(u8),
    /// Value passed by already converted to the bytes
    Bytes(Vec<u8>),
}
///
/// Socket Message
pub struct Message<'a, FieldIn, FieldOut> {
    build: Vec<ConfField>,
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
        build: Vec<ConfField>,
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
    pub fn build<T: ToBytes + Debug>(&mut self, data: &[Field<T>]) -> Vec<u8> {
        let mut message = vec![];
        for field in &self.build {
            match field {
                ConfField::Const(val) => message.extend(val),
                ConfField::ValueBe(field) => match data.get(field.index) {
                    Some(val) => message.extend(field.to_be_bytes(val)),
                    None => todo!(),
                },
                ConfField::ValueLe(val) => message.extend(val.to_le_bytes()),
                ConfField::Byte(i) => match data.get(*i) {
                    Some(field) => match field {
                        Field::Byte(byte) => message.push(*byte),
                        _ => log::error!("{}.build | 'Field::Byte' expected in the field [{i}], but found {:?}", self.dbg, field),
                    }
                    None => log::error!("{}.build | 'Field::Byte' configured with index [{i}], but input fields contains only {} elements", self.dbg, data.len()),
                }
                ConfField::Bytes(i) => match data.get(*i) {
                    Some(field) => match field {
                        Field::Bytes(byte) => message.extend_from_slice(byte),
                        _ => log::error!("{}.build | 'Field::Bytes' expected in the field [{i}], but found {:?}", self.dbg, field),
                    }
                    None => log::error!("{}.build | 'Field::Bytes' configured with index [{i}], but input fields contains only {} elements", self.dbg, data.len()),
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
//
// //
// impl ToBytes for u16 {
//     fn to_be_bytes<const N: usize>(&self) -> [u8; N] {
//         u16::to_be_bytes(*self)
//     }
//     fn to_le_bytes(&self) -> [u8; 2] {
//         u16::to_le_bytes(*self).into_iter()
//     }
// }
// impl ToBytes for &u16 {
//     fn to_be_bytes(&self) -> [u8; 2] {
//         u16::to_be_bytes(**self)
//     }
//     fn to_le_bytes(&self) -> [u8; 2] {
//         u16::to_le_bytes(**self)
//     }
// }