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
                let mut rope_len_bck = 0.0;
                let mut blocks = VecDeque::from(blocks);
                match blocks.pop_front() {
                    Some(mut block) => {
                        let mut result = vec![];
                        while let Some(next) = blocks.pop_front() {
                            let (k, j) = block.scheme.kj();
                            // let l_block = block.pos.distance(next.pos);
                            // // log::debug!("{}.eval | Block: {}: l_block: {:.3}", self.dbg, block1.name, l_block);
                            // let alpha_block = block.pos.alpha_horiz(&next.pos, l_block);
                            // // log::debug!("{}.eval | Block: {}: alpha_block: {:.3}", self.dbg, block1.name, alpha_block);
                            // let rope_alpha_fwd = alpha_block + j * (0.5 * (block.diameter + k * next.diameter) / l_block).fract().asin().to_degrees();
                            // // if block1.bind.is_same(BlockBind::BoomPair(0)) && block2.bind.is_same(BlockBind::BoomPair(0)) {
                            // if next.bind.is(BlockBind::BoomPair(0)) {
                            //     if rope_alpha_fwd > 90.0 {
                            //         continue;
                            //     }
                            // }
                            let block_x = block.pos.x + j * 0.5 * block.diameter * block.rope_alpha_fwd.to_radians().sin();
                            let block_y = block.pos.y + j * 0.5 * block.diameter * block.rope_alpha_fwd.to_radians().cos();
                            let next_x = next.pos.x - j * k * 0.5 * next.diameter * block.rope_alpha_fwd.to_radians().sin();
                            let next_y = next.pos.y - j * k * 0.5 * next.diameter * block.rope_alpha_fwd.to_radians().cos();
                            // log::debug!("{}.eval | Block: {}: {:.3}, {:.3} | Block: {}: {:.3}, {:.3}", self.dbg, block1.name, block1_x, block1_y, block2.name, block2_x, block2_y);
                            let rope_len_fwd = Offset::new(next_x, next_y).distance(Offset::new(block_x, block_y));
                            // log::debug!("{}.eval | Block: {}: rope_alpha_bck: {:.3}", self.dbg, block1.name, rope_alpha_bck);
                            // log::debug!("{}.eval | Block: {}: rope_alpha_fwd: {:.3}", self.dbg, block1.name, rope_alpha_fwd);
                            // log::debug!("{}.eval | Block: {}: rope_len_bck: {:.3}", self.dbg, block1.name, rope_len_bck);
                            // log::debug!("{}.eval | Block: {}: rope_len_fwd: {:.3}", self.dbg, block1.name, rope_len_fwd);
                            // if rope_alpha_fwd.is_nan() {
                            //     log::debug!("{}.eval | Block {} pos: {}, {}", self.dbg, block.name, block.pos.x, block.pos.y);
                            //     log::debug!("{}.eval | Block {} pos: {}, {}", self.dbg, next.name, next.pos.x, next.pos.y);
                            //     // log::debug!("{}.eval | Block: {:?}", self.dbg, block);
                            //     // log::debug!("{}.eval | Block: {:?}", self.dbg, next);
                            //     log::debug!("{}.eval | rope_alpha_bck: {:.3}", self.dbg, rope_alpha_bck);
                            //     log::debug!("{}.eval | rope_len_bck: {:.3}", self.dbg, rope_len_bck);
                            // }
                            block.rope_len_fwd = rope_len_fwd;
                            block.rope_len_bck = rope_len_bck;
                            result.push(block);
                            // Block::new(
                            //     block.name.clone(),
                            //     block.lf,
                            //     block.diameter,
                            //     block.scheme,
                            //     block.bind,
                            //     block.rope_alpha_fwd,
                            //     block.rope_alpha_bck,
                            //     block.wrap_alpha,
                            //     block.wrap_length,
                            //     rope_len_fwd,
                            //     rope_len_bck,
                            //     block.bending,
                            // ));
                            rope_len_bck = rope_len_fwd;
                            block = next;
                        }
                        // block.rope_len_fwd = rope_len_fwd;
                        block.rope_len_bck = rope_len_bck;
                        result.push(block);
                        // Block::new(
                        //     block.name.clone(),
                        //     block.lf,
                        //     block.diameter,
                        //     block.scheme,
                        //     block.bind,
                        //     block.rope_alpha_fwd,
                        //     rope_alpha_bck,
                        //     block.wrap_alpha,
                        //     block.wrap_length,
                        //     block.rope_len_fwd,
                        //     rope_len_bck,
                        //     block.bending,
                        // ));
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