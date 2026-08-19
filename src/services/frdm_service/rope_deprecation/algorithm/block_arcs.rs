use std::f64::consts::PI;
use sal_core::dbg::Dbg;
use sal_sync::services::conf::ConfDistance;
use crate::services::frdm_service::{Block, BlockBind, RopeSections};

///
/// [BlockArcs] |
/// 7. Углы обхвата и длины дуг каждого блока
/// - Углы наклона к горизонту прямолинейных участков каната
/// - Углы обхвата канатом всех  блоков
/// - Дуги обхвата канатом всех блоков
pub struct BlockArcs {
    rope_sections: RopeSections,
    /// Длина сегмента, мм
    segment: f64,
    #[allow(unused)]
    dbg: Dbg,
}
//
//
impl BlockArcs {
    ///
    /// Returns [BlockArcs] new instance
    /// - `segment` - Rope segmetn length. Whole rope will divided by the segments for the Depreciation Rate calculation.
    pub fn new(parent: impl Into<String>, segment: &ConfDistance, rope_sections: RopeSections) -> Self {
        Self {
            rope_sections,
            segment: segment.as_mm(),
            dbg: Dbg::new(parent, "BlockArcs"),
        }
    }
    ///
    /// Evaluates Block arck's
    pub fn eval(&mut self) -> Option<Vec<Block>> {
        let mut blocks = self.rope_sections.eval()?;
        blocks.iter_mut().for_each(|block| {
            if block.skipped {
                // log::debug!("{}.eval | Block {} SKIPED", self.dbg, block.name);
                return;   // Пропускаем элемент
            }
            let wrap_alpha = match block.bind {
                BlockBind::Drum => 0.0,
                BlockBind::Fixed => f64::abs(block.rope_alpha_fwd - block.rope_alpha_bck),
                BlockBind::Boom(_) => f64::abs(block.rope_alpha_fwd - block.rope_alpha_bck),
                BlockBind::Hook => 0.0,     // TODO: implement caclultions for Hook block if exists
            };
            // log::trace!("{}.eval | Block {} wrap_alpha: {}°", self.dbg, block.name, wrap_alpha);
            let wrap_length = match block.bind {
                BlockBind::Drum => self.segment,
                _ => (PI * block.diameter * 0.5 * wrap_alpha) / 180.0,
            };
            // log::trace!("{}.eval | Block {} wrap_length: {}°", self.dbg, block.name, wrap_length);
            block.wrap_alpha = wrap_alpha;
            block.wrap_length = wrap_length;
        });
        Some(blocks)
    }
}
