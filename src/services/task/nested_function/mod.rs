//!
//! Allows to build a declorative logic in the Task service using specific syntax in the yaml file
//! 
//! Fallowing example implements logging a result of the comparation '/App/Ied001/Load' >= 5.5:
//!   - if point '/App/Ied001/Load' has value > 5.5 'true' will be logged
//!   - if point '/App/Ied001/Load' has value < 5.5 'false' will be logged
//! ```yaml
//! fn Debug:
//!     input fn Ge:
//!         input1: point real '/App/Ied001/Load'
//!         input2: const real 5.5
//! ```
//! 
//! The embedded functions and keywords must be used in the lower case:
//! - var
//! - input
//! - const
//! 
//! Another functions must be used in CamelCase:
//! - Add
//! - Ge  
//! etc...
pub mod fn_;
pub mod fn_kind;
pub mod fn_result;
pub mod functions;
pub mod fn_input;
pub mod fn_count;
pub mod fn_timer;
pub mod fn_var;
pub mod fn_const;
pub mod fn_point_id;
pub mod fn_debug;
pub mod fn_to_int;
pub mod fn_average;
pub mod fn_acc;
pub mod fn_to_real;
pub mod fn_to_double;
pub mod fn_to_bool;
pub mod fn_max;
pub mod fn_piecewise_line_approx;
pub mod fn_is_changed_value;
pub mod fn_keep_valid;
pub mod fn_to_string;

pub mod nested_fn;

pub mod sql_metric;

pub mod edge_detection;
pub mod export;
pub mod import;
pub mod io;
pub mod filter;
pub mod comp;
pub mod ops;
pub mod plot;

pub mod reset_counter;

///
/// Recorder functions
pub mod fn_rec_op_cycle_metric; 

pub mod va;