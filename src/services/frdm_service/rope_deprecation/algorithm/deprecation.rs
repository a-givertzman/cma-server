use std::{collections::HashSet, ops::Range};

use sal_core::dbg::Dbg;
use sal_sync::{collections::FxIndexMap, services::entity::Point};
use crate::services::frdm_service::{Bendings, CraneConf};

///
/// Evaluation for the crane rope deprication
pub struct Deprecation<'a> {
    inputs: FxIndexMap<String, f64>,
    subscriptions: Vec<String>,
    conf: CraneConf,
    // slices: Vec<RopeSlice>,
    //                 Block     Slices
    slices: FxIndexMap<usize, HashSet<usize>>,
    bendings: Bendings,
    results: Box<dyn Fn(&usize, f64) + 'a>,
    dbg: Dbg,
}
//
//
impl<'a> Deprecation<'a> {
    ///
    /// Returns [Boom] new instance
    /// - `results` - Callback provides deprication results as index of slice and it new deprication value
    pub fn new(parent: impl Into<String>, conf: &CraneConf, bendings: Bendings, mut subscriptions: Vec<String>, results: impl Fn(&usize, f64) + 'a) -> Self {
        let dbg = Dbg::new(parent, "Deprication");
        subscriptions.push(conf.rope.load.clone());
        subscriptions.push(conf.rope.pos.clone());
        // let slices = (conf.rope.length.as_m() / conf.rope.segment.as_m()).ceil() as usize;
        Self {
            inputs: FxIndexMap::default(),
            subscriptions,
            conf: conf.clone(),
            slices: FxIndexMap::default(),
            // slices: (0..slices).map(|slice| {
            //     let offset = (slice as f64) * conf.rope.segment.as_mm();
            //     log::trace!("{dbg}.new | Slice: {slice}: offset: {:.2} mm", offset);
            //     RopeSlice::new(slice, conf.blocks.len(), offset)
            // }).collect(),
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
                log::debug!("{}.add | Point '{}', value: {:?}", self.dbg, event.name(), event.value());
            }
            None => {
                match self.subscriptions.contains(&event.name()) {
                    true => {
                        let val = event.to_double().as_double().value;
                        self.inputs.insert(event.name(), val);
                        log::warn!("{}.add | Point '{}', value: {:?}", self.dbg, event.name(), val);
                    }
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
                // log::debug!("{} | Bendings:", self.dbg);
                // for block in &blocks {
                //     log::debug!("{} | \t Block[{}]: {:.4}..{:.4}", self.dbg, block.name, block.bending.start, block.bending.end);
                // }
                let pos = self.inputs.get(&self.conf.rope.pos);
                let load = self.inputs.get(&self.conf.rope.load);
                match (pos, load) {
                    (None, None) => {
                        log::warn!("{}.eval | Inputs '{}', '{}' - Not found", self.dbg, self.conf.rope.pos, self.conf.rope.load);
                        return None;
                    }
                    (None, Some(_)) => {
                        log::warn!("{}.eval | Input '{}' - Not found", self.dbg, self.conf.rope.pos);
                        return None;
                    }
                    (Some(_), None) => {
                        log::warn!("{}.eval | Input '{}' - Not found", self.dbg, self.conf.rope.load);
                        return None;
                    }
                    (Some(_), Some(load)) => {
                        for (block_ix, block) in blocks.iter().enumerate() {
                            let deprecation = load / (block.diameter * 0.001);
                            let current = self.slices(&block.bending);
                            // the Slices that are in self.slices but not in current
                            let exit = self.slices[block_ix].difference(&current);
                            for slice in exit {
                                // log::debug!("{dbg} | Slice[{}] -> Out({ix}),  offset: {},  D: {} m,  result: {:?}", self.ix, self.offset, block.diameter * 0.001, result);
                                (self.results)(slice, deprecation);
                            }
                            // the Slices that are in current but not in self.slices
                            let enter = current.difference(&self.slices[block_ix]);
                            for slice in enter {
                                (self.results)(slice, deprecation);
                            }
                            self.slices[block_ix] = self.slices[block_ix].intersection(&current).map(|s| *s).collect();
                        }
                        // todo!("Try to replace it with much more faster algorithm");
                        // for slice in &mut self.slices {
                        //     if let Some(deprecation) = slice.deprecation(&blocks, *load) {
                        //         (self.results)(slice.id(), deprecation);
                        //     }
                        // }
                        Some(())
                    }
                }
            },
            None => None,
        }
    }
    ///
    /// Returns slices (indexes) intersects with the bend range
    fn slices(&self, bend: &Range<f64>) -> HashSet<usize> {
        let rope_len = self.conf.rope.length.as_mm();
        let segment = self.conf.rope.segment.as_mm();
        // Номер Слайса который приходится на начало Бенда
        let first_slice = ((rope_len - bend.start) / segment).ceil() as usize;
        // Точка начала первого Слайса в Бенде
        let start_point = (first_slice as f64) * segment;
        let delta = bend.end - start_point;
        let slices = (delta / segment).ceil() as usize;
        (first_slice..first_slice + slices).collect()
    }
}
