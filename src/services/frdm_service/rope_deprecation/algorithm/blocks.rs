use std::collections::VecDeque;

use sal_core::dbg::Dbg;
use crate::services::frdm_service::{rope_deprecation::rotate_xy, Block, BlockBind, BlockConf, Boom, Booms, Offset};

///
/// Evaluation for the crane `Block`'s collection
/// 4. Координаты блоков X, Y
/// - First one is always a `Winch drum`
/// - Next - are regular block from `Winch` towards `Hook`
pub struct Blocks {
    items: Vec<Block>,
    hook_l: f64,
    booms: Booms,
    #[allow(unused)]
    dbg: Dbg,
}
//
//
impl Blocks {
    ///
    /// Returns [Boom] new instance
    pub fn new(parent: impl Into<String>, hook_l: f64, conf: &Vec<(String, BlockConf)>, booms: Booms) -> Self {
        Self {
            hook_l,
            items: conf.iter().map(|(key, conf)| Block::new(
                key,
                Offset::new(conf.lf.x.as_mm(), conf.lf.y.as_mm()),
                conf.d.as_mm(),
                conf.scheme,
                conf.bind,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0..0.0,
            )).collect(),
            booms,
            dbg: Dbg::new(parent, "Blocks"),
        }
    }
    ///
    /// Evaluates Boom's values using passed new parameters
    pub fn eval(&mut self) -> Option<Vec<Block>> {
        match self.booms.eval() {
            Some(booms) => {
                let mut rope_alpha_bck = 0.0;
                let mut prev_pos = Offset::new(f64::NAN, f64::NAN);
                let mut prev_diameter = f64::NAN;
                let mut blocks = VecDeque::from(self.items.clone());
                match blocks.pop_front() {
                    Some(mut block) => {
                        let mut result = vec![];
                        while let Some(next) = blocks.pop_front() {
                            // let (k, j) = match block.scheme {
                            //     super::BlockScheme::TopTop => (-1.0, 1.0),
                            //     super::BlockScheme::TopBottom => (1.0, 1.0),
                            //     super::BlockScheme::BottomTop => (1.0, -1.0),
                            //     super::BlockScheme::BottomBottom => (-1.0, -1.0),
                            // };
                            block = self.blocks_pos(block, &booms, &prev_pos, prev_diameter);
                            log::debug!("{}.blocks_pos | Block {}: pos {:.4}, {:.4}", self.dbg, block.name, block.pos.x, block.pos.y);
                            prev_pos = block.pos;
                            prev_diameter = block.diameter;

                            let (k, j) = block.scheme.kj();
                            let l_block = block.pos.distance(next.pos);
                            // log::debug!("{}.eval | Block: {}: l_block: {:.3}", self.dbg, block1.name, l_block);
                            let alpha_block = block.pos.alpha_horiz(&next.pos, l_block);
                            // log::debug!("{}.eval | Block: {}: alpha_block: {:.3}", self.dbg, block1.name, alpha_block);
                            let rope_alpha_fwd = alpha_block + j * (0.5 * (block.diameter + k * next.diameter) / l_block).fract().asin().to_degrees();
                            if rope_alpha_fwd.is_nan() {
                                log::debug!("{}.eval | Block {} pos: {}, {}", self.dbg, block.name, block.pos.x, block.pos.y);
                                log::debug!("{}.eval | Block {} pos: {}, {}", self.dbg, next.name, next.pos.x, next.pos.y);
                                // log::debug!("{}.eval | Block: {:?}", self.dbg, block);
                                // log::debug!("{}.eval | Block: {:?}", self.dbg, next);
                                log::debug!("{}.eval | rope_alpha_bck: {:.3}", self.dbg, rope_alpha_bck);
                            }
                            // if block1.bind.is_same(BlockBind::BoomPair(0)) && block2.bind.is_same(BlockBind::BoomPair(0)) {
                            if next.bind.is(BlockBind::BoomPair(0)) {
                                if rope_alpha_fwd > 90.0 {
                                    continue;
                                }
                            }
                            block.rope_alpha_fwd = rope_alpha_fwd;
                            block.rope_alpha_bck = rope_alpha_bck;
                            result.push(block);
                            rope_alpha_bck = rope_alpha_fwd;
                            block = next;
                        }
                        Some(result)
                    }
                    None => None,
                }
            },
            None => None,
        }
    }
    ///
    /// 4. Координаты блоков X, Y
    fn blocks_pos(&self, mut block: Block, booms: &Vec<Boom>, prev_pos: &Offset<f64>, prev_diameter: f64) -> Block {
        match block.bind {
            BlockBind::Fixed => {
                // Формула из алгоритма:
                let Offset{x: dx1, y: dy1} = rotate_xy(- block.lf.x, block.lf.y, 0.0);
                let Offset{x: dx2, y: dy2} = rotate_xy(booms[0].l4, booms[0].l3, 90.0);  // от первой стрелы
                let x = dx1 + dx2;
                let y = dy1 + dy2;
                block.pos.x = x;
                block.pos.y = y;
            }
            BlockBind::Boom(index) => {
                let base_point = booms[index].gpt;  // точка G
                let Offset{x: dx, y: dy} = rotate_xy(block.lf.x, block.lf.y, booms[index].alpha);
                block.pos = Offset::new(base_point.x + dx, base_point.y + dy);
            }
            BlockBind::BoomPair(index) => {
                let base_point = booms[index].gpt;  // точка G
                let Offset{x: dx, y: dy} = rotate_xy(block.lf.x, block.lf.y, booms[index].alpha);
                block.pos = Offset::new(base_point.x + dx, base_point.y + dy);
            }
            BlockBind::Hook => {
                block.pos = Offset::new(
                    prev_pos.x + 0.5 * prev_diameter,
                    prev_pos.y - self.hook_l,
                );
            }
        }
        // log::debug!("{}.blocks_pos | Block {} [{idx}]: pos: {:.4}, {:.4}", self.dbg, block.name, block.pos.x, block.pos.y);
        block
    }
}
