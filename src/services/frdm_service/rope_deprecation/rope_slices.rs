use sal_core::dbg::Dbg;
use sal_sync::{collections::FxIndexMap, services::entity::Point};
use crate::services::frdm_service::{CraneConf, RopeSlice};

pub struct Input {
    val: f64,
}
///
/// The collection of [RopeSlice]
/// - Devide rope by specified in the config number of slices
/// - Calculate deprecation for each slice
pub struct RopeSlices<'a> {
    inputs: FxIndexMap<String, f64>,
    slices: Vec<RopeSlice>,
    conf: CraneConf,
    deprecation: Box<dyn Fn(usize, f64) + 'a>,
    dbg: Dbg,
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
            inputs: FxIndexMap::default(),
            slices: (0..slices).map(|slice| {
                let offset = (slice as f64) * conf.rope.segment.as_m();
                log::trace!("{dbg}.new | Slice: {slice}: offset: {:.2}", offset);
                RopeSlice::new(slice, &conf.bendings, offset)
            }).collect(),
            conf,
            deprecation: Box::new(deprecation),
            dbg,
        }
    }
    ///
    /// ### Use this method to pass a new Event contains a value for the calculation
    /// - Expected boom len / angle, rope pos / load events, for example:
    ///     - [Load.MainBoomAngle], current angle of the boom (relative axis), degrees
    ///     - [Load.RotaryBoomLen], length of the rotary boom, meter
    ///     - [Winch.EncoderBR2], current rope position, meter
    ///     - [Winch.Load], current rope load, tonn
    /// - Event mast have proper name, defined in the configured inputs, else it will be ignored
    /// - Event mast have value in proper units:
    ///     - angle: degrees
    ///     - distances: millimeters
    ///     - weight: tonn
    /// - Event can have type (else it will be ignores):
    ///     - `Int`
    ///     - `Real`
    ///     - `Double`
    pub fn add(&mut self, key: &str, event: &Point) {
        match self.inputs.get_mut(key) {
            Some(input) => {
                match event {
                    Point::Bool(_) => log::warn!("{}.new | Point '{}' expected numeric type, but has 'Bool'", self.dbg, event.name()),
                    Point::Int(point) => *input = point.value as f64,
                    Point::Real(point) => *input = point.value as f64,
                    Point::Double(point) => *input = point.value,
                    Point::String(_) => log::warn!("{}.new | Point '{}' expected numeric type, but has 'String'", self.dbg, event.name()),
                    Point::Bytes(_) => log::warn!("{}.new | Point '{}' expected numeric type, but has 'Bytes'", self.dbg, event.name()),
                }
            }
            None => log::warn!("{}.new | Unexpected Point '{}'", self.dbg, event.name()),
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