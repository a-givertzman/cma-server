//!
//! Communication with some server sotware over the TCP/IP
//!  
//! - Establishing the connection on the specified server address
//! - Holding single input queue
//! - Received messages pops from the queue into the end of local buffer
//! - Buffered bessages converts into the regular `Events` and sends to the specified receiver, `MultiQueue` for example
//! - Receiving incoming from `Services` regular `Events and storing to the local buffer
//! - Sending buffered regular `Events` converted to the messages over the isteblished connection
//! - Sent messages immediately removed from the buffer
//!
pub mod tcp_client;