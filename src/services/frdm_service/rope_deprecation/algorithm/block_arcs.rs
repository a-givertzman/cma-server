use std::f64::consts::PI;
use sal_core::dbg::Dbg;
use crate::services::frdm_service::{Block, BlockBind, RopeSections};

///
/// [BlockArcs] |
/// 7. Углы обхвата и длины дуг каждого блока
/// - Углы наклона к горизонту прямолинейных участков каната
/// - Углы обхвата канатом всех  блоков
/// - Дуги обхвата канатом всех блоков
pub struct BlockArcs {
    rope_sections: RopeSections,
    #[allow(unused)]
    dbg: Dbg,
}
//
//
impl BlockArcs {
    ///
    /// Returns [BlockArcs] new instance
    pub fn new(parent: impl Into<String>, rope_sections: RopeSections) -> Self {
        Self {
            rope_sections,
            dbg: Dbg::new(parent, "BlockArcs"),
        }
    }
    ///
    /// Evaluates Block arck's
    pub fn eval(&mut self) -> Option<Vec<Block>> {
        let blocks = self.rope_sections.eval()?;
        let blocks: Vec<Block> = blocks.iter().filter_map(|block| {
            match block.skipped {
                true => {
                    log::debug!("{}.eval | Block {} SKIPED", self.dbg, block.name);
                    None
                }
                false => {
                    let wrap_alpha = match block.bind {
                        BlockBind::Drum => 0.0,
                        BlockBind::Fixed => f64::abs(block.rope_alpha_fwd - block.rope_alpha_bck),
                        BlockBind::Boom(_) => f64::abs(block.rope_alpha_fwd - block.rope_alpha_bck),
                        BlockBind::Hook => 0.0,     // TODO: implement caclultions for Hook block if exists
                    };
                    // log::trace!("{}.eval | Block {} wrap_alpha: {}°", self.dbg, block.name, wrap_alpha);
                    let wrap_length = (PI * block.diameter * 0.5 * wrap_alpha) / 180.0;
                    // log::trace!("{}.eval | Block {} wrap_length: {}°", self.dbg, block.name, wrap_length);
                    Some(Block::new(
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
                        block.deflector,
                    ))
                }
            }
        }).collect();
        // log::debug!("{} | Blocks: {:?}", self.dbg, blocks.len());
        Some(blocks)
    }
}
