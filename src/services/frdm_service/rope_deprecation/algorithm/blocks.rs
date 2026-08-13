use std::collections::VecDeque;
use sal_core::dbg::Dbg;
use sal_sync::services::conf::ConfDistance;
use crate::services::frdm_service::{rope_deprecation::rotate_xy, Block, BlockBind, BlockConf, Boom, Booms, Offset};

///
/// Evaluation for the crane `Block`'s collection
/// 4. Координаты блоков X, Y и угол наклона каната
/// - Коордтнаты блоков в ГСК
/// - Углы наклона к горизонту прямолинейных участков каната
/// - First one is always a `Winch Drum`
/// - Next - are fixed and regular block from `Winch` towards `Hook`
pub struct Blocks {
    items: Vec<Block>,
    aux_length: f64,
    /// Угол (к горизонту) схода каната с лебедки в парковочном положении
    winch_rope_alpha: f64,
    /// Если `true` то при первом вызове расчитывается парковочное положение,
    /// затем устанавливается в `false`.
    parking: bool,
    booms: Booms,
    #[allow(unused)]
    dbg: Dbg,
}
//
impl Blocks {
    ///
    /// Returns [Blocks] new instance
    /// - `aux_length` - Auxiliary whip line. Length of the rope from the last block located on the end of last boom to the hook
    /// - `parking` - Calculates parking position in the first step, meaning calculations will use specific angles of booms for that position
    pub fn new(parent: impl Into<String>, aux_length: ConfDistance, conf: &Vec<(String, BlockConf)>, parking: bool, booms: Booms) -> Self {
        Self {
            aux_length: aux_length.as_mm(),
            winch_rope_alpha: 0.0,
            parking,
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
                conf.deflector.map(|angle| angle.as_deg()),
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
                let mut blocks = VecDeque::from(self.items.clone());
                match blocks.pop_front() {
                    Some(mut block) => {
                        block.pos = self.blocks_pos(&block, &booms, &Block::default(), false);
                        let mut result = Vec::with_capacity(blocks.len());
                        let mut skipped = None;
                        let mut winch_dl;
                        while let Some(mut next) = blocks.pop_front() {
                            next.pos = self.blocks_pos(&next, &booms, &block, skipped.is_some());
                            // log::debug!("{}.eval | Block {}: pos {:.4}, {:.4}", self.dbg, next.name, next.pos.x, next.pos.y);
                            let (k, j) = block.scheme.kj();
                            let l_block = block.pos.distance(next.pos);
                            // log::debug!("{}.eval | Block: {}: l_block: {:.3}", self.dbg, block1.name, l_block);
                            let alpha_block = block.pos.alpha_horiz(&next.pos, l_block);
                            // log::debug!("{}.eval | Block: {}: alpha_block: {:.3}", self.dbg, block1.name, alpha_block);
                            let rope_alpha_fwd = alpha_block + j * ((0.5 * (block.diameter + k * next.diameter) / l_block).asin().to_degrees());
                            if let BlockBind::Drum = block.bind {
                                if self.parking {
                                    self.winch_rope_alpha = rope_alpha_fwd;
                                    log::debug!("{}.eval | Block: {}: winch_rope_alpha: {:.3}", self.dbg, block.name, rope_alpha_fwd);
                                    self.parking = false;
                                }
                                winch_dl = (rope_alpha_fwd - self.winch_rope_alpha).to_radians() * block.diameter * 0.5;
                                // Изменение длины каната на лебедке за счет изменения угла первой стрелы, мм
                                // Сохраняем его в расстояние от блока назад, в Bendings будет учтено в расчете длин
                                block.rope_len_bck = winch_dl;
                            }
                            // if rope_alpha_fwd.is_nan() {
                            //     log::warn!("{}.eval | Block {} pos: {}, {}", self.dbg, block.name, block.pos.x, block.pos.y);
                            //     log::warn!("{}.eval | Block {} pos: {}, {}", self.dbg, next.name, next.pos.x, next.pos.y);
                            // }
                            if let Some(deflection) = next.deflector && rope_alpha_fwd > deflection {
                            // if next.bind.is(BlockBind::BoomPair(0)) && rope_alpha_fwd > 90.0 {
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
    fn blocks_pos(&self, block: &Block, booms: &Vec<Boom>, prev: &Block, skipped: bool) -> Offset<f64> {
        match block.bind {
            BlockBind::Drum => {
                // Формула из алгоритма:
                let Offset{x: dx1, y: dy1} = rotate_xy(- block.lf.x, block.lf.y, 0.0);
                let Offset{x: dx2, y: dy2} = rotate_xy(booms[0].l4, booms[0].l3, 90.0);  // от первой стрелы
                Offset::new(dx1 + dx2, dy1 + dy2)
            }
//            BlockBind::Fixed => {
//                let Offset{x: dx1, y: dy1} = rotate_xy(- block.lf.x, block.lf.y, 0.0);
//                let Offset{x: dx2, y: dy2} = rotate_xy(booms[0].l4, booms[0].l3, 90.0);  // от первой стрелы
//                Offset::new(dx1 + dx2, dy1 + dy2)
//            }
            BlockBind::Boom(index) => {
                let base_point = booms[index].gpt;  // точка G
                let Offset{x: dx, y: dy} = rotate_xy(block.lf.x, block.lf.y, booms[index].alpha);
                Offset::new(base_point.x + dx, base_point.y + dy)
            }
            // BlockBind::BoomPair(index) => {
            //     let base_point = booms[index].gpt;  // точка G
            //     let Offset{x: dx, y: dy} = rotate_xy(block.lf.x, block.lf.y, booms[index].alpha);
            //     Offset::new(base_point.x + dx, base_point.y + dy)
            // }
            BlockBind::Hook => {
                // log::debug!("{}.blocks_pos | Prev  {} bind: {:?}  pos: {:.3}, {:.3}, D: {:.3}, new X: {:.3}", self.dbg, prev.name, prev.bind, prev.pos.x, prev.pos.y, prev.diameter * 0.5, prev.pos.x - 0.5 * prev.diameter);
                // log::debug!("{}.blocks_pos | Block {} bind: {:?}", self.dbg, block.name, block.bind);
                // log::debug!("{}.blocks_pos | Block {} bind: {:?}  winch_dl: {:.3}", self.dbg, block.name, block.bind, winch_dl);
                Offset::new(
                    match skipped {
                        true => prev.pos.x - 0.5 * prev.diameter,
                        false => prev.pos.x + 0.5 * prev.diameter,
                    },
                    prev.pos.y - self.aux_length,
                )
            }
        }
        // log::debug!("{}.blocks_pos | Block {} [{idx}]: pos: {:.4}, {:.4}", self.dbg, block.name, block.pos.x, block.pos.y);
    }
}
