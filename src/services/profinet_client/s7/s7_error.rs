#![allow(non_snake_case)]

use std::ffi::{c_char, c_int};

use super::s7_lib::S7LIB;


/// Snap7 documented Error codes
/// - Please refer to code of the function ErrorText() for the explanation
/// - source: https://snap7.sourceforge.net/sharp7.html
#[derive(Debug)]
#[repr(i32)]
pub enum S7Error {
    TCPSocketCreation         = 0x00000001,
    TCPConnectionTimeout      = 0x00000002,
    TCPConnectionFailed       = 0x00000003,
    TCPReceiveTimeout         = 0x00000004,
    TCPDataReceive            = 0x00000005,
    TCPSendTimeout            = 0x00000006,
    TCPDataSend               = 0x00000007,
    TCPConnectionReset        = 0x00000008,
    TCPNotConnected           = 0x00000009,
    TCPUnreachableHost        = 0x00002751,
    IsoConnect                = 0x00010000,
    IsoInvalidPDU             = 0x00030000,
    IsoInvalidDataSize        = 0x00040000,
    CliNegotiatingPDU         = 0x00100000,
    CliInvalidParams          = 0x00200000,
    CliJobPending             = 0x00300000,
    CliTooManyItems           = 0x00400000,
    CliInvalidWordLen         = 0x00500000,
    CliPartialDataWritten     = 0x00600000,
    CliSizeOverPDU            = 0x00700000,
    CliInvalidPlcAnswer       = 0x00800000,
    CliAddressOutOfRange      = 0x00900000,
    CliInvalidTransportSize   = 0x00A00000,
    CliWriteDataSizeMismatch  = 0x00B00000,
    CliItemNotAvailable       = 0x00C00000,
    CliInvalidValue           = 0x00D00000,
    CliCannotStartPLC         = 0x00E00000,
    CliAlreadyRun             = 0x00F00000,
    CliCannotStopPLC          = 0x01000000,
    CliCannotCopyRamToRom     = 0x01100000,
    CliCannotCompress         = 0x01200000,
    CliAlreadyStop            = 0x01300000,
    CliFunNotAvailable        = 0x01400000,
    CliUploadSequenceFailed   = 0x01500000,
    CliInvalidDataSizeRecvd   = 0x01600000,
    CliInvalidBlockType       = 0x01700000,
    CliInvalidBlockNumber     = 0x01800000,
    CliInvalidBlockSize       = 0x01900000,
    CliNeedPassword           = 0x01D00000,
    CliInvalidPassword        = 0x01E00000,
    CliNoPasswordToSetOrClear = 0x01F00000,
    CliJobTimeout             = 0x02000000,
    CliPartialDataRead        = 0x02100000,
    CliBufferTooSmall         = 0x02200000,
    CliFunctionRefused        = 0x02300000,
    CliDestroying             = 0x02400000,
    CliInvalidParamNumber     = 0x02500000,
    CliCannotChangeParam      = 0x02600000,
    CliFunctionNotImplemented = 0x02700000,
    #[allow(unused)]
    Inner(String),
}
//
// 
impl S7Error {
    pub fn text(code: i32) -> String {
        let mut err = vec![0; 1024];
        unsafe {
            S7LIB.Cli_ErrorText(
                code as c_int,
                err.as_mut_ptr() as *mut c_char,
                err.len() as c_int,
            );
        }
        if let Some(i) = err.iter().position(|&r| r == 0) {
            err.truncate(i);
        }
        let err = unsafe { std::str::from_utf8_unchecked(&err) };
        err.to_owned()
    }    
}
//
// 
impl From<i32> for S7Error {
    fn from(value: i32) -> Self {
        match value {
            0x00000001 => Self::TCPSocketCreation,
            0x00000002 => Self::TCPConnectionTimeout,
            0x00000003 => Self::TCPConnectionFailed,
            0x00000004 => Self::TCPReceiveTimeout,
            0x00000005 => Self::TCPDataReceive,
            0x00000006 => Self::TCPSendTimeout,
            0x00000007 => Self::TCPDataSend,
            0x00000008 => Self::TCPConnectionReset,
            0x00000009 => Self::TCPNotConnected,
            0x00002751 => Self::TCPUnreachableHost,
            0x00010000 => Self::IsoConnect,
            0x00030000 => Self::IsoInvalidPDU,
            0x00040000 => Self::IsoInvalidDataSize,
            0x00100000 => Self::CliNegotiatingPDU,
            0x00200000 => Self::CliInvalidParams,
            0x00300000 => Self::CliJobPending,
            0x00400000 => Self::CliTooManyItems,
            0x00500000 => Self::CliInvalidWordLen,
            0x00600000 => Self::CliPartialDataWritten,
            0x00700000 => Self::CliSizeOverPDU,
            0x00800000 => Self::CliInvalidPlcAnswer,
            0x00900000 => Self::CliAddressOutOfRange,
            0x00A00000 => Self::CliInvalidTransportSize,
            0x00B00000 => Self::CliWriteDataSizeMismatch,
            0x00C00000 => Self::CliItemNotAvailable,
            0x00D00000 => Self::CliInvalidValue,
            0x00E00000 => Self::CliCannotStartPLC,
            0x00F00000 => Self::CliAlreadyRun,
            0x01000000 => Self::CliCannotStopPLC,
            0x01100000 => Self::CliCannotCopyRamToRom,
            0x01200000 => Self::CliCannotCompress,
            0x01300000 => Self::CliAlreadyStop,
            0x01400000 => Self::CliFunNotAvailable,
            0x01500000 => Self::CliUploadSequenceFailed,
            0x01600000 => Self::CliInvalidDataSizeRecvd,
            0x01700000 => Self::CliInvalidBlockType,
            0x01800000 => Self::CliInvalidBlockNumber,
            0x01900000 => Self::CliInvalidBlockSize,
            0x01D00000 => Self::CliNeedPassword,
            0x01E00000 => Self::CliInvalidPassword,
            0x01F00000 => Self::CliNoPasswordToSetOrClear,
            0x02000000 => Self::CliJobTimeout,
            0x02100000 => Self::CliPartialDataRead,
            0x02200000 => Self::CliBufferTooSmall,
            0x02300000 => Self::CliFunctionRefused,
            0x02400000 => Self::CliDestroying,
            0x02500000 => Self::CliInvalidParamNumber,
            0x02600000 => Self::CliCannotChangeParam,
            0x02700000 => Self::CliFunctionNotImplemented,
            _ => {
                Self::Inner(format!("{} ({})", Self::text(value), value))
            }
        }
    }
}