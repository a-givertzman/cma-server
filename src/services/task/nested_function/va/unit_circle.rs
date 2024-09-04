use rustfft::num_complex::Complex;
///
/// Implements an UnitCircle, splitted into sectors corresponding to the specified frequence 
#[derive(Debug)]
pub struct UnitCircle {
    sampling_freq: usize,
    step: usize,
    angles: Vec<f64>,
    complex: Vec<Complex<f64>>,
}
//
//
impl UnitCircle {
    pub fn new(sampling_freq: usize) -> Self {
        let delta = std::f64::consts::PI * 2.0 / (sampling_freq as f64);
        let angles: Vec<f64> = (0..sampling_freq).map(|i| delta * (i as f64)).collect();
        let complex: Vec<Complex<f64>> = angles.iter().map(|angle| {
            Complex {
                re: angle.cos(),
                im: angle.sin()
            }
        }).collect();
        Self {
            sampling_freq,
            step: 0,
            angles,
            complex,
        }
    }
    ///
    /// Resets current angle to zero
    pub fn reset(&mut self) {
        self.step = 0;
    }
    ///
    /// Returns next angle and corresponding complex value
    pub fn next(&mut self) -> (f64, Complex<f64>) {
        self.step = (self.step + 1) % self.sampling_freq;
        (self.angles[self.step], self.complex[self.step])
    }
}
