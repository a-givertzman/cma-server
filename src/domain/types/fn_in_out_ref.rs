use std::{rc::Rc, cell::RefCell};
use crate::services::task::FnInOut;
///
/// FnInOut mutable reference
pub type FnInOutRef = Rc<RefCell<Box<dyn FnInOut>>>;
