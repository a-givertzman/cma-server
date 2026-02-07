use rustfft::num_complex::Complex;
///
/// Implements an SamplingFreq usable for FFT
/// - Holds Δt = 1 / `freq`
/// - Calculates any time moment corresponding to Δt
// #[derive(Debug)]
pub struct SamplingFreq {
    /// Specified frequency, Hz
    pub freq: usize,
    step: usize,
    circular_step: usize,
    complex: Box<dyn Iterator<Item = Complex<f64>>>, //Vec<Complex<f64>>,
    /// Angular frequency `ω = 2π•f`
    pub pi2f: f64,
    delta: f64,
}
//
//
impl std::fmt::Debug for SamplingFreq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SamplingFreq")
            .field("freq", &self.freq)
            .field("step", &self.step)
            .field("circular_step", &self.circular_step)
            // .field("complex", &self.complex)
            .field("pi2f", &self.pi2f)
            .field("delta", &self.delta)
            .finish()
    }
}
//
//
impl SamplingFreq {
    ///
    /// `Returns 2π`
    pub const PI2: f64 = std::f64::consts::PI * 2.0;
    ///
    /// Returns new instance of SamplingFreq corresponding to `freq`
    pub fn new(freq: usize, fft_size: usize) -> Self {
        let angles: Vec<f64> = (0..fft_size).map(|i| {
            (i as f64) * Self::PI2 / (fft_size as f64)
        }).collect();
        let complex = angles.into_iter().map(|angle| {
            Complex {
                re: angle.cos(),
                im: angle.sin()
            }
        }).cycle();
        let sampling_period = 1.0 / (freq as f64);
        let delta = sampling_period / (fft_size as f64);
        Self {
            freq,
            step: 0,
            circular_step: 0,
            complex: Box::new(complex),
            pi2f: Self::PI2 * freq as f64,
            delta,
        }
    }
    ///
    /// Returns next `(time, unit_complex)`, (sec, rad, rel.units)
    /// - Having Δt = 1 / `freq`
    pub fn next(&mut self) -> (f64, Complex<f64>) {
        let t = (self.step as f64) * self.delta;
        // let angle = self.angles[self.step];
        // let complex = self.complex[self.circular_step];
        let complex = self.complex.next().unwrap();
        self.circular_step = (self.circular_step  + 1) % self.freq;
        self.step += 1;
        (t, complex)
    }
    ///
    /// Resets current step to zero, time begins from 0.0
    pub fn reset(&mut self) {
        self.step = 0;
        self.circular_step = 0;
    }
}
