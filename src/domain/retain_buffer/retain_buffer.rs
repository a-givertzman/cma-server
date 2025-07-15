use std::collections::VecDeque;
///
/// Holds array of T
/// - push(value: T) - appends an element to the end of a buffer
///     - if capacity exeeded ferst element will be deleted
/// - pop(value: T) - returns and removes value<T> from the end of array
/// - remove(index) - returns and removes value<T> from the [index] position
/// - len() - Returns the number of elements in the buffer
#[derive(Debug)]
pub struct RetainBuffer<T> {
    id: String,
    vec: VecDeque<T>,
    capacity: Option<usize>,
}
//
// 
impl<T> RetainBuffer<T> {
    ///
    /// Creates new instance of the ReatinBuffer
    pub fn new(parent: impl Into<String>, name: impl Into<String>, capacity: Option<usize>) -> Self {
        Self { 
            id: format!("{}/RetainBuffer({})", parent.into(), name.into()),
            vec: VecDeque::new(),
            capacity,
        }
    }
    ///
    /// Appends an element to the back of a buffer,
    /// - if capacity exeeded ferst element will be lost.
    pub fn push(&mut self, value: T) {
        if let Some(capacity) = self.capacity {
            if self.vec.len() >= capacity {
                self.vec.pop_front();
            }
        }
        self.vec.push_back(value);
    }
    ///
    /// Returns first &value<T> in the buffer or None if empty
    pub fn first(&mut self) -> Option<&T> {
        self.vec.front()
    }
    ///
    /// Returns and removes first value<T> in the buffer
    pub fn pop_first(&mut self) -> Option<T> {
        self.vec.pop_front()
    }
    // ///
    // /// Returns and removes value<T> from the [index] position
    // pub fn remove(&mut self, index: usize) -> Option<T> {
    //     self.vec.remove(index)
    // }
    ///
    /// Returns the number of elements in the buffer
    pub fn len(&self) -> usize {
        self.vec.len()
    }
    ///
    /// Immediately stores the content of the buffer
    pub fn store(&self) {
        // TODO self.vec to be stored into the json located on the path coming from self.id
        panic!("{}.store | Method is not implemented yet", self.id);
    }
}