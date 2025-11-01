use std::collections::VecDeque;

use sal_core::dbg::Dbg;
use crate::services::frdm_service::{Block, Blocks, Offset};

///
/// Rope Sections
/// 6. Расчёт участков каната между блоками
pub struct RopeSections {
    blocks: Blocks,
    #[allow(unused)]
    dbg: Dbg,
}
impl RopeSections {
    ///
    /// Returns [RopeSections] new instance
    pub fn new(parent: impl Into<String>, blocks: Blocks) -> Self {
        Self {
            blocks,
            dbg: Dbg::new(parent, "RopeSections"),
        }
    }
    ///
    /// Evaluates Boom's values using passed new parameters
    pub fn eval(&mut self) -> Option<Vec<Block>> {
        match self.blocks.eval() {
            Some(blocks) => {
                // let mut rope_alpha_bck = 0.0;
                // let mut rope_len_bck = 0.0;
                let mut blocks = VecDeque::from(blocks);
                match blocks.pop_front() {
                    Some(mut block) => {
                        let mut result = vec![];
                        while let Some(mut next) = blocks.pop_front() {
                            if next.skipped {
                                continue;
                            }
                            let (k, j) = block.scheme.kj();
                            let block_x = block.pos.x + j * 0.5 * block.diameter * block.rope_alpha_fwd.to_radians().sin();
                            let block_y = block.pos.y + j * 0.5 * block.diameter * block.rope_alpha_fwd.to_radians().cos();
                            let next_x = next.pos.x - j * k * 0.5 * next.diameter * block.rope_alpha_fwd.to_radians().sin();
                            let next_y = next.pos.y - j * k * 0.5 * next.diameter * block.rope_alpha_fwd.to_radians().cos();
                            // log::debug!("{}.eval | Block: {}: {:.3}, {:.3} | Block: {}: {:.3}, {:.3}", self.dbg, block1.name, block1_x, block1_y, block2.name, block2_x, block2_y);
                            let rope_len_fwd = Offset::new(next_x, next_y).distance(Offset::new(block_x, block_y));
                            block.rope_len_fwd = rope_len_fwd;
                            // block.rope_len_bck = rope_len_bck;
                            result.push(block);
                            next.rope_len_bck = rope_len_fwd;
                            block = next;
                        }
                        // block.rope_len_fwd = rope_len_fwd;
                        // block.rope_len_bck = rope_len_bck;
                        result.push(block);
                        // log::debug!("{} | Blocks: {:?}", self.dbg, result.len());
                        Some(result)
                    }
                    None => None,
                }
            }
            None => None,
        }
    }
}