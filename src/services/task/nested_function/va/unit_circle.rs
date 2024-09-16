use rustfft::num_complex::Complex;
///
/// Implements an UnitCircle, splitted into sectors corresponding to the specified frequence 
#[derive(Debug)]
pub struct UnitCircle {
    freq: usize,
    step: usize,
    global_step: f64,
    // angles: Vec<f64>,
    // complex: Vec<Complex<f64>>,
    /// `2 x Pi x f`
    pi2f: f64,
}
//
//
impl UnitCircle {
    ///
    /// `2 x Pi`
    pub const PI2: f64 = std::f64::consts::PI * 2.0;
    ///
    /// Returns new instance of UnitCircle corresponding to `freq`
    pub fn new(freq: usize) -> Self {
        // let delta = std::f64::consts::PI * 2.0 / (freq as f64);
        // let angles: Vec<f64> = (0..freq).map(|i| delta * (i as f64)).collect();
        // let complex: Vec<Complex<f64>> = angles.iter().map(|angle| {
        //     Complex {
        //         re: angle.cos(),
        //         im: angle.sin()
        //     }
        // }).collect();
        Self {
            freq,
            step: 0,
            global_step: 0.0,
            // angles,
            // complex,
            pi2f: Self::PI2 * freq as f64,
        }
    }
    ///
    /// Resets current angle to zero
    pub fn reset(&mut self) {
        self.step = 0;
    }
    ///
    /// Returns next `t, secs`, having Δt = 1 / `freq`
    pub fn next(&mut self) -> f64 {
        let t = self.global_step / (self.freq as f64);
        self.global_step += 1.0;
        t
    }
    ///
    /// Returns (`angle`, `complex`) at time `t, sec`
    /// - `angle = 2 x Pi x f x t`, radians
    pub fn angle(&self, t: f64) -> f64 {
        self.pi2f * t
    }
    ///
    /// Returns complex at time `t, sec`
    pub fn complex(&self, t: f64) -> Complex<f64> {
        let angle = self.angle(t);
        Complex::new(angle.cos(), angle.sin())
    }
    ///
    /// Returns (angle, complex) at a time `t, sec`
    /// - `angle = 2 x Pi x f x t`, radians
    /// - `complex = cos(angle) + sin(angle) i`
    pub fn at(&self, t: f64) -> (f64, Complex<f64>) {
        let angle = self.angle(t);
        (angle, Complex::new(angle.cos(), angle.sin()))
    }
    ///
    /// Returns `(angle, amp x complex)` at a time `t, sec`
    /// - `angle = 2 x Pi x f x t`, radians
    /// - `complex = cos(angle) + sin(angle) i`
    pub fn at_with(&self, t: f64, amp: f64) -> (f64, Complex<f64>) {
        let angle = self.angle(t);
        (angle, Complex::new(amp * angle.cos(), amp * angle.sin()))
    }
}
