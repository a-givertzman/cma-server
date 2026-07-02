use std::{rc::Rc, cell::RefCell};
use crate::services::task::{FnInOut, FnOut};
///
/// FnInOut mutable reference
pub type FnInOutRef = Rc<RefCell<dyn FnInOut>>;
pub type FnOutRef = Rc<RefCell<dyn FnOut>>;
