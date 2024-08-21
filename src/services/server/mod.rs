//!
//! ### Bounds TCP socket server
//! 
//! Listening socket for incoming connections  
//! Handles connections in the separate thread  
//!   - Verifing incoming connection
//!   - Authenticating client
//!   - Provide "Points" request - returning list of configured points
//!   - Provide "Subscribe" request - begins transfering points subscribed on

pub mod tcp_server;

pub mod jds_auth;

pub mod jds_cnnection;

pub mod connections;

pub mod jds_request;

pub mod jds_routes;
