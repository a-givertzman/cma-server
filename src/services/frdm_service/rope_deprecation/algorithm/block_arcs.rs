use std::f64::consts::PI;
use sal_core::dbg::Dbg;
use sal_sync::collections::FxIndexMap;
use crate::services::frdm_service::{Block, LooseRopeSections};

///
/// 7. Углы обхвата и длины дуг каждого блока
pub struct BlockArcs {
    loose_rope_sections: LooseRopeSections,
    dbg: Dbg,
}
//
//
impl BlockArcs {
    ///
    /// Returns [BlockArcs] new instance
    pub fn new(parent: impl Into<String>, loose_rope_sections: LooseRopeSections) -> Self {
        Self {
            loose_rope_sections,
            dbg: Dbg::new(parent, "BlockArcs"),
        }
    }
    ///
    /// Evaluates Block arck's
    pub fn eval(&mut self, inputs: &FxIndexMap<String, f64>) -> Option<Vec<Block>> {
        match self.loose_rope_sections.eval(inputs) {
            Some(blocks) => {
                let mut l_sys_arc = 0.0;
                Some(blocks.iter().map(|block| {
                    log::debug!("{}.eval | Block {} alpha_rope: {}", self.dbg, block.name, block.rope_alpha_fwd);
                    let wrap_alpha = f64::abs(block.rope_alpha_fwd - block.rope_alpha_bck);
                    let wrap_length = (PI * block.diameter * 0.5 * wrap_alpha) / 180.0;
                    l_sys_arc += wrap_length;
                    Block::new(
                        block.name.clone(),
                        block.lf,
                        block.diameter,
                        block.scheme,
                        block.bind,
                        block.rope_alpha_fwd,
                        block.rope_alpha_bck,
                        wrap_alpha,
                        wrap_length,
                        0.0,
                        0.0,
                        0.0..0.0,
                    )
                }).collect())
            }
            None => None,
        }
    }
}
