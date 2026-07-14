//! Storing all received events in the memory
//! 
//! Events received from the Service's will sent to the `Cache` Service and transmitted to the Cliens simultaniusly.
//! 
//! `Cache` Service stores only the latest received Event for each name
//! 
//! This is useful when the connection with sub devices lost or not established, but the Client's
//! are olready connected, then cached Event's will be sent them with the status `Obsolete`
//! 
//! So the clients can know the last state of the defices if neccesary
//!
mod cache_service_conf;
mod cache_service;
mod delay_store;

pub use cache_service_conf::*;
pub use cache_service::*;
