use std::collections::VecDeque;

use sal_core::dbg::Dbg;
use sal_sync::services::conf::ConfDistance;
use crate::services::frdm_service::{rope_deprecation::rotate_xy, Block, BlockBind, BlockConf, Boom, Booms, Offset};

///
/// Evaluation for the crane `Block`'s collection
/// 4. Координаты блоков X, Y и угол наклона каната
/// - Коордтнаты блоков в ГСК
/// - Углы наклона к горизонту прямолинейных участков каната
/// - First one is always a `Winch drum`
/// - Next - are regular block from `Winch` towards `Hook`
pub struct Blocks {
    items: Vec<Block>,
    aux_length: f64,
    /// Угол (к горизонту) схода каната с лебедки в парковочном положении
    winch_rope_alpha: Option<f64>,
    booms: Booms,
    #[allow(unused)]
    dbg: Dbg,
}
//
//
impl Blocks {
    ///
    /// Returns [Boom] new instance
    /// - `hook_l` - Auxiliary whip line. Length of the rope from the last block located on the end of last boom to the hook
    pub fn new(parent: impl Into<String>, aux_length: ConfDistance, conf: &Vec<(String, BlockConf)>, booms: Booms) -> Self {
        Self {
            aux_length: aux_length.as_mm(),
            winch_rope_alpha: None,
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
        let winch_rope_alpha =  match self.winch_rope_alpha {
            Some(winch_rope_alpha) => winch_rope_alpha,
            None => {
                match self.booms.eval() {
                    Some(booms) => {
                        let mut blocks = self.items.iter().take(2);
                        match blocks.next().cloned() {
                            Some(mut block) => {
                                block.pos = self.blocks_pos(&block, &booms, &Block::default(), 0.0, false);
                                match blocks.next().cloned() {
                                    Some(mut next) => {
                                        next.pos = self.blocks_pos(&next, &booms, &block, 0.0, false);
                                        let (k, j) = block.scheme.kj();
                                        let l_block = block.pos.distance(next.pos);
                                        let alpha_block = block.pos.alpha_horiz(&next.pos, l_block);
                                        let rope_alpha_fwd = alpha_block + j * ((0.5 * (block.diameter + k * next.diameter) / l_block).asin().to_degrees());
                                        self.winch_rope_alpha = Some(rope_alpha_fwd);
                                        rope_alpha_fwd
                                    }
                                    None => {
                                        log::error!("{}.eval | Can't evaluate 'Parking' position. At least two blocks required, but only one present.", self.dbg);
                                        0.0
                                    }
                                }
                                // log::debug!("{}.eval | Block {}: pos {:.4}, {:.4}", self.dbg, block.name, block.pos.x, block.pos.y);
                            }
                            None => {
                                log::error!("{}.eval | Can't evaluate 'Parking' position. At least two blocks required, but nothing present.", self.dbg);
                                0.0
                            },
                        }
                    }
                    None => {
                        log::error!("{}.eval | Can't evaluate 'Parking' position. Check Booms and Blocks configuration! Probably first (main) Boom 'parking' angle is missed.", self.dbg);
                        0.0
                    }
                }
            }
        };
        match self.booms.eval() {
            Some(booms) => {
                let mut blocks = VecDeque::from(self.items.clone());
                match blocks.pop_front() {
                    Some(mut block) => {
                        block.pos = self.blocks_pos(&block, &booms, &Block::default(), 0.0, false);
                        let mut result = vec![];
                        let mut skipped = None;
                        let mut winch_dl = 0.0;
                        while let Some(mut next) = blocks.pop_front() {
                            next.pos = self.blocks_pos(&next, &booms, &block, winch_dl, skipped.is_some());
                            // log::debug!("{}.eval | Block {}: pos {:.4}, {:.4}", self.dbg, next.name, next.pos.x, next.pos.y);
                            let (k, j) = block.scheme.kj();
                            let l_block = block.pos.distance(next.pos);
                            // log::debug!("{}.eval | Block: {}: l_block: {:.3}", self.dbg, block1.name, l_block);
                            let alpha_block = block.pos.alpha_horiz(&next.pos, l_block);
                            // log::debug!("{}.eval | Block: {}: alpha_block: {:.3}", self.dbg, block1.name, alpha_block);
                            let rope_alpha_fwd = alpha_block + j * ((0.5 * (block.diameter + k * next.diameter) / l_block).asin().to_degrees());
                            if let BlockBind::Fixed = block.bind {
                                winch_dl = (rope_alpha_fwd - winch_rope_alpha).to_radians() * block.diameter * 0.5;
                            };
                            // if rope_alpha_fwd.is_nan() {
                            //     log::warn!("{}.eval | Block {} pos: {}, {}", self.dbg, block.name, block.pos.x, block.pos.y);
                            //     log::warn!("{}.eval | Block {} pos: {}, {}", self.dbg, next.name, next.pos.x, next.pos.y);
                            // }
                            if next.bind.is(BlockBind::BoomPair(0)) && rope_alpha_fwd > 90.0{
                                // log::debug!("{}.eval | Block {} bind: {:?} - SKIPPED", self.dbg, next.name, next.bind);
                                next.skipped = true;
                                skipped = Some(next);
                            } else {
                                block.rope_alpha_fwd = rope_alpha_fwd;
                                next.rope_alpha_bck = rope_alpha_fwd;
                                // log::info!("{}.eval | Block {}: pos {:.4}, {:.4}", self.dbg, block.name, block.pos.x, block.pos.y);
                                result.push(block);
                                if let Some(skipped) = skipped.take() {
                                    // log::debug!("{}.eval | Block {}: pos {:.4}, {:.4}", self.dbg, skipped.name, skipped.pos.x, skipped.pos.y);
                                    result.push(skipped);
                                }
                                block = next;
                            }
                        }
                        // log::debug!("{}.eval | Block {}: pos {:.4}, {:.4}", self.dbg, block.name, block.pos.x, block.pos.y);
                        result.push(block);
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
    /// 'winch_dl' - Изменение длины каната на лебедке за счет изменения угла первой стрелы, мм
    fn blocks_pos(&self, block: &Block, booms: &Vec<Boom>, prev: &Block, winch_dl: f64, skipped: bool) -> Offset<f64> {
        match block.bind {
            BlockBind::Fixed => {
                // Формула из алгоритма:
                let Offset{x: dx1, y: dy1} = rotate_xy(- block.lf.x, block.lf.y, 0.0);
                let Offset{x: dx2, y: dy2} = rotate_xy(booms[0].l4, booms[0].l3, 90.0);  // от первой стрелы
                Offset::new(dx1 + dx2, dy1 + dy2)
            }
            BlockBind::Boom(index) => {
                let base_point = booms[index].gpt;  // точка G
                let Offset{x: dx, y: dy} = rotate_xy(block.lf.x, block.lf.y, booms[index].alpha);
                Offset::new(base_point.x + dx, base_point.y + dy)
            }
            BlockBind::BoomPair(index) => {
                let base_point = booms[index].gpt;  // точка G
                let Offset{x: dx, y: dy} = rotate_xy(block.lf.x, block.lf.y, booms[index].alpha);
                Offset::new(base_point.x + dx, base_point.y + dy)
            }
            BlockBind::Hook => {
                // log::debug!("{}.blocks_pos | Prev  {} bind: {:?}  pos: {:.3}, {:.3}, D: {:.3}, new X: {:.3}", self.dbg, prev.name, prev.bind, prev.pos.x, prev.pos.y, prev.diameter * 0.5, prev.pos.x - 0.5 * prev.diameter);
                // log::debug!("{}.blocks_pos | Block {} bind: {:?}", self.dbg, block.name, block.bind);
                log::debug!("{}.blocks_pos | Block {} bind: {:?}  winch_dl: {:.3}", self.dbg, block.name, block.bind, winch_dl);
                Offset::new(
                    match skipped {
                        true => prev.pos.x - 0.5 * prev.diameter,
                        false => prev.pos.x + 0.5 * prev.diameter,
                    },
                    prev.pos.y - self.aux_length + winch_dl,
                )
            }
        }
        // log::debug!("{}.blocks_pos | Block {} [{idx}]: pos: {:.4}, {:.4}", self.dbg, block.name, block.pos.x, block.pos.y);
    }
}
