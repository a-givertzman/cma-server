use circular_buffer::CircularBuffer;
use super::filter::Filter;
///
/// 
#[derive(Debug, Clone)]
pub struct FilterThreshold<const N: usize, T> {
    buffer: CircularBuffer<N, T>,
    last: Option<T>,
    threshold: f64,
    factor: f64,
    acc: f64,
}
//
// 
impl<T: Copy, const N: usize> FilterThreshold<N, T> {
    const N: usize = N;
    ///
    /// Creates new FilterThreshold<const N: usize, T>
    /// - `N` - size of the Filter bufer,
    /// - `T` - Type of the Filter Item
    pub fn new(initial: Option<T>, threshold: f64, factor: f64) -> Self {
        let mut buffer = CircularBuffer::<N, T>::new();
        let last = initial.map(|initial| {
            buffer.push_back(initial);
            initial
        });
        Self {
            buffer,
            last,
            threshold, 
            factor,
            acc: 0.0,
        }
    }
}
//
//
impl<const N: usize> Filter for FilterThreshold<N, i16> {
    type Item = i16;
    //
    //
    fn pop(&mut self) -> Option<Self::Item> {
        self.buffer.pop_front()
    }
    //
    //
    fn add(&mut self, value: Self::Item) {
        match self.last {
            Some(last) => {
                let delta = (last as f64) - (value as f64);
                let delta = if self.factor > 0.0 {
                    self.acc += delta * self.factor;
                    self.acc.abs()
                } else {
                    delta.abs()
                };
                if delta > self.threshold {
                    self.buffer.push_back(value);
                    self.last = Some(value);
                    self.acc = 0.0;
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
//
//
impl<const N: usize> Filter for FilterThreshold<N, i32> {
    type Item = i32;
    //
    //
    fn pop(&mut self) -> Option<Self::Item> {
        self.buffer.pop_front()
    }
    //
    //
    fn add(&mut self, value: Self::Item) {
        match self.last {
            Some(last) => {
                let delta = (last as f64) - (value as f64);
                let delta = if self.factor > 0.0 {
                    self.acc += delta * self.factor;
                    self.acc.abs()
                } else {
                    delta.abs()
                };
                if delta > self.threshold {
                    self.buffer.push_back(value);
                    self.last = Some(value);
                    self.acc = 0.0;
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
//
//
impl<const N: usize> Filter for FilterThreshold<N, i64> {
    type Item = i64;
    //
    //
    fn pop(&mut self) -> Option<Self::Item> {
        self.buffer.pop_front()
    }
    //
    //
    fn add(&mut self, value: Self::Item) {
        match self.last {
            Some(last) => {
                let delta = (last as f64) - (value as f64);
                let delta = if self.factor > 0.0 {
                    self.acc += delta * self.factor;
                    self.acc.abs()
                } else {
                    delta.abs()
                };
                if delta > self.threshold {
                    self.buffer.push_back(value);
                    self.last = Some(value);
                    self.acc = 0.0;
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
//
//
impl<const N: usize> Filter for FilterThreshold<N, f32> {
    type Item = f32;
    //
    //
    fn pop(&mut self) -> Option<Self::Item> {
        self.buffer.pop_front()
    }
    //
    //
    fn add(&mut self, value: Self::Item) {
        match self.last {
            Some(last) => {
                let delta = last - value;
                let delta = if self.factor > 0.0 {
                    self.acc += (delta as f64) * (self.factor);
                    self.acc.abs()
                } else {
                    delta.abs() as f64
                };
                if delta > self.threshold {
                    self.buffer.push_back(value);
                    self.last = Some(value);
                    self.acc = 0.0;
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
//
//
impl<const N: usize> Filter for FilterThreshold<N, f64> {
    type Item = f64;
    //
    //
    fn pop(&mut self) -> Option<Self::Item> {
        self.buffer.pop_front()
    }
    //
    //
    fn add(&mut self, value: Self::Item) {
        match self.last {
            Some(last) => {
                let delta = last - value;
                let delta = if self.factor > 0.0 {
                    self.acc += delta * self.factor;
                    self.acc.abs()
                } else {
                    delta.abs()
                };
                if delta > self.threshold {
                    self.buffer.push_back(value);
                    self.last = Some(value);
                    self.acc = 0.0;
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