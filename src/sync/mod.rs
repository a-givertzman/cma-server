//! 
//! Tools useful for multi thread code and entities which can be safely shared between threads.
//! 
mod atomic_usize_option;

pub use atomic_usize_option::*;