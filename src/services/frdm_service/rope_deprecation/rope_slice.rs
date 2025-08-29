use sal_core::dbg::Dbg;
use crate::services::frdm_service::Block;

/// 
/// A atomic part of a rope, used for rope deprecation rate calculation.
/// Whole rope divided by equal slices, the index if each slice stored in the Self.id
pub struct RopeSlice {
    ix: usize,
    state: Vec<(usize, RopeSliceSate)>,
    offset: f64,
    dbg: Dbg,
}
//
//
impl RopeSlice {
    ///
    /// Returns [RopeSlice] new instance
    /// - `id` - index of the current slice, keep in mind the rope devided by number of equal slices
    /// - `blocks` - Count of blocks including winch drum and block at the hook
    pub fn new(id: usize, blocks: usize, offset: f64) -> Self {
        Self {
            ix: id,
            state: (0..blocks).map(|ix| (ix, RopeSliceSate::Unknown)).collect(),
            offset,
            dbg: Dbg::own(format!("RopeSlice[{id}]"))
        }
    }
    ///
    /// Returns the index of current rope slice withing a whole rope devided by equal slices
    pub fn id(&self) -> usize {
        self.ix
    }
    ///
    /// Evaluates [RopeSlice] deprecation
    /// - `pos` rope position, mm
    /// - `load` - rope load, tonn
    pub fn deprecation(&mut self, blocks: &Vec<Block>, pos: f64, load: f64) -> Option<f64> {
        let dbg = self.dbg.clone();
        let mut result = None;
        let pos = pos + self.offset;
        // log::debug!("{dbg} | Slice[{}]: {:.4}", self.ix, pos);
        for (ix, state) in &mut self.state {
            // log::debug!("{dbg} | ix: {ix},  state: {:?}", state);
            let block = &blocks[*ix];
            match state {
                RopeSliceSate::Block => {
                    if !block.bending.contains(&pos) {
                        let deprecation = load / (block.diameter * 0.001);
                        *result.get_or_insert(0.0) += deprecation;
                        log::debug!("{dbg} | Slice[{}] -> Out({ix}),  pos: {pos},  D: {} m,  result: {:?}", self.ix, block.diameter * 0.001, result);
                        *state = RopeSliceSate::Out;
                    }
                }
                RopeSliceSate::Out => {
                    if block.bending.contains(&pos) {
                        let deprecation = load / (block.diameter * 0.001);
                        *result.get_or_insert(0.0) += deprecation;
                        log::debug!("{dbg} | Slice[{}] -> Block({ix}),  pos: {pos},  D: {} m,  result: {:?}", self.ix, block.diameter * 0.001, result);
                        *state = RopeSliceSate::Block;
                    }
                }
                RopeSliceSate::Unknown => {
                    if block.bending.contains(&pos) {
                        *state = RopeSliceSate::Block;
                    } else {
                        *state = RopeSliceSate::Out;
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
    Unknown,
}