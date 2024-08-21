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
    value: T,
    is_changed: bool,
}
//
// 
impl<T> FilterEmpty<T> {
    pub fn new(initial: T) -> Self {
        Self { value: initial, is_changed: true }
    }
}
//
// 
impl<T: Copy + std::fmt::Debug + std::cmp::PartialEq> Filter for FilterEmpty<T> {
    type Item = T;
    //
    //
    fn value(&self) -> Self::Item {
        self.value
    }
    //
    //
    fn add(&mut self, value: Self::Item) {
        if value != self.value {
            self.is_changed = true;
            self.value = value;
        } else {
            self.is_changed = false;
        }
    }
    //
    //
    fn is_changed(&self) -> bool {
        self.is_changed
    }
}