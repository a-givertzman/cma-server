use sal_core::dbg::Dbg;
use sal_sync::{collections::FxIndexMap, services::entity::Point};
use crate::services::frdm_service::{Blocks, RopeConf};

///
/// Evaluation for the crane rope deprication
pub struct Deprication {
    inputs: FxIndexMap<String, f64>,
    blocks: Blocks,
    dbg: Dbg,
}
//
//
impl Deprication {
    ///
    /// Returns [Boom] new instance
    pub fn new(parent: impl Into<String>, conf: &RopeConf, blocks: Blocks, inputs: FxIndexMap<String, f64>) -> Self {
        Self {
            inputs,
            blocks,
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
                    Point::Bool(_) => log::warn!("{}.new | Point '{}' - expected numeric type, but has 'Bool'", self.dbg, event.name()),
                    Point::Int(point) => *input = point.value as f64,
                    Point::Real(point) => *input = point.value as f64,
                    Point::Double(point) => *input = point.value,
                    Point::String(_) => log::warn!("{}.new | Point '{}' - expected numeric type, but has 'String'", self.dbg, event.name()),
                    Point::Bytes(_) => log::warn!("{}.new | Point '{}' - expected numeric type, but has 'Bytes'", self.dbg, event.name()),
                }
            }
            None => log::warn!("{}.new | Unexpected Point '{}'", self.dbg, event.name()),
        }
    }
    ///
    /// Evaluates Boom's values using passed new parameters
    pub fn eval(&mut self, event: &Point) -> Option<()> {
        self.add(event);
        match self.blocks.eval(&self.inputs) {
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
