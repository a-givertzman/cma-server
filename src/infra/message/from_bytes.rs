use sal_core::error::Error;
///
/// 
pub trait FromBytes: Sized {
    fn from_bytes(bytes: &[u8]) -> Result<Self, Error>;
}
// ///
// /// 
// pub trait FromBytesLe: Sized {
//     fn from_bytes_le(bytes: &[u8]) -> Result<Self, Error>;
// }
// ///
// /// 
// pub trait FromBytesBe: Sized {
//     fn from_bytes_be(bytes: &[u8]) -> Result<Self, Error>;
// }
