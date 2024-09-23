use super::filter::Filter;
///
/// 
#[derive(Debug, Clone)]
pub struct FilterThreshold<T> {
    value: Option<T>,
    // is_changed: bool,
    threshold: f64,
    factor: f64,
    acc: f64,
}
//
// 
impl<T> FilterThreshold<T> {
    pub fn new(initial: Option<T>, threshold: f64, factor: f64) -> Self {
        Self {
            value: initial,
            // is_changed: true,
            threshold, 
            factor,
            acc: 0.0,
        }
    }
}
//
//
impl Filter for FilterThreshold<i16> {
    type Item = i16;
    //
    //
    fn value(&mut self) -> Option<Self::Item> {
        self.value.take()
    }
    //
    //
    fn add(&mut self, value: Self::Item) {
        match self.value {
            Some(self_value) => {
                let delta = (self_value as f64) - (value as f64);
                let delta = if self.factor > 0.0 {
                    self.acc += delta * self.factor;
                    self.acc.abs()
                } else {
                    delta.abs()
                };
                if delta > self.threshold {
                    self.value = Some(value);
                    self.acc = 0.0;
                }
            }
            None => self.value = Some(value),
        }
    }
    //
    //
    fn is_changed(&self) -> bool {
        self.value.is_some()
    }
}
//
//
impl Filter for FilterThreshold<i32> {
    type Item = i32;
    //
    //
    fn value(&mut self) -> Option<Self::Item> {
        self.value.take()
    }
    //
    //
    fn add(&mut self, value: Self::Item) {
        match self.value {
            Some(self_value) => {
                let delta = (self_value as f64) - (value as f64);
                let delta = if self.factor > 0.0 {
                    self.acc += delta * self.factor;
                    self.acc.abs()
                } else {
                    delta.abs()
                };
                if delta > self.threshold {
                    self.value = Some(value);
                    self.acc = 0.0;
                }
            }
            None => self.value = Some(value),
        }
    }
    //
    //
    fn is_changed(&self) -> bool {
        self.value.is_some()
    }
}
//
//
impl Filter for FilterThreshold<i64> {
    type Item = i64;
    //
    //
    fn value(&mut self) -> Option<Self::Item> {
        self.value.take()
    }
    //
    //
    fn add(&mut self, value: Self::Item) {
        match self.value {
            Some(self_value) => {
                let delta = (self_value as f64) - (value as f64);
                let delta = if self.factor > 0.0 {
                    self.acc += delta * self.factor;
                    self.acc.abs()
                } else {
                    delta.abs()
                };
                if delta > self.threshold {
                    self.value = Some(value);
                    self.acc = 0.0;
                }
            }
            None => self.value = Some(value),
        }
    }
    //
    //
    fn is_changed(&self) -> bool {
        self.value.is_some()
    }
}
//
//
impl Filter for FilterThreshold<f32> {
    type Item = f32;
    //
    //
    fn value(&mut self) -> Option<Self::Item> {
        self.value.take()
    }
    //
    //
    fn add(&mut self, value: Self::Item) {
        match self.value {
            Some(self_value) => {
                let delta = self_value - value;
                let delta = if self.factor > 0.0 {
                    self.acc += (delta as f64) * (self.factor);
                    self.acc.abs()
                } else {
                    delta.abs() as f64
                };
                if delta > self.threshold {
                    self.value = Some(value);
                    self.acc = 0.0;
                }
            }
            None => self.value = Some(value),
        }
    }
    //
    //
    fn is_changed(&self) -> bool {
        self.value.is_some()
    }
}
//
//
impl Filter for FilterThreshold<f64> {
    type Item = f64;
    //
    //
    fn value(&mut self) -> Option<Self::Item> {
        self.value.take()
    }
    //
    //
    fn add(&mut self, value: Self::Item) {
        match self.value {
            Some(self_value) => {
                let delta = self_value - value;
                let delta = if self.factor > 0.0 {
                    self.acc += delta * self.factor;
                    self.acc.abs()
                } else {
                    delta.abs()
                };
                if delta > self.threshold {
                    self.value = Some(value);
                    self.acc = 0.0;
                }
            }
            None => self.value = Some(value),
        }
    }
    //
    //
    fn is_changed(&self) -> bool {
        self.value.is_some()
    }
}