//! ### Sending requiests to the API
//! 
//! - Holding single input queue
//! - Received string events (containig SQL for example) pops from the queue into the end of local buffer
//! - Sending requests (wrapped into ApiQuery) from the beginning of the buffer
//! - Sent requests immediately removed from the buffer
//! - Replies from requests can be returned with same event name, if **`send-to`** is specified
//!
mod api_client_conf;
mod api_client;

pub use api_client_conf::*;
pub use api_client::*;