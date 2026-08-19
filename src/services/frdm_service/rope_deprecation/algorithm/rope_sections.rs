use sal_core::dbg::Dbg;
use crate::services::frdm_service::{Block, BlockBind, Blocks, Offset};

///
/// [RopeSections] |
/// 6. Расчёт прямолинейных участков каната между блоками
/// - Длина прямолинейных участков
/// - Расчет углов наклона - перенесен в `Blocks`
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
        let mut blocks = self.blocks.eval()?;
        let mut blocks_iter = blocks.iter_mut();
        let Some(mut block) = blocks_iter.next() else {
            log::warn!("{}.eval | No blocks found", self.dbg);
            return None;
        };
        if !block.bind.is(BlockBind::Drum) {
            log::warn!("{}.eval | First block expected 'Drum', but found {:?}", self.dbg, block.bind);
            return None;
        }
        while let Some(next) = blocks_iter.next() {
            if next.skipped { continue; }
            let (k, j) = block.scheme.kj();
            let block_x = block.pos.x + j * 0.5 * block.diameter * block.rope_alpha_fwd.to_radians().sin();
            let block_y = block.pos.y + j * 0.5 * block.diameter * block.rope_alpha_fwd.to_radians().cos();
            let next_x = next.pos.x - j * k * 0.5 * next.diameter * block.rope_alpha_fwd.to_radians().sin();
            let next_y = next.pos.y - j * k * 0.5 * next.diameter * block.rope_alpha_fwd.to_radians().cos();
            // log::debug!("{}.eval | Block: {}: {:.3}, {:.3} | Block: {}: {:.3}, {:.3}", self.dbg, block1.name, block1_x, block1_y, block2.name, block2_x, block2_y);
            let rope_len_fwd = Offset::new(next_x, next_y).distance(Offset::new(block_x, block_y));
            block.rope_len_fwd = rope_len_fwd;
            next.rope_len_bck = rope_len_fwd;
            block = next;
        }
        // log::debug!("{} | Blocks: {:?}", self.dbg, result.len());
        Some(blocks)
    }
}
