use std::ops::{Add, Mul, Sub};

///
/// Wrapper for x, y values of any useful cases
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Offset<T> {
    pub x: T,
    pub y: T,
}
impl<T> Offset<T> {
    ///
    /// Returns [Offset] new instance
    pub fn new(x: T, y: T) -> Self {
        Self {
            x,
            y,
        }
    }
}
impl<T: Copy + Add<Output = T> + Mul<Output = T> + Sub<Output = T> + Sqrt<T>> Offset<T> {
    ///
    /// Returns the distance between `self` and `other`
    pub fn distance(&self, other: Self) -> T {
        let dif_x = other.x - self.x;
        let dif_y = other.y - self.y;
        let v = dif_x * dif_x + dif_y * dif_y;
        v.sqrt_()
    }
}
impl<T: std::fmt::Display> std::fmt::Display for Offset<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Offset({}, {})", self.x, self.y)
    }
}
impl<T: std::fmt::Display> std::fmt::Debug for Offset<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Offset({}, {})", self.x, self.y)
    }
}
///
/// Used for square root calculations
pub trait Sqrt<T> {
    fn sqrt_(&self) -> T;
}
//
impl Sqrt<f64> for f64 {
    fn sqrt_(&self) -> f64 {
        f64::sqrt(*self)
    }
}
//
impl Sqrt<f32> for f32 {
    fn sqrt_(&self) -> f32 {
        f32::sqrt(*self)
    }
}
