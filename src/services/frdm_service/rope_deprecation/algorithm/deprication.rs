use sal_core::dbg::Dbg;
use sal_sync::{collections::FxIndexMap, services::entity::Point};
use crate::services::frdm_service::{BlockArcs, Blocks, RopeConf};

///
/// Evaluation for the crane rope deprication
pub struct Deprication {
    inputs: FxIndexMap<String, f64>,
    subscriptions: Vec<String>,
    block_arcs: BlockArcs,
    dbg: Dbg,
}
//
//
impl Deprication {
    ///
    /// Returns [Boom] new instance
    pub fn new(parent: impl Into<String>, conf: &RopeConf, block_arcs: BlockArcs, mut subscriptions: Vec<String>) -> Self {
        subscriptions.push(conf.load.clone());
        subscriptions.push(conf.pos.clone());
        Self {
            inputs: FxIndexMap::default(),
            subscriptions,
            block_arcs,
            dbg: Dbg::new(parent, "Deprication"),
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
        match self.block_arcs.eval(&self.inputs) {
            Some(blocks) => {
                todo!("
                    - Add implementeation of `Bendings` calculation
                    - Add here combination of `deprication` methods from `RopeSlice` & `RopeSlices`
                ");
            },
            None => None,
        }
    }
}
