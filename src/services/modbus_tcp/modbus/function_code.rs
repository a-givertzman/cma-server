use std::str::FromStr;
use sal_core::error::Error;

///
/// Modbus message Function Code
/// 
/// Code 01,  Read Coil Status:           0xxxx,  register address = 00001 + offset
/// Code 02,  Read Input Status:          1xxxx,  register address = 10001 + offset
/// Code 03,  Read Holding Registers:     4xxxx,  register address = 40001 + offset
/// Code 04,  Read Input Registers:       3xxxx,  register address = 30001 + offset
/// Code 05,  Force Single Coil:          0xxxx,  register address = 00001 + offset
/// Code 06,  Preset Single Register:     4xxxx,  register address = 40001 + offset
/// Code 15,  Force Multiple Coils:       0xxxx,  register address = 00001 + offset
/// Code 16,  Preset Multiple Registers:  4xxxx,  register address = 40001 + offset
/// 
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct FunctionCode(pub u8);
//
//
impl FromStr for FunctionCode {
    type Err = Error;
    ///
    /// Returns [FunctionCode] parsed from string
    fn from_str(code: &str) -> Result<Self, Self::Err> {
        match code.parse() {
            Ok(code) => Ok(Self(code)),
            Err(err) => Err(
                Error::new("FunctionCode", "from_str").pass_with(format!(" | Can't parse Modbus Function Code from {code}"), err.to_string())
            ),
        }
    }
}
