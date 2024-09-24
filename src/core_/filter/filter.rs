use circular_buffer::CircularBuffer;

///
/// Holds single value
/// - call add(value) to apply new value
/// - pop current value by calling value()
/// - is_changed() - check if value was changed after las add()
pub trait Filter: std::fmt::Debug {
    type Item;
    ///
    /// Returns current state
    fn pop(&mut self) -> Option<Self::Item>;
    /// - Updates state with value if value != inner
    fn add(&mut self, value: Self::Item);
    ///
    /// Returns true if last [add] was successful, internal value was changed
    fn is_changed(&self) -> bool;
}
///
/// Pass input value as is
#[derive(Debug, Clone)]
pub struct FilterEmpty<const N: usize, T> {
    value: CircularBuffer<N, T>,
}
//
// 
impl<T, const N: usize> FilterEmpty<N, T> {
    pub fn new(initial: Option<T>) -> Self {
        let mut value = CircularBuffer::<N, T>::new();
        if let Some(initial) = initial {
            value.push_back(initial);
        };
        Self { value }
    }
}
//
// 
impl<T: Copy + std::fmt::Debug + std::cmp::PartialEq, const N: usize> Filter for FilterEmpty<N, T> {
    type Item = T;
    //
    //
    fn pop(&mut self) -> Option<Self::Item> {
        self.value.pop_front()
    }
    //
    //
    fn add(&mut self, value: Self::Item) {
        match self.value.iter().last() {
            Some(last) => {
                if value != *last {
                    self.value.push_back(value);
                }
            }
            None => {
                self.value.push_back(value);
            }
        }
    }
    //
    //
    fn is_changed(&self) -> bool {
        !self.value.is_empty()
    }
}