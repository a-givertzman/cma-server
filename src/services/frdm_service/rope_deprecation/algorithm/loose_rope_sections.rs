use std::collections::VecDeque;

use sal_core::dbg::Dbg;
use crate::services::frdm_service::{Block, BlockBind, Blocks, Offset};

///
/// Rope Loose Sections
/// 6. Расчёт участков каната между блоками
pub struct LooseRopeSections {
    blocks: Blocks,
    #[allow(unused)]
    dbg: Dbg,
}
impl LooseRopeSections {
    ///
    /// Returns [LooseRopeSections] new instance
    pub fn new(parent: impl Into<String>, blocks: Blocks) -> Self {
        Self {
            blocks,
            dbg: Dbg::new(parent, "LooseRopeSections"),
        }
    }
    ///
    /// Evaluates Boom's values using passed new parameters
    pub fn eval(&mut self) -> Option<Vec<Block>> {
        match self.blocks.eval() {
            Some(blocks) => {
                let mut rope_alpha_bck = 0.0;
                let mut rope_len_bck = 0.0;
                let mut blocks = VecDeque::from(blocks);
                match blocks.pop_front() {
                    Some(mut block1) => {
                        let mut result = vec![];
                        while let Some(block2) = blocks.pop_front() {
                            let (k, j) = match block1.scheme {
                                super::BlockScheme::TopTop => (-1.0, 1.0),
                                super::BlockScheme::TopBottom => (1.0, 1.0),
                                super::BlockScheme::BottomTop => (1.0, -1.0),
                                super::BlockScheme::BottomBottom => (-1.0, -1.0),
                            };
                            let l_block = block1.pos.distance(block2.pos);
                            // log::debug!("{}.eval | Block: {}: l_block: {:.3}", self.dbg, block1.name, l_block);
                            let alpha_block = Self::alpha_horiz(l_block, block1.pos, block2.pos);
                            // log::debug!("{}.eval | Block: {}: alpha_block: {:.3}", self.dbg, block1.name, alpha_block);
                            let rope_alpha_fwd = alpha_block + j * (0.5 * (block1.diameter + k * block2.diameter) / l_block).asin().to_degrees();
                            if block1.bind.is_same(BlockBind::BoomPair(0)) && block2.bind.is_same(BlockBind::BoomPair(0)) {
                                if rope_alpha_fwd > 90.0 {
                                    continue;
                                }
                            }
                            let block1_x = block1.pos.x + j * 0.5 * block1.diameter * rope_alpha_fwd.to_radians().sin();
                            let block1_y = block1.pos.y + j * 0.5 * block1.diameter * rope_alpha_fwd.to_radians().cos();
                            let block2_x = block2.pos.x - j * k * 0.5 * block2.diameter * rope_alpha_fwd.to_radians().sin();
                            let block2_y = block2.pos.y - j * k * 0.5 * block2.diameter * rope_alpha_fwd.to_radians().cos();
                            // log::debug!("{}.eval | Block: {}: {:.3}, {:.3} | Block: {}: {:.3}, {:.3}", self.dbg, block1.name, block1_x, block1_y, block2.name, block2_x, block2_y);
                            let rope_len_fwd = Offset::new(block2_x, block2_y).distance(Offset::new(block1_x, block1_y));
                            // log::debug!("{}.eval | Block: {}: rope_alpha_bck: {:.3}", self.dbg, block1.name, rope_alpha_bck);
                            // log::debug!("{}.eval | Block: {}: rope_alpha_fwd: {:.3}", self.dbg, block1.name, rope_alpha_fwd);
                            // log::debug!("{}.eval | Block: {}: rope_len_bck: {:.3}", self.dbg, block1.name, rope_len_bck);
                            // log::debug!("{}.eval | Block: {}: rope_len_fwd: {:.3}", self.dbg, block1.name, rope_len_fwd);
                            let block = Block::new(
                                block1.name.clone(),
                                block1.lf,
                                block1.diameter,
                                block1.scheme,
                                block1.bind,
                                rope_alpha_fwd,
                                rope_alpha_bck,
                                0.0,
                                0.0,
                                rope_len_fwd,
                                rope_len_bck,
                                0.0..0.0,
                            );
                            rope_alpha_bck = rope_alpha_fwd;
                            rope_len_bck = rope_len_fwd;
                            result.push(block);
                            block1 = block2;
                        }
                        result.push(Block::new(
                            block1.name.clone(),
                            block1.lf,
                            block1.diameter,
                            block1.scheme,
                            block1.bind,
                            0.0,
                            0.0,
                            0.0,
                            0.0,
                            0.0,
                            block1.rope_len_fwd,
                            0.0..0.0,
                        ));
                        Some(result)
                    }
                    None => None,
                }
                // let mut result: Vec<Block> = blocks.windows(2).map(|pair| {
                //     let (block1, block2) = (&pair[0], &pair[1]);
                // }).collect();
                // log::debug!("{} | Blocks: {:?}", self.dbg, result.len());
            }
            None => None,
        }
    }
    ///
    /// Угол наклона отрезка к горизонту (в градусах)
    /// - `length` - длина отрезка
    fn alpha_horiz(length: f64, dot1: Offset<f64>, dot2: Offset<f64>) -> f64 {
        if length == 0.0 {
            return 0.0
        }
        let a =  (((dot1.y - dot2.y) / length).asin()).to_degrees();
        if dot1.x <= dot2.x {
            a
        } else {
            180.0 - a
        }
    }
}