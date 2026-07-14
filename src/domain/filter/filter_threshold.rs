use super::filter::Filter;
trait ToF64 {
    fn to_f64(self) -> f64;
}
impl ToF64 for i8 { fn to_f64(self) -> f64 { self as f64 } }
impl ToF64 for i16 { fn to_f64(self) -> f64 { self as f64 } }
impl ToF64 for i32 { fn to_f64(self) -> f64 { self as f64 } }
impl ToF64 for i64 { fn to_f64(self) -> f64 { self as f64 } }
impl ToF64 for u8 { fn to_f64(self) -> f64 { self as f64 } }
impl ToF64 for u16 { fn to_f64(self) -> f64 { self as f64 } }
impl ToF64 for u32 { fn to_f64(self) -> f64 { self as f64 } }
impl ToF64 for u64 { fn to_f64(self) -> f64 { self as f64 } }
impl ToF64 for usize { fn to_f64(self) -> f64 { self as f64 } }
impl ToF64 for isize { fn to_f64(self) -> f64 { self as f64 } }
impl ToF64 for f32 { fn to_f64(self) -> f64 { self as f64 } }
impl ToF64 for f64 { fn to_f64(self) -> f64 { self } }
///
/// Фильтром значимых изменений 
/// - с абсолютной зоной нечувствительности, если `factor` не указан (`factor == 0.0`)
/// - с накоплением ошибки, если `factor` указан (`factor > 0.0`)
#[derive(Debug, Clone)]
pub struct FilterThreshold<T> {
    initial: Option<T>,
    last: Option<T>,
    threshold: f64,
    factor: f64,
    acc: f64,
}
//
// 
impl<T: Copy> FilterThreshold<T> {
    ///
    /// Returns [FilterThreshold<T>] new instance
    /// - `T` - Type of the Filter Item
    /// - `initial` - To be stored as start value
    /// - `threshold` - Absolute threshold
    /// - `factor` - Integrated threshold, dipends on the cycle frequence
    pub fn new(initial: Option<T>, threshold: f64, factor: f64) -> Self {
        Self {
            initial: initial,
            last: initial,
            threshold, 
            factor,
            acc: 0.0,
        }
    }
    ///
    /// Returns [FilterThreshold<T>] instance with updated `threshold`, keeping the state
    /// - `threshold` - Absolute threshold
    pub fn with_threshold(&self, threshold: f64) -> Self {
        Self {
            initial: self.initial,
            last: self.last,
            threshold, 
            factor: self.factor,
            acc: self.acc,
        }
    }
    ///
    /// Returns [FilterThreshold<T>] instance with updated `factor`, keeping the state
    /// - `factor` - Integrated threshold, dipends on the cycle frequence
    pub fn with_factor(&self, factor: f64) -> Self {
        Self {
            initial: self.initial,
            last: self.last,
            threshold: self.threshold, 
            factor,
            acc: self.acc,
        }
    }
    ///
    /// Resets the state to the initial, keeping `threshold` and `factor`
    pub fn reset(&mut self) {
        self.last = self.initial;
        self.acc = 0.0;
    }
}
//
//
impl<T: Copy + ToF64 + std::fmt::Debug> Filter for FilterThreshold<T> {
    type Item = T;
    //
    //
    fn add(&mut self, value: Self::Item) -> Option<Self::Item> {
        let Some(last) = self.last else {
            self.last = Some(value);
            return Some(value);
        };
        let delta = last.to_f64() - value.to_f64();
        let diff = if self.factor > 0.0 {
            self.acc += delta * self.factor;
            self.acc.abs()
        } else {
            delta.abs()
        };
        if diff > self.threshold {
            self.last = Some(value);
            self.acc = 0.0;
            Some(value)
        } else {
            None
        }
    }
    //
    //
    fn last(&self) -> Option<Self::Item> {
        self.last
    }
}
// //
// //
// impl Filter for FilterThreshold<i32> {
//     type Item = i32;
//     //
//     //
//     fn add(&mut self, value: Self::Item) -> Option<Self::Item> {
//         match self.last {
//             Some(last) => {
//                 let delta = (last as f64) - (value as f64);
//                 let delta = if self.factor > 0.0 {
//                     self.acc += delta * self.factor;
//                     self.acc.abs()
//                 } else {
//                     delta.abs()
//                 };
//                 if delta > self.threshold {
//                     self.buffer.push_back(value);
//                     self.last = Some(value);
//                     self.acc = 0.0;
//                 }
//             }
//             None => {
//                 self.buffer.push_back(value);
//                 self.last = Some(value);
//             }
//         }
//     }
//     //
//     //
//     fn last(&self) -> Option<Self::Item> {
//         self.last
//     }
// }
// //
// //
// impl Filter for FilterThreshold<i64> {
//     type Item = i64;
//     //
//     //
//     fn add(&mut self, value: Self::Item) -> Option<Self::Item> {
//         match self.last {
//             Some(last) => {
//                 let delta = (last as f64) - (value as f64);
//                 let delta = if self.factor > 0.0 {
//                     self.acc += delta * self.factor;
//                     self.acc.abs()
//                 } else {
//                     delta.abs()
//                 };
//                 if delta > self.threshold {
//                     self.buffer.push_back(value);
//                     self.last = Some(value);
//                     self.acc = 0.0;
//                 }
//             }
//             None => {
//                 self.buffer.push_back(value);
//                 self.last = Some(value);
//             }
//         }
//     }
//     //
//     //
//     fn last(&self) -> Option<Self::Item> {
//         self.last
//     }
// }
// //
// //
// impl Filter for FilterThreshold<f32> {
//     type Item = f32;
//     //
//     //
//     fn add(&mut self, value: Self::Item) -> Option<Self::Item> {
//         match self.last {
//             Some(last) => {
//                 let delta = last - value;
//                 let delta = if self.factor > 0.0 {
//                     self.acc += (delta as f64) * (self.factor);
//                     self.acc.abs()
//                 } else {
//                     delta.abs() as f64
//                 };
//                 if delta > self.threshold {
//                     self.buffer.push_back(value);
//                     self.last = Some(value);
//                     self.acc = 0.0;
//                 }
//             }
//             None => {
//                 self.buffer.push_back(value);
//                 self.last = Some(value);
//             }
//         }
//     }
//     //
//     //
//     fn last(&self) -> Option<Self::Item> {
//         self.last
//     }
// }
// //
// //
// impl Filter for FilterThreshold<f64> {
//     type Item = f64;
//     //
//     //
//     fn add(&mut self, value: Self::Item) -> Option<Self::Item> {
//         match self.last {
//             Some(last) => {
//                 let delta = last - value;
//                 let delta = if self.factor > 0.0 {
//                     self.acc += delta * self.factor;
//                     self.acc.abs()
//                 } else {
//                     delta.abs()
//                 };
//                 if delta > self.threshold {
//                     self.buffer.push_back(value);
//                     self.last = Some(value);
//                     self.acc = 0.0;
//                 }
//             }
//             None => {
//                 self.buffer.push_back(value);
//                 self.last = Some(value);
//             }
//         }
//     }
//     //
//     //
//     fn last(&self) -> Option<Self::Item> {
//         self.last
//     }
// }