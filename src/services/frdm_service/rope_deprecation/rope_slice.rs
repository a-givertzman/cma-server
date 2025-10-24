use sal_core::dbg::Dbg;
use crate::services::frdm_service::{BendingsConf, BlockConf};

/// 
/// A atomic part of a rope, used for rope deprecation rate calculation.
/// Whole rope divided by equal slices, the index if each slice stored in the Self.id
pub struct RopeSlice {
    ix: usize,
    state: Vec<(usize, RopeSliceSate)>,
    offset: f64,
    pos: Option<f64>,
    load: Option<f64>,
    dbg: Dbg,
}
//
//
impl RopeSlice {
    ///
    /// Returns [RopeSlice] new instance
    /// - `id` - index of the current slice, keep in mind the rope devided by number of equal slices
    pub fn new(id: usize, blocks: usize, offset: f64) -> Self {
        Self {
            ix: id,
            state: (0..blocks).map(|ix| (ix, RopeSliceSate::Out)).collect(),
            offset,
            pos: None,
            load: None,
            dbg: Dbg::own(format!("RopeSlice[{id}]"))
        }
    }
    ///
    /// Returns the index of current rope slice withing a whole rope devided by equal slices
    pub fn id(&self) -> usize {
        self.ix
    }
    // ///
    // /// Registering new `pos` value (meter),
    // /// So new deprecation result can be evaluated
    // pub fn add_pos(&mut self, val: f64) {
    //     self.pos = Some(val);
    //     self.changed = Some(());
    // }
    // ///
    // /// Registering new `load` value (tonn),
    // /// So new deprecation result can be evaluated
    // pub fn add_load(&mut self, val: f64) {
    //     self.load = Some(val);
    //     self.changed = Some(());
    // }
    ///
    /// Evaluates [RopeSlice] deprecation
    /// - `pos` rope position, meter
    /// - `load` - rope load, tonn
    pub fn deprecation(&mut self, bendings: &BendingsConf, pos: f64, load: f64) -> Option<f64> {
        // match self.changed {
        //     Some(_) => {
        //         self.changed = None;
        //         match (pos, load) {
        //             (None, None) => None,
        //             (None, Some(_)) => None,
        //             (Some(_), None) => None,
        //             (Some(pos), Some(load)) => self._deprecation(bendings, pos, load),
        //         }
        //     },
        //     None => None,
        // }
        self._deprecation(bendings, pos, load)
    }
    ///
    /// 
    fn _deprecation(&mut self, bendings: &BendingsConf, pos: f64, load: f64) -> Option<f64> {
        let dbg = self.dbg.clone();
        let mut result = None;
        let pos = pos + self.offset;
        for (ix, state) in &mut self.state {
            log::trace!("{dbg} | state: {:?}", state);
            let (diameter, bending_range) = &bendings.bendings[*ix];
            match state {
                RopeSliceSate::Block => {
                    if !bending_range.contains(&pos) {
                        let deprecation = load / diameter.as_m();
                        *result.get_or_insert(0.0) += deprecation;
                        log::debug!("{dbg} | Slice[{}] -> Out({ix}),  pos: {pos},  D: {} m,  result: {:?}", self.ix, diameter.as_m(), result);
                        *state = RopeSliceSate::Out;
                    }
                }
                RopeSliceSate::Out => {
                    if bending_range.contains(&pos) {
                        let deprecation = load / diameter.as_m();
                        *result.get_or_insert(0.0) += deprecation;
                        log::debug!("{dbg} | Slice[{}] -> Block({ix}),  pos: {pos},  D: {} m,  result: {:?}", self.ix, diameter.as_m(), result);
                        *state = RopeSliceSate::Block;
                    }
                }
            }
        }
        result
    }
}
///
/// State of the [RopeSlice]
/// - `Block` - current slice intered into the block
/// - `Out` - current slice exited out of the block
#[derive(Debug)]
enum RopeSliceSate {
    Block,
    Out,
}