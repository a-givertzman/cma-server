#![allow(non_snake_case)]
use super::c_slmp_const::{FrameType, ProcessorNumber, SlmpCommand, SlmpSubCommand, TimerValue};
///
/// SLMP Info structure in c-type.
/// Used in SLMP_MakePacketStream as argument.
/// It needs to be prepared like that because of C language and pointers.
/// - Frame type     -> ASCII/BIN + REQUEST/RESPONSE + ST/MT
/// - Serial number  -> used only with MT(multiple transmission) frame types, marks request and response with this number
/// - Net number     -> network number(1 to 239), can be fined in parameters of target device (e.g. SLMPNWNO)
/// - Node number    -> station number (1 to 120), can be fined in parameters of target device (e.g. SLMPNDID)
/// - Proc number    -> destination unit I/O number
/// - Data length    -> length of data from usTimer to end of pucData, use function get_data_length
/// - Timer          -> monitoring timer
/// - Command        -> SLMP command (e.g. SLMP_COMMAND_DEVICE_READ, SLMP_COMMAND_SELF_TEST)
/// - SubCommand     -> 0x0001/0x0003 = per bit, 0x0000/0x0002 = per word, 0x0080/0x0082 = per word on CPU
/// - EndCode        -> used mostly in response
/// - Data           -> data which obtains device number, number of devices, its like arguments of used command
#[repr(C)]
pub struct CSlmpInfo {
	ulFrameType     : std::ffi::c_ulong,			// unsigned long
	usSerialNumber  : std::ffi::c_ushort,			// unsigned short
	usNetNumber     : std::ffi::c_ushort,			// unsigned short
	usNodeNumber    : std::ffi::c_ushort,			// unsigned short
	usProcNumber    : std::ffi::c_ushort,			// unsigned short
	usDataLength    : std::ffi::c_ushort,			// unsigned short
	usTimer         : std::ffi::c_ushort,			// unsigned short
	usCommand       : std::ffi::c_ushort,			// unsigned short
	usSubCommand    : std::ffi::c_ushort,			// unsigned short
	usEndCode       : std::ffi::c_ushort,			// unsigned short
	pucData	        : *const std::ffi::c_uchar,		// unsigned char *
}
//
//
impl CSlmpInfo {
    pub fn new(
        ulFrameType     : FrameType,
        usSerialNumber  : u16,
        usNetNumber     : u16,
        usNodeNumber    : u16,
        usProcNumber    : ProcessorNumber,
        usDataLength    : u16,
        usTimer         : TimerValue,
        usCommand       : SlmpCommand,
        usSubCommand    : SlmpSubCommand,
        usEndCode       : u16,
        pucData	        : &[u8],
    ) -> Self {
        Self {
            ulFrameType: ulFrameType as  u64,
            usSerialNumber: usSerialNumber,
            usNetNumber: usNetNumber,
            usNodeNumber: usNodeNumber,
            usProcNumber: usProcNumber as u16,
            usDataLength: usDataLength,
            usTimer: usTimer as u16,
            usCommand: usCommand as u16,
            usSubCommand: usSubCommand as u16,
            usEndCode: usEndCode,
            pucData: pucData.as_ptr(),
        }
    }
}
