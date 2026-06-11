//! Core and Domain tools
//! - auth - Authentication entities, i.e. encripted password storing and validation, ssh storing and validation etc...
//! - cli - Command-line interface (CLI) instruments
//! - constants - Global constants, basically teporary used, later moved to the specific classes if possible
//! - failure - Future entity used specifically for the multi thread code
//! - filter - Filtering interface and it's common implementations
//! - format - Extracts yhe formated string. For example defined in the configuration yaml file and later used in the code with exact values pasted instead of parametes 
//! - net - Ethernet TCP/UDP or serial instruments
//! - retain_buffer - Buffer used for retaining mechanisms
//! - testing - Tools helpfull for testing
//! - types - Local types
//! 
pub mod auth;
pub mod cli;
pub mod constants;
mod dsp;
pub use dsp::*;
pub mod failure;
pub mod filter;
pub mod format;
pub mod net;
mod point;
pub(crate) use point::*;
pub mod retain_buffer;
pub mod testing;
mod types;
pub(crate) use types::*;
mod error;
pub(crate) use error::*;
