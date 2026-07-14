use sal_core::error::Error;
use snap7_sys::S7Object;
use std::ffi::CString;
use std::ffi::{c_void, c_int};
use std::sync::atomic::{AtomicBool, Ordering};

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
    is_connected: AtomicBool,
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
            is_connected: AtomicBool::new(false),
        }
    }
    ///
    /// Connects the client to the PLC
    pub fn connect(&mut self) -> Result<(), Error> {
        let _ = self.close();
        let err_code = unsafe {
            // #[warn(temporary_cstring_as_ptr)]
            let err_code = S7LIB.Cli_ConnectTo(self.handle, self.ip.as_ptr(), 0, 1);
            err_code
        };
        if err_code == 0 {
            let mut req: c_int = 0;
            let mut neg: c_int = 0;
            unsafe { S7LIB.Cli_GetPduLength(self.handle, &mut req, &mut neg); }
            self.req_len = req as usize;
            self.neg_len = neg as usize;
            self.is_connected.store(true, Ordering::Release);
            if log::max_level() == log::LevelFilter::Trace {
                log::debug!("{}.connect | Successfully connected", self.id);
            }
            Ok(())
        } else {
            self.is_connected.store(false, Ordering::Release);
            let code = S7Error::from(err_code);
            let err = S7Error::text(err_code);
            if log::max_level() == log::LevelFilter::Trace {
                log::warn!("{}.connect | Connection error: [{:?}] {:?}", self.id, code, err);
            }
            Err(Error::new(&self.id, "read").pass(err))
        }
    }
    ///
    /// Returns the actual connection status immediately without I/O operations
    pub fn is_connected(&self) -> bool {
        self.is_connected.load(Ordering::Acquire)
        // let mut is_connected: c_int = 0;
        // let code = unsafe {
        //     S7LIB.Cli_GetConnected(self.handle, &mut is_connected)
        // };
        // match code {
        //     0 => is_connected != 0,
        //     _ => {
        //         if log::max_level() == log::LevelFilter::Debug {
        //             log::warn!("{}.is_connected | Error: {:?}", self.id, S7Error::text(code));
        //         }
        //         false
        //     }
        // }
    }
    ///
    /// This is the main function to read data from a PLC.
    /// With it you can read DB, Inputs, Outputs, Merkers, Timers and Counters
    pub fn read(&self, db_num: u32, start: u32, size: u32) -> Result<Vec<u8>, Error> {
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
        if size as usize > self.neg_len {
            return Err(Error::new(&self.id, "read").err("Requested size is larger than PDU length"));
        }
        match code {
            0 => Ok(buf),
            _ => {
                let is_connected = S7Error::is_connected(code);
                self.is_connected.store(is_connected, Ordering::Release);
                Err(Error::new(&self.id, "read").pass(S7Error::text(code)))
            }
        }
    }
    ///
    /// This is the main function to write data into a PLC.
    /// 
    /// It’s the complementary function of
    /// Cli_ReadArea(), the parameters and their meanings are the same.
    /// The only difference is that the data is transferred from the buffer pointed by pUsrData
    /// into PLC.
    pub fn write(&self, db_num: u32, start: u32, size: u32, buf: &mut [u8]) -> Result<(), Error> {
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
            _ => {
                let is_connected = S7Error::is_connected(code);
                self.is_connected.store(is_connected, Ordering::Release);
                Err(Error::new(&self.id, "read").pass(S7Error::text(code)))
            }
        }
    }
    ///
    /// Disconnects “gracefully” the Client from the PLC.
    pub fn close(&mut self) -> Result<(), Error> {
        let code = unsafe {
            S7LIB.Cli_Disconnect(self.handle)
        };
        self.is_connected.store(false, Ordering::Release);
        match code {
            0 => Ok(()),
            _ => Err(Error::new(&self.id, "read").pass(S7Error::text(code))),
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
