//!
//! # Implements communication with Vibro-analytics microcontroller (Sub MC) over udp simple protocol.
//! 
//! - Read Data
//!     - Cyclically reads data bytes from the device (STM32 Micro-controller)
//!     - Data bytes contains an array of the samples receaved from the ADC, corresponds to the sample frequence
//!     - Converts data bytes into the amplitudes of the source input signal, where each value coresponds to the exact time
//!     - Sends each amplitude value as `Event` to the specified destination service.
//! - Send Commands
//!     - Writes received command `Event` to the device.
//! 
//! ## 1. General
//! 
//! - **Functional bytes**
//! 
//!     `SYN` = 0x22 - Start
//!     `EOT` =  0x04
//! 
//!     `STX` = 0x02 - Data message
//!     `CMD` = 0x05 - Command message
//!     `ERR` = 0x07 - Error message
//! 
//! - **Members**
//! 
//!     `Client` - Backend application
//!     `Server` - Device (micro-controller)
//! 
//! - **Message structure**
//! 
//!     |Field name:   | FUN | ADDR | TYPE | COUNT | DATA        |
//!     |---           | --- | ---- | ---- | ----- | ----        |
//!     |Data type:    | u8  | u8   | u8   | u32   | [T; COUNT]  | 
//!     |Example value:| 22  | 0    | 16   | 512   | [u16; 512]  |
//!     
//!     - `FUN` Functional byte, 
//!         - `0x22` - Initialization message
//!         - `0x02` - Data message
//!         - `0x05` - Command message
//!         - `0x07` - Error message
//!     - `ADDR` = 0...255 - Index of the input channel (0 - first input channel)
//!     - `TYPE` - type of values in the array in `DATA` field
//!         - 8 - u8, 1 byte unsigned integer value
//!         - 9 - i8, 1 byte signed integer value
//!         - 16 - u16, 2 byte unsigned integer value
//!         - 17 - i16, 2 byte signed integer value
//!         - 32 - u32, 4 byte unsigned integer value
//!         - 33 - i32, 4 byte signed integer value
//!         - 132 - f32, 4 bytes float value
//!     - `COUNT` - length of the `DATA` field in bytes
//!     - `DATA` - array of values of type specified in the `TYPE` field
//! 
//! - **Error codes**
//!     `0x01` - System error
//!     `0x02` - ADC Error
//!     `0x03` - DMA Error
//!     `0x04` - Network error
//!     `...` - To be extended if necessary
//! 
//! ## 2 Initialization
//! 
//! `Client` sends to `Device`
//!     `[0x22, 0x04]`
//! 
//! - If `Server` hasn't any active connection
//!     - `Client` IP registered
//!     - Data transmission to the registered `Client` IP
//!     - All additional connection ignored until current `Client` is active
//! - If `Server` already has active connection with same IP, connection restored
//! - If `Server` already has active connection, with different IP, initialization ignored
//! 
//! ## 3 Flow
//! 
//! - `Server` begins transitions of the data messages immediately after initialization, continues until disconnected
//!     `[0x02, 0x01, 0x16, 0x02, 0x00, 0xXX, ..., 0xXX]` - data message
//!     - Byte 0:  0x02 - Data message,
//!     - Byte 1: 0x01 - Index of channel (Second channel)
//!     - Byte 2: 0x16 - Type of values in the array (2 byte unsigned integer value)
//!     - Byte 3, 4: 0x02, 0x00 - Count of values of type u16 in the array (512)
//!     - Data bytes of length 1024 bytes (512 values u16)
//! - Any time the `Server` has internal error, it's code immediately sent to the `Client` 
//!     `[0x07, 0x01]`
//!     - Byte 0: `0x07` - Error message
//!     - Byte 1: `0x01` - Error code
//! - `Server` received the Command message, it's handled, applied, then `Server` returns to the data transmission
//!     TODO: Command messages to be defined later
//! 
//! ## 4. Configuration example for single Sub MC:
//! 
//! **Default port number 15180**
//! 
//! ```yaml
//! service UdpClient UdpClientSencor01:
//!     cycle: 10ms
//!     ...
//! ```
//! 
mod input_type;
mod udp_client_conf;
mod udp_client_connect;
mod udp_client;

pub(crate) use input_type::*;
pub use udp_client_conf::*;
pub(crate) use udp_client_connect::*;
pub use udp_client::*;
