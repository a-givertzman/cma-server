///
/// Holds single value
/// - call add(value) to apply new value
/// - pop current value by calling value()
/// - is_changed() - check if value was changed after las add()
pub trait Filter: std::fmt::Debug {
    type Item;
    ///
    /// Returns current state
    fn value(&mut self) -> Option<Self::Item>;
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
    value: Option<T>,
}
//
// 
impl<T> FilterEmpty<T> {
    pub fn new(initial: Option<T>) -> Self {
        Self { value: initial }
    }
}
//
// 
impl<T: Copy + std::fmt::Debug + std::cmp::PartialEq> Filter for FilterEmpty<T> {
    type Item = T;
    //
    //
    fn value(&mut self) -> Option<Self::Item> {
        self.value.take()
    }
    //
    //
    fn add(&mut self, value: T) {
        if let Some(self_value) = self.value {
            if value != self_value {
                self.value = Some(value);
            }
        }
    }
    //
    //
    fn is_changed(&self) -> bool {
        self.value.is_some()
    }
}