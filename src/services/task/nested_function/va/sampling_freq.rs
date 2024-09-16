use rustfft::num_complex::Complex;
///
/// Implements an SamplingFreq usable for FFT
/// - Holds Δt = 1 / `freq`
/// - Calculates any time moment corresponding to Δt
/// - Calculates angular frequency `ω = 2π•f`
/// - Calculates angle at specific `α = 𝑓(t) = 2π•f•t`
#[derive(Debug)]
pub struct SamplingFreq {
    freq: usize,
    step: f64,
    /// Angular frequency `ω = 2π•f`
    pi2f: f64,
}
//
//
impl SamplingFreq {
    ///
    /// `Returns 2π`
    pub const PI2: f64 = std::f64::consts::PI * 2.0;
    ///
    /// Returns new instance of SamplingFreq corresponding to `freq`
    pub fn new(freq: usize) -> Self {
        Self {
            freq,
            step: 0.0,
            // angles,
            // complex,
            pi2f: Self::PI2 * freq as f64,
        }
    }
    ///
    /// Resets current step to zero, time begins from 0.0
    pub fn reset(&mut self) {
        self.step = 0.0;
    }
    ///
    /// Returns next `t, secs`, having Δt = 1 / `freq`
    pub fn next(&mut self) -> f64 {
        let t = self.step / (self.freq as f64);
        self.step += 1.0;
        t
    }
    ///
    /// Returns `angle` at time `t, sec`
    /// - `α = 𝑓(t) = 2π•f•t`, radians
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
        (angle, Complex::new(amp * angle.cos(), amp * angle.sin()))
    }
}
