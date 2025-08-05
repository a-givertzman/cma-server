use sal_core::dbg::Dbg;
use crate::services::frdm_service::{CraneConf, RopeSlice};

///
/// The collection of [RopeSlice]
/// - Devide rope by specified in the config number of slices
/// - Calculate deprecation for each slice
pub struct RopeSlices<'a> {
    slices: Vec<RopeSlice>,
    conf: CraneConf,
    deprecation: Box<dyn Fn(usize, f64) + 'a>,
}
//
//
impl<'a> RopeSlices<'a> {
    ///
    /// Returns [RopeSlices] new instance
    /// - `deprecation` - Here will be passed evaluated deprecation for each [RopeSlice] with it's index,
    pub fn new(parent: impl Into<String>, conf: CraneConf, deprecation: impl Fn(usize, f64) + 'a) -> Self {
        let dbg = Dbg::new(parent, "RopeSlices");
        let slices = (conf.rope.length.as_m() / conf.rope.segment.as_m()).ceil() as usize;
        log::debug!("{dbg}.new | Rope: {} m, slices: {slices}, devided by {:.2} mm", conf.rope.length.as_m(), conf.rope.segment.as_mm());
        Self {
            slices: (0..slices).map(|slice| {
                let offset = (slice as f64) * conf.rope.segment.as_m();
                log::trace!("{dbg}.new | Slice: {slice}: offset: {:.2}", offset);
                RopeSlice::new(slice, &conf.bendings, offset)
            }).collect(),
            conf,
            deprecation: Box::new(deprecation),
        }
    }
    ///
    /// Registering new `pos` or/and `load` values,
    /// So new deprecation result can be evaluated, will be passed via `deprication` callback
    pub fn eval(&mut self, pos: Option<f64>, load: Option<f64>) {
        match (pos, load) {
            (None, None) => {},
            (None, Some(load)) => for slice in &mut self.slices { slice.add_load(load) },
            (Some(pos), None) => for slice in &mut self.slices { slice.add_pos(pos) },
            (Some(pos), Some(load)) => {
                for slice in &mut self.slices {
                    slice.add_pos(pos);
                    slice.add_load(load);
                }
            }
        }
        for slice in &mut self.slices {
            if let Some(deprecation) = slice.deprecation(&self.conf.bendings) {
                (self.deprecation)(slice.id(), deprecation);
            }
        }
    }
}