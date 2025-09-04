use sal_core::{dbg::Dbg, error::Error};
use crate::infra::message::{Field, FieldConf, FieldTerminator, FixedField, Message, SizedField};

///
/// # Represents a Modbus network message
/// 
/// - Parse Modbus message fields from bytes
/// - Build bytes from Modbus message fields
/// 
/// ## Message format
/// 
/// ```ignore
///  Transaction ID | Protocol ID | Length Field |  Unit ID | Function Code | Data
///  ---            | ---         | ---          | ---      | ---           | ---
///   2 Bytes       | 2 Bytes     | 2 Bytes      | 1 Bytes  | 1 Byte        | Vec<u8>
/// ```

pub struct ModbusMessage {
    /// Transaction Identifier, auto incremented for each next message
    transaction: u16,
    message: Message<(((((((), ()), u16), u16), u16), u8), u8), Vec<u8>>,
    dbg: Dbg,
}
//
//
impl ModbusMessage {
    ///
    /// Returns [ModbusMessage] new instance
    pub fn new(parent: impl Into<String>) -> Self {
        let dbg = Dbg::new(parent, "ModbusMessage");
        Self {
            transaction: 0,
            message: Message::new(
                &dbg,
                vec![
                    // FieldConf::U16Be,        // Transaction Identifier u16       , index 0
                    // FieldConf::Const(vec![0x00, 0x00]),   // Protocol Identifier u16
                    // FieldConf::U16Be,        // Length Field u16                 , index 1
                    FieldConf::Byte,         // Unit ID, u8                      , index 2
                    FieldConf::Byte,         // Function Code, u8                , index 3
                    FieldConf::String,       // Bytes, Vec<u8>                   , index 4
                ],
                SizedField::new(
                    &dbg,
                    |((_, size), _), _| *size as usize,
                    move |_, bytes| {
                        Ok(bytes.to_vec())
                    },
                    FixedField::new(
                        &dbg, 1,    // Function Code 4, u8
                        move |dbg, bytes| match bytes.first() {
                            Some(byte) => Ok(*byte),
                            None => Err(Error::new(dbg, "from_bytes").err(format!("Can't parse u8 from bytes {:?}", bytes))),
                        },
                        FixedField::new(
                            &dbg, 1,    // Unit ID 33, u8
                            move |dbg, bytes| match bytes.first() {
                                Some(val) => Ok(*val),
                                None => Err(Error::new(dbg, "from_bytes").err(format!("Can't parse u8 from bytes {:?}", bytes))),
                            },
                            FixedField::new(
                                &dbg, 2,    // Length Field u16
                                move |dbg, bytes| match bytes.try_into() {
                                    Ok(bytes) => Ok(u16::from_be_bytes(bytes)),
                                    Err(err) => Err(Error::new(dbg, "from_bytes").pass_with(format!("Can't parse u16 from bytes {:?}", bytes), err.to_string())),
                                },
                                FixedField::new(
                                    &dbg, 2,    // Protocol Identifier u16
                                    move |dbg, bytes| match bytes.try_into() {
                                        Ok(bytes) => Ok(u16::from_be_bytes(bytes)),
                                        Err(err) => Err(Error::new(dbg, "from_bytes").pass_with(format!("Can't parse u16 from bytes {:?}", bytes), err.to_string())),
                                    },
                                    FixedField::new(
                                        &dbg, 2,    // Transaction Identifier u16
                                        move |dbg, bytes| match bytes.try_into() {
                                            Ok(bytes) => Ok(u16::from_be_bytes(bytes)),
                                            Err(err) => Err(Error::new(dbg, "from_bytes").pass_with(format!("Can't parse u16 from bytes {:?}", bytes), err.to_string())),
                                        },
                                        FieldTerminator::new(),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
            dbg,
        }
    }
    ///
    /// ## Returns Modbus fields parsed from `bytes`
    /// - `transaction` - u16, Transaction Identifier
    /// - `size` - u16, Length of the data field
    /// - `unit` - u8, Modbus Unit ID
    /// - `code` - u8, Function code
    /// - `bytes` - bytes of the data field
    pub fn parse(&mut self, bytes: Vec<u8>) -> Result<(u16, u16, u8, u8, Vec<u8>), Error> {
        match self.message.parse(bytes) {
            Ok(((((((_, transaction), _), size), unit), code), bytes)) => Ok((transaction, size, unit, code, bytes)),
            Err(err) => Err(Error::new(&self.dbg, "parse").pass(err)),
        }
    }
    ///
    /// ## Returns Modbus message bytes built from fields:
    /// - `unit` - u8, Modbus Unit ID
    /// - `code` - u8, Function code
    /// - `start` - Address of the first register (40108-40001 = 107 = 6B hex)
    /// - `count` - The number of required registers (reading 3 registers from 40108 to 40110)
    pub fn build(&mut self, unit: u8, code: u8, start: u16, count: u16) -> Vec<u8> {
        self.transaction += 1;
        self.message.build(&[
            Field::Byte(unit),
            Field::Byte(code),
            Field::Bytes([start.to_be_bytes(), count.to_be_bytes()].concat())
        ])
    }
}
