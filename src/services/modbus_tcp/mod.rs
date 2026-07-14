//! # Modbus-TCP Client
//! 
//! References:
//!    - [Wikipedia | Modbus](https://en.wikipedia.org/wiki/Modbus)
//!    - [ACROMAG | INTRODUCTION TO MODBUS TCP/IP](https://www.prosoft-technology.com/kb/assets/intro_modbustcp.pdf)
//! 
//! ## 1. TCP/IP Port: 502
//! 
//! Standard TCP frames are sent via TCP to well-known system
//! port 502, which is specifically reserved for Modbus applications.
//! 
//! ## 2. Data messages (PDU)
//! 
//!  Modbus Application Protocol (MBAP) Header | Protocol Data Unit (PDU)
//! :---: | :---:
//! 7 Bytes | 
//! 
//!  Transaction ID | Protocol ID | Length Field |  Unit ID | Function Code | Data
//!  ---            | ---         | ---          | ---      | ---           | ---
//!   2 Bytes       | 2 Bytes     | 2 Bytes      | 1 Bytes  | 1 Byte        | Varies
//! 
//! - **Transaction ID (2 Bytes):** This identification field is used for transaction pairing when multiple messages are sent along
//! the same TCP connection by a client without waiting for a prior response.
//! - **Protocol ID (2 bytes):** This field is always 0 for Modbus services and other values are reserved for future extensions.
//! - **Length (2 bytes):** This field is a byte count of the remaining fields and includes the `Unit ID` byte, `Function Code` byte, and the `Data` fields.
//! - **Unit ID (1 byte):** This field is used to identify a remote server located on a non TCP/IP network (for serial bridging).
//! In a typical Modbus TCP/IP server application, the `Unit ID` is set to 00 or FF, ignored by the server, and simply echoed back in the response.
//! 
//! ## Functions
//! 
//! - **Coils (Outputs)**
//! - **Discrete Inputs**
//! - **Input Registers (Input Data)**
//! - **Holding Registers (Output Data)**
//! 
//! Function                                            | Code   | Description
//! ---                                                 | ---    | ---
//! Read Coil Status                                    | 01     | Read the ON/OFF status of discrete outputs or coils (0x reference addresses) in the slave/server.
//! Force Single Coil                                   | 05     | Forces a single coil/output (0x reference address) ON or OFF.
//! Read Input Registers                                | 04     | Read the binary contents of input registers (3x reference addresses) in the slave device.
//! Preset Single Register                              | 06     | Preset a single holding register (4x reference address) to a specific value.
//! Read Holding Registers                              | 03     | Read the binary contents of holding registers (4x reference addresses) in the slave device.
//! 
//! ## Register Data Type
//! 
//! Reference Address | Description
//! ---               | ---
//! 0xxxx             | Read/Write Discrete Outputs or Coils. A 0x reference address is used to drive output data to a digital output channel.
//! 1xxxx             | Read Discrete Inputs. The ON/OFF status of a 1x reference address is controlled by the corresponding digital input channel.
//! 3xxxx             | Read Input Registers. A 3x reference register contains a 16-bit number received from an external source—e.g. an analog signal.
//! 4xxxx             | Read/Write Output or Holding Registers. A 4x register is used to store 16-bits of numerical data (binary or decimal), or to send the data from the CPU to an output channel.
//! 
//! IMPORTANT: The reference addresses noted in the memory map are not explicit hard-coded memory addresses.
//! Internally, all Modbus devices use a zero-based memory offset computed from the reference address.
//! However, the system interface of Modbus systems (software) will vary in this regard and may require
//! you to enter the actual reference address, drop the leading number, or enter an absolute memory offset from 1,
//! or a memory address offset from 0. This is system dependent and a common source of programming errors.
//! Be wary of this when writing higher-level application programs to access these registers.
//! 
//! Note that not all Modbus functions operate on register map registers. All data addresses in Modbus messages
//! are referenced to 0, with the first occurrence of a data item addressed as item number zero.
//! Further, a function code field already specifies which register group it operates on
//! (i.e. 0x, 1x, 3x, or 4x reference addresses). For example, holding register 40001 is addressed
//! as register 0000 in the data address field of the message. The function code that operates on
//! this register specifies a “holding register” operation and the “4xxxx” reference group is implied.
//! Thus, holding register 40108 is actually addressed as register 006BH (107 decimal).
//! 
//! The function code field of the message (PDU) will contain one byte that tells the slave what kind
//! of action to take. Valid function codes are from 1-255, but not all codes will apply to a module
//! and some codes are reserved for future use. Additionally, the Modbus specification allocates
//! function codes 65-72 and 100-110 for user-defined services.
//! 
mod modbus;
mod modbus_tcp_conf;
mod modbus_tcp_read;
mod modbus_tcp_write;
mod modbus_tcp;
mod modbus_unit_conf;
mod modbus_unit;

pub use modbus::*;
pub use modbus_tcp_conf::*;
pub use modbus_tcp_read::*;
pub use modbus_tcp_write::*;
pub use modbus_tcp::*;
pub use modbus_unit_conf::*;
pub use modbus_unit::*;