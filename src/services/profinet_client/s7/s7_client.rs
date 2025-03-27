use snap7_sys::S7Object;
use std::ffi::CString;
use std::ffi::{c_void, c_int};

use super::s7_error::S7Error;
use super::s7_lib::S7LIB;

///
/// Ethrnet access to the PROFINET device
#[derive(Debug)]
pub struct S7Client {
    pub id: String,
    ip: CString,
    handle: S7Object,
    req_len: usize,
    neg_len: usize,
    // isConnected: bool,
    // reconnectDelay: Duration,
}
//
// 
impl S7Client {
    ///
    /// Creates new instance of the S7Client
    pub fn new(parent: impl Into<String>, ip: String) -> Self {
        Self {
            id: format!("{}/S7Client({})", parent.into(), ip),
            ip: CString::new(ip).unwrap(),
            handle: unsafe { S7LIB.Cli_Create() },
            req_len: 0,
            neg_len: 0,
            // isConnected: false,
        }
    }
    ///
    /// Connects the client to the PLC
    pub fn connect(&mut self) -> Result<(), S7Error> {
        let mut req: c_int = 0;
        let mut neg: c_int = 0;
        let err_code = unsafe {
            // #[warn(temporary_cstring_as_ptr)]
            let err_code = S7LIB.Cli_ConnectTo(self.handle, self.ip.as_ptr(), 0, 1);
            S7LIB.Cli_GetPduLength(self.handle, &mut req, &mut neg);
            self.req_len = req as usize;
            self.neg_len = neg as usize;
            err_code
        };
        if err_code == 0 {
            // self.isConnected = true;
            log::debug!("{}.connect | successfully connected", self.id);
            Ok(())
        } else {
            // self.isConnected = false;
            let err = S7Error::from(err_code);
            if log::max_level() == log::LevelFilter::Trace {
                log::warn!("{}.connect | connection error: {:?}", self.id, err);
            }
            Err(err)
        }
    }
    ///
    /// Returns the connection status
    pub fn is_connected(&self) -> Result<bool, String> {
        let mut is_connected: c_int = 0;
        let code = unsafe {
            S7LIB.Cli_GetConnected(self.handle, &mut is_connected)
        };
        match code {
            0 => Ok(is_connected != 0),
            _ => Err(S7Error::text(code))
        }
    }
    ///
    /// This is the main function to read data from a PLC.
    /// With it you can read DB, Inputs, Outputs, Merkers, Timers and Counters
    pub fn read(&self, db_num: u32, start: u32, size: u32) -> Result<Vec<u8>, String> {
        let mut buf = vec![0; size as usize];
        let code;
        unsafe {
            code = S7LIB.Cli_DBRead(
                self.handle,
                db_num as c_int,
                start as c_int,
                size as c_int,
                buf.as_mut_ptr() as *mut c_void,
            );
        }
        match code {
            0 => Ok(buf),
            _ => Err(S7Error::text(code)),
        }
    }
    ///
    /// This is the main function to write data into a PLC. It’s the complementary function of
    /// Cli_ReadArea(), the parameters and their meanings are the same.
    /// The only difference is that the data is transferred from the buffer pointed by pUsrData
    /// into PLC.
    pub fn write(&self, db_num: u32, start: u32, size: u32, buf: &mut [u8]) -> Result<(), String> {
        let code = unsafe {
            S7LIB.Cli_DBWrite(
                self.handle,
                db_num as c_int, 
                start as c_int, 
                size as c_int, 
                buf.as_mut_ptr() as *mut c_void,
            )
        };
        match code {
            0 => Ok(()),
            _ => Err(S7Error::text(code)),
        }
    }
    ///
    /// Disconnects “gracefully” the Client from the PLC.
    pub fn close(&mut self) -> Result<(), String> {
        let code = unsafe {
            S7LIB.Cli_Disconnect(self.handle)
        };
        match code {
            0 => Ok(()),
            _ => Err(S7Error::text(code)),
        }
    }
}
//
// 
impl Drop for S7Client {
    fn drop(&mut self) {
        unsafe {
            S7LIB.Cli_Destroy(&mut self.handle);
        }
    }
}
