///
/// Wrapper for x, y values of any useful cases
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Offset<T> {
    pub x: T,
    pub y: T,
}
impl<T> Offset<T> {
    pub fn new(x: T, y: T) -> Self {
        Self {
            x,
            y,
        }
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
