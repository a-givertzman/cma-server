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
impl Default for InputType {
    fn default() -> Self {
        Self::U16
    }
}