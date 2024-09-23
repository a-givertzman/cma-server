use std::cell::RefCell;
///
/// Holds single value
/// - call add(value) to apply new value
/// - get current value by calling value()
/// - is_changed() - check if value was changed after las add()
pub trait Filter: std::fmt::Debug {
    type Item;
    ///
    /// Returns current state
    fn value(&self) -> Self::Item;
    /// - Updates state with value if value != inner
    fn add(&mut self, value: Self::Item);
    ///
    /// Returns true if last [add] was successful, internal value was changed
    fn is_changed(&self) -> bool;
}
///
/// Pass input value as is
#[derive(Debug, Clone)]
pub struct FilterEmpty<T> {
    value: RefCell<Option<T>>,
    is_changed: bool,
}
//
// 
impl<T> FilterEmpty<T> {
    pub fn new(initial: T) -> Self {
        Self { value: RefCell::new(Some(initial)), is_changed: true }
    }
}
//
// 
impl<T: Copy + std::fmt::Debug + std::cmp::PartialEq> Filter for FilterEmpty<T> {
    type Item = T;
    //
    //
    fn value(&self) -> Self::Item {
        self.value.borrow_mut().take().unwrap()
    }
    //
    //
    fn add(&mut self, value: Self::Item) {
        if Some(value) != *self.value.borrow() {
            self.is_changed = true;
            *self.value.borrow_mut() = Some(value);
        // } else {
        }
    }
    //
    //
    fn is_changed(&self) -> bool {
        self.is_changed
    }
}