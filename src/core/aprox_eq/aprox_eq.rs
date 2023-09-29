pub trait AproxEq<T> {
    fn aproxEq(&self, other: T, decimals: usize) -> bool;
}


impl AproxEq<f32> for f32 {
    fn aproxEq(&self, other: f32, decimals: usize) -> bool {
        let factor = 10.0f64.powi(decimals as i32) as f32;
        let a = (self * factor).trunc();
        let b = (other * factor).trunc();
        a == b
    }
}

impl AproxEq<f64> for f64 {
    fn aproxEq(&self, other: f64, decimals: usize) -> bool {
        let factor = 10.0f64.powi(decimals as i32);
        let a = (self * factor).trunc();
        let b = (other * factor).trunc();
        a == b
    }
}
