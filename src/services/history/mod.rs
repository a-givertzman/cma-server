//!
//! Service specifically designed for sending incoming `Events` to the specified API Server
//! - Receives incoming events, stores them into the local buffer
//! - Using configured formated string, writes value/timestamp/status
//! - Sends formated string with the posted arguments (wrapped into the ApiQuery) to the
//! specified API Server to store it into the special History table
//!
pub mod producer_service;
pub mod producer_service_conf;