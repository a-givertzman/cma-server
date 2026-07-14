use rustfft::num_complex::Complex;
///
/// Implements an UnitCircle usable for FFT, 
/// - Splitted into sectors corresponding to the specified frequence 
/// - Calculates angular frequency `ω = 2π•f`
/// - Calculates angle at specific `α = 𝑓(t) = 2π•f•t`
#[derive(Debug)]
pub struct UnitCircle {
    pub freq: usize,
    /// Angular frequency `ω = 2π•f`
    pi2f: f64,
}
//
//
impl UnitCircle {
    ///
    /// `Returns 2π`
    pub const PI2: f64 = std::f64::consts::PI * 2.0;

    ///
    /// Returns new instance of SamplingFreq corresponding to `freq`
    /// - `freq` - sampling frequency, Hz
    pub fn new(freq: usize) -> Self {
        Self {
            freq,
            pi2f: Self::PI2 * freq as f64,
        }
    }
    ///
    /// Returns `angle` in radians at time `t, sec`
    /// - `α = 𝑓(t) = 2π•f•t`
    pub fn angle(&self, t: f64) -> f64 {
        self.pi2f * t
    }
    ///
    /// Returns complex value at time `t, sec`
    /// - `α = 𝑓(t) = 2π•f•t`, radians
    /// - `complex = cos(α) + sin(α)i`
    pub fn complex(&self, t: f64) -> Complex<f64> {
        let angle = self.angle(t);
        Complex::new(angle.cos(), angle.sin())
    }
    ///
    /// Returns (α, complex) at a time `t, sec`
    /// - `α = 𝑓(t) = 2π•f•t`, radians
    /// - `complex = cos(α) + sin(α)i`
    pub fn at(&self, t: f64) -> (f64, Complex<f64>) {
        let angle = self.angle(t);
        (angle, Complex::new(angle.cos(), angle.sin()))
    }
    ///
    /// Returns `(α, amp • complex)` at a time `t, sec`
    /// - `α = 𝑓(t) = 2π•f•t`, radians
    /// - `complex = amp•cos(α) + amp•sin(α)i`
    pub fn at_with(&self, t: f64, amp: f64) -> (f64, Complex<f64>) {
        let angle = self.angle(t);
        self.at_angle_with(angle, amp)
    }
    ///
    /// Returns `(α, complex)` at an `angle, rad`
    /// - `complex = cos(α) + sin(α)i`
    pub fn at_angle(&self, angle: f64) -> (f64, Complex<f64>) {
        (angle, Complex::new(angle.cos(), angle.sin()))
    }
    ///
    /// Returns `(α, amp • complex)` at an `angle, rad` with `amp` multiplier
    /// - `complex = amp•cos(α) + amp•sin(α)i`
    pub fn at_angle_with(&self, angle: f64, amp: f64) -> (f64, Complex<f64>) {
        (angle, Complex::new(amp * angle.cos(), amp * angle.sin()))
    }
}
