/// Modbus documented error codes
/// - Please refer to code of the function Error::Text() for the explanation
/// - source: design/protocol-slmp/jy997d56001k.pdf
#[derive(Debug)]
#[repr(i64)]
pub enum ModbusError {
    IllegalFunction                              = 0x01,
    IllegalDataAddress                           = 0x02,
    IllegalDataValue                             = 0x03,
    DeviceFailure                                = 0x04,
    Acknowledge                                  = 0x05,
    DeviceBusy                                   = 0x06,
    NegativeAcknowledge                          = 0x07,
    MemoryParityError                            = 0x08,
    GatewayPathProblem                           = 0x0A,
    DeviceNotRespond                             = 0x0B,
    ExtendedExceptionResponse(String)            = 0xFF,
}
//
// 
impl ModbusError {
    pub fn text(code: i32) -> String {
        let as_str = &format!("{}", code);
        let err = match code {
            0x01 => "The function code received in the query is not allowed or invalid",
            0x02 => "The data address received in the query is not an allowable address for the slave or is invalid",
            0x03 => "A value contained in the query data field is not an allowable value for the slave or is invalid",
            0x04 => "The server failed during execution. An unrecoverable error occurred while the slave/server was attempting to perform the requested action",
            0x05 => "The slave/server has accepted the request and is processing it, but a long duration of time is required to do so. This response is returned to prevent a timeout error from occurring in the master",
            0x06 => "The slave is engaged in processing a long-duration program command. The master should retransmit the message later when the slave is free",
            0x07 => "The slave cannot perform the program function received in the query. This code is returned for an unsuccessful programming request using function code 13 or 14 (codes not supported by this model). The master should request diagnostic information from the slave",
            0x08 => "The slave attempted to read extended memory, but detected a parity error in memory. The master can retry the request, but service may be required at the slave device",
            0x0A => "Gateway path(s) not available",
            0x0B => "The target device failed to respond (the gateway generates this exception)",
            0xFF => "The exception response PDU contains extended exception information. A subsequent 2 byte length field indicates the size in bytes of this function-code specific exception information",
            _ => as_str,
        };
        err.to_owned()
    }    
}
//
// 
impl From<i32> for ModbusError {
    fn from(value: i32) -> Self {
        match value {
            0x01 => Self::IllegalFunction,
            0x02 => Self::IllegalDataAddress,
            0x03 => Self::IllegalDataValue,
            0x04 => Self::DeviceFailure,
            0x05 => Self::Acknowledge,
            0x06 => Self::DeviceBusy,
            0x07 => Self::NegativeAcknowledge,
            0x08 => Self::MemoryParityError,
            0x0A => Self::GatewayPathProblem,
            0x0B => Self::DeviceNotRespond,
            0xFF => Self::ExtendedExceptionResponse(String::new()),
            _ => {
                Self::Inner(format!("{} ({})", Self::text(value), value))
            }
        }
    }
}
