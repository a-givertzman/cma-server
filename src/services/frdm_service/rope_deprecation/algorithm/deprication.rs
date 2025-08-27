use sal_core::dbg::Dbg;
use sal_sync::{collections::FxIndexMap, services::entity::Point};
use crate::services::frdm_service::{Bendings, CraneConf, RopeSlice};

///
/// Evaluation for the crane rope deprication
pub struct Deprication<'a> {
    inputs: FxIndexMap<String, f64>,
    subscriptions: Vec<String>,
    conf: CraneConf,
    slices: Vec<RopeSlice>,
    bendings: Bendings,
    results: Box<dyn Fn(usize, f64) + 'a>,
    dbg: Dbg,
}
//
//
impl<'a> Deprication<'a> {
    ///
    /// Returns [Boom] new instance
    /// - `results` - Callback provides deprication results as index of slice and it new deprication value
    pub fn new(parent: impl Into<String>, conf: &CraneConf, bendings: Bendings, mut subscriptions: Vec<String>, results: impl Fn(usize, f64) + 'a) -> Self {
        let dbg = Dbg::new(parent, "Deprication");
        subscriptions.push(conf.rope.load.clone());
        subscriptions.push(conf.rope.pos.clone());
        let slices = (conf.rope.length.as_m() / conf.rope.segment.as_m()).ceil() as usize;
        Self {
            inputs: FxIndexMap::default(),
            subscriptions,
            conf: conf.clone(),
            slices: (0..slices).map(|slice| {
                let offset = (slice as f64) * conf.rope.segment.as_m();
                log::trace!("{dbg}.new | Slice: {slice}: offset: {:.2}", offset);
                RopeSlice::new(slice, conf.blocks.len(), offset)
            }).collect(),
            bendings,
            results: Box::new(results),
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
    fn add(&mut self, event: &Point) {
        match self.inputs.get_mut(&event.name()) {
            Some(input) => {
                match event {
                    Point::Bool(_) => log::warn!("{}.add | Point '{}' - expected numeric type, but has 'Bool'", self.dbg, event.name()),
                    Point::Int(point) => *input = point.value as f64,
                    Point::Real(point) => *input = point.value as f64,
                    Point::Double(point) => *input = point.value,
                    Point::String(_) => log::warn!("{}.add | Point '{}' - expected numeric type, but has 'String'", self.dbg, event.name()),
                    Point::Bytes(_) => log::warn!("{}.add | Point '{}' - expected numeric type, but has 'Bytes'", self.dbg, event.name()),
                }
            }
            None => {
                match self.subscriptions.contains(&event.name()) {
                    true => _ = self.inputs.insert(event.name(), event.to_double().as_double().value),
                    false => log::warn!("{}.add | Unexpected Point '{}'", self.dbg, event.name()),
                }
            }
        }
    }
    ///
    /// Returns current calue from inputs by the key if exists
    pub fn get(&self, key: &str) -> Option<f64> {
        match self.inputs.get(key) {
            Some(val) => Some(*val),
            None => None,
        }
    }
    ///
    /// Evaluates Boom's values using passed new parameters
    pub fn eval(&mut self, event: &Point) -> Option<()> {
        self.add(event);
        match self.bendings.eval(&self.inputs) {
            Some(blocks) => {
                let pos = self.inputs.get(&self.conf.rope.pos);
                let load = self.inputs.get(&self.conf.rope.load);
                match (pos, load) {
                    (None, None) => {
                        log::warn!("{}.eval | Inputs '{}', '{}' - Not found", self.dbg, self.conf.rope.pos, self.conf.rope.load);
                        return None;
                    }
                    (None, Some(_)) => {
                        log::warn!("{}.eval | Input '{}' - Not found", self.dbg, self.conf.rope.load);
                        return None;
                    }
                    (Some(_), None) => {
                        log::warn!("{}.eval | Input '{}' - Not found", self.dbg, self.conf.rope.pos);
                        return None;
                    }
                    (Some(pos), Some(load)) => {
                        for slice in &mut self.slices {
                            if let Some(deprecation) = slice.deprecation(&blocks, *pos, *load) {
                                (self.results)(slice.id(), deprecation);
                            }
                        }
                        Some(())
                    }
                }
            },
            None => None,
        }
    }
}
