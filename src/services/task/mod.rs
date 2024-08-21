//!
//! ### Service provides declarative computations configurable in the yaml file.
//! 
//! Consists of number of computation nodes. Each node consists of number of functions.  
//! The computation value - is point {type, value, tumestamp, status}  
//! Each computation cycle sequentally calls computation nodes of the task in the order defined in the configuration.  
//! So variables used in the task must be defined earlier then used.  
//! 
//! ### Basic entities and principles of the Tasck service computations
//! 
//! - **The computations can be executed**
//!   - periodically with configured cycle time (min 10ms for now)
//!   - event-trigger, computation node will be performed if at least one of it's input received new point
//! 
//! - **Definitions**
//!   - let VarName - allows to define a variable
//!   - const - allows to define a constant, typed
//!   - input - alows to define an input, typed
//!   - fn FunctionName - allows to use a function by it's name
//! 
//! - **Inputs**
//!   - VariableName - read point from defined earlier variable
//!   - const - read point from constant
//!   - input - read point from input
//! 
//! - **Variable**
//! 
//! The result of the computation node can be stored in the variable,  
//! wich can be used late in the any computation node.  
//! 
//! ```yaml
//! # Syntax
//! let <VariableName>:
//!     input...
//! ```
//! 
//! For example variable 'Var' defined. And used late in the function 'Add':
//! 
//! ```yaml
//! service Task ExampleTask:
//!     cycle: 1s
//!     let Var:
//!         input: ...
//!     fn Add:
//!         input1: const int 3
//!         input2: Var
//! # returns 3 + Var on each step
//! ```
//! 
//! - **Constant**
//! 
//! Always returns configured constant value, can be used in the any input of the any function.
//! 
//! ```yaml
//! # Syntax
//! const <type> <value>
//! ```
//! 
//! For example constant used on the inputs of the 'Add' function
//! 
//! ```yaml
//! service Task ExampleTask:
//!     cycle: 1s
//!     fn Add:
//!         input1: const int 3
//!         input2: const int 7
//! # returns 10 on each step
//! ```
//! 
//! - **Input**
//! 
//! Returns latest received point
//! 
//! ```yaml
//! # Syntax
//! input <type> <'/path/PointName'>
//! ```
//! 
pub mod task;

pub mod task_nodes;

pub mod task_node_vars;

pub mod service_cycle;

pub mod task_eval_node;

pub mod task_test_receiver;

pub mod task_test_producer;

pub mod nested_function;
