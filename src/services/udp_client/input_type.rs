use std::fmt::Display;

use sal_core::error::Error;

///
/// `TYPE` - type of values in the array in `DATA` field
///   - 8 - u8, 1 byte unsigned integer value
///   - 9 - i8, 1 byte signed integer value
///   - 16 - u16, 2 byte unsigned integer value
///   - 17 - i16, 2 byte signed integer value
///   - 32 - u32, 4 byte unsigned integer value
///   - 33 - i32, 4 byte signed integer value
///   - 132 - f32, 4 bytes float value
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum InputType {
    U8 = 8,
    I8 = 9,
    U16 = 16,
    I16 = 17,
    U32 = 32,
    I32 = 33,
    F32 = 132,
}
//
//
impl InputType {
    ///
    /// Returns size of the [InputType] in bytes
    pub fn size(&self) -> usize {
        match self {
            InputType::U8 => 1,
            InputType::I8 => 1,
            InputType::U16 => 2,
            InputType::I16 => 2,
            InputType::U32 => 4,
            InputType::I32 => 4,
            InputType::F32 => 4,
        }
    }
}
//
//
impl TryFrom<u8> for InputType {
    type Error = Error;
    ///
    /// Returns [InputType] created from it's raw value
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            val if val == InputType::U8 as u8 => Ok(InputType::U8),
            val if val == InputType::I8 as u8 => Ok(InputType::I8),
            val if val == InputType::U16 as u8 => Ok(InputType::U16),
            val if val == InputType::I16 as u8 => Ok(InputType::I16),
            val if val == InputType::U32 as u8 => Ok(InputType::U32),
            val if val == InputType::I32 as u8 => Ok(InputType::I32),
            val if val == InputType::F32 as u8 => Ok(InputType::F32),
            _ => Err(Error::new("InputType", "new").err(format!("Unknown InputType value {val}")))
        }
    }
}
//
//
impl Default for InputType {
    fn default() -> Self {
        Self::U16
    }
}
//
//
impl Display for InputType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputType::U8 => write!(f, "U8"),
            InputType::I8 => write!(f, "I8"),
            InputType::U16 => write!(f, "U16"),
            InputType::I16 => write!(f, "I16"),
            InputType::U32 => write!(f, "U32"),
            InputType::I32 => write!(f, "I32"),
            InputType::F32 => write!(f, "F32"),
        }
    }
}