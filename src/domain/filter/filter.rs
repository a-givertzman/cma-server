use sal_sync::services::entity::Status;

///
/// Holds single value
/// - call add(value) to apply new value
/// - pop current value by calling value()
/// - is_changed() - check if value was changed after las add()
pub trait Filter: std::fmt::Debug {
    type Item;
    // ///
    // /// Returns filtered value if exists
    // fn pop(&mut self) -> Option<Self::Item>;
    ///
    /// - Adds new value to the filter
    /// - Returns filtered value if exists
    fn add(&mut self, value: Self::Item) -> Option<Self::Item>;
    ///
    /// Returns filtered value if exists
    fn last(&self) -> Option<Self::Item>;
    // ///
    // /// Returns true if last [add] was successful, internal value was changed
    // fn is_changed(&self) -> bool;
}
///
/// Pass input value as is if changed
#[derive(Debug, Clone)]
pub struct FilterEmpty<T> {
    last: Option<T>, // Последнее отфильтрованное (актуальное состояние)
}
//
// 
impl<T: Copy> FilterEmpty<T> {
    pub fn new(initial: Option<T>) -> Self {
        Self { last: initial }
    }
}
//
// 
impl Filter for FilterEmpty<f32> {
    type Item = f32;
    //
    fn add(&mut self, value: f32) -> Option<f32> {
        if self.last.map_or(true, |last| (value - last).abs() > f32::EPSILON) {
            self.last = Some(value);
            Some(value)
        } else {
            None
        }
    }
    //
    fn last(&self) -> Option<Self::Item> {
        self.last
    }
}
//
// 
impl Filter for FilterEmpty<f64> {
    type Item = f64;
    //
    fn add(&mut self, value: f64) -> Option<f64> {
        if self.last.map_or(true, |last| (value - last).abs() > f64::EPSILON) {
            self.last = Some(value);
            Some(value)
        } else {
            None
        }
    }
    //
    fn last(&self) -> Option<Self::Item> {
        self.last
    }
}
//
// 
/// Marker trait for types that compare via Eq
pub trait StepFilterEq {}
// Mark your integer types
impl StepFilterEq for i8 {}
impl StepFilterEq for i16 {}
impl StepFilterEq for i32 {}
impl StepFilterEq for i64 {}
impl StepFilterEq for u8 {}
impl StepFilterEq for u16 {}
impl StepFilterEq for u32 {}
impl StepFilterEq for u64 {}
impl StepFilterEq for usize {}
impl StepFilterEq for bool {}
impl StepFilterEq for Status {}

//
// 
impl<T: Copy + std::fmt::Debug + std::cmp::Eq + StepFilterEq> Filter for FilterEmpty<T> {
    type Item = T;
    //
    fn add(&mut self, value: Self::Item) -> Option<Self::Item> {
        match self.last {
            Some(last) => if value != last {
                self.last = Some(value);
                return Some(value)
            }
            None => {
                self.last = Some(value);
                return Some(value)
            }
        }
        None
    }
    //
    fn last(&self) -> Option<Self::Item> {
        self.last
    }
}
