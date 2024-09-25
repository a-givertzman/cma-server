use circular_buffer::CircularBuffer;

///
/// Holds single value
/// - call add(value) to apply new value
/// - pop current value by calling value()
/// - is_changed() - check if value was changed after las add()
pub trait Filter: std::fmt::Debug {
    type Item;
    ///
    /// Removes and returns value from front
    fn pop(&mut self) -> Option<Self::Item>;
    ///
    /// - Updates state with value if value != inner
    fn add(&mut self, value: Self::Item);
    ///
    /// Returns last added value if exists
    fn last(&self) -> Option<Self::Item>;
    ///
    /// Returns true if last [add] was successful, internal value was changed
    fn is_changed(&self) -> bool;
}
///
/// Pass input value as is
#[derive(Debug, Clone)]
pub struct FilterEmpty<const N: usize, T> {
    buffer: CircularBuffer<N, T>,
    last: Option<T>,
}
//
// 
impl<T: Copy, const N: usize> FilterEmpty<N, T> {
    pub fn new(initial: Option<T>) -> Self {
        let mut buffer = CircularBuffer::<N, T>::new();
        let last = initial.map(|initial| {
            buffer.push_back(initial);
            initial
        });
        Self { buffer, last }
    }
}
//
// 
impl<T: Copy + std::fmt::Debug + std::cmp::PartialEq, const N: usize> Filter for FilterEmpty<N, T> {
    type Item = T;
    //
    //
    fn pop(&mut self) -> Option<Self::Item> {
        self.buffer.pop_front()
    }
    //
    //
    fn add(&mut self, value: Self::Item) {
        match self.buffer.iter().last() {
            Some(last) => {
                if value != *last {
                    self.buffer.push_back(value);
                    self.last = Some(value);
                }
            }
            None => {
                self.buffer.push_back(value);
                self.last = Some(value);
            }
        }
    }
    //
    //
    fn last(&self) -> Option<Self::Item> {
        self.last
    }
    //
    //
    fn is_changed(&self) -> bool {
        !self.buffer.is_empty()
    }
}