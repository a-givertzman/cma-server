//!
//! # `VirtualDevice` `Service` is emulation of the real device behavior. 
//! 
//! It's signals can be charged from file-based test data or calculated in the `Task`-based calculations
//! 
//! Can be used instead of a real device connections such as `UdpClient` or `ProfinetClient` etc.
//! - Signals configuration can be directly copied from the real device
//! - Loading test sequences from the table files (ODS)
//! - Storing results nier by the corresponding input event
//! - Comparison of target and result values to highlight test failures
//!
mod cmd_kind;
mod header;
mod input_block;
mod result_block;
mod result_kind;
mod sql_result;
mod table;
mod virtual_device_conf;
mod virtual_device;

pub use cmd_kind::*;
pub use header::*;
pub use input_block::*;
pub use result_block::*;
pub use result_kind::*;
pub use sql_result::*;
pub(super) use table::*;
pub use virtual_device_conf::*;
pub use virtual_device::*;
