use std::f64::consts::PI;
use sal_core::dbg::Dbg;
use sal_sync::collections::FxIndexMap;
use crate::services::frdm_service::{Block, BlockBind, LooseRopeSections};

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
                let blocks: Vec<Block> = blocks.iter().map(|block| {
                    let wrap_alpha = match block.bind {
                        BlockBind::Fixed => 0.0,
                        BlockBind::Boom(_) => f64::abs(block.rope_alpha_fwd - block.rope_alpha_bck),
                        BlockBind::Hook => f64::abs(block.rope_alpha_fwd - block.rope_alpha_bck),
                    };
                    log::trace!("{}.eval | Block {} wrap_alpha: {}°", self.dbg, block.name, wrap_alpha);
                    let wrap_length = (PI * block.diameter * 0.5 * wrap_alpha) / 180.0;
                    log::trace!("{}.eval | Block {} wrap_length: {}°", self.dbg, block.name, wrap_length);
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
                        block.rope_len_fwd,
                        block.rope_len_bck,
                        0.0..0.0,
                    )
                }).collect();
                log::debug!("{} | Blocks: {:?}", self.dbg, blocks.len());
                Some(blocks)
            }
            None => None,
        }
    }
}
