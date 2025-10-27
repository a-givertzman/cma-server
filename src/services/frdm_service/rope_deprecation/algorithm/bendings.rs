use std::{sync::Arc, time::Instant};
use sal_core::dbg::Dbg;
use crate::services::frdm_service::{Block, BlockArcs, BlockBind, Inputs, RopeConf};

///
/// 10. Определение опорных точек по длине каната
pub struct Bendings {
    rope_len: f64,
    segment: f64,
    block_arcs: BlockArcs,
    dbg: Dbg,
}
//
//
impl Bendings {
    ///
    /// Returns [Bendings] new instance
    pub fn new(parent: impl Into<String>, conf: &RopeConf, block_arcs: BlockArcs) -> Self {
        Self {
            rope_len: conf.length.as_mm(),
            segment: conf.segment.as_mm(),
            block_arcs,
            dbg: Dbg::new(parent, "Bendings"),
        }
    }
    ///
    /// Возвращает опорные точки каната в миллиметрах,
    /// то есть точки входа и выхода каната с блоков
    /// 
    /// Формируем опорных точек:
    ///     F1  = L_winch // длина каната на барабане до точки схода
    ///     F2  = F1 + l_rope_1
    ///     F3  = F2 + arc_1
    ///     F4  = F3 + l_rope_2
    ///     F5  = F4 + arc_2
    ///     ...
    ///     F10 = F9  + l_rope_5
    ///     F11 = F10 + arc_5
    ///     F12 = F11 + l_rope_6
    pub fn eval(&mut self, inputs: &Arc<Inputs>) -> Option<Vec<Block>> {
        let t = Instant::now();
        match self.block_arcs.eval() {
            Some(blocks) => {
                let mut result = vec![];
                match inputs.rope_pos() {
                    Some(rope_pos) => {
                        let mut start = self.rope_len - rope_pos * 1000.0;  // Точка входа каната на блок
                        let mut end = 0.0;                                  // Точка схода каната с барабана, а в общем с блока
                        let mut bend = start .. end;                 // Первый сход считаем с барабана
                        for block in blocks.iter().rev() {
                            end = bend.start - block.rope_len_fwd;
                            start = match block.bind {
                                BlockBind::Fixed => end - self.segment,
                                BlockBind::Boom(_) => end - block.wrap_length,
                                BlockBind::Hook => end - block.wrap_length,
                            };
                            bend = start .. end;
                            result.push(Block::new(
                                block.name.clone(),
                                block.lf,
                                block.diameter,
                                block.scheme,
                                block.bind,
                                block.rope_alpha_fwd,
                                block.rope_alpha_bck,
                                block.wrap_alpha,
                                block.wrap_length,
                                block.rope_len_fwd,
                                block.rope_len_bck,
                                bend.clone(),
                            ));
                        }
                        result.reverse();
                        log::debug!("{}.eval | Elapsed: {:?}", self.dbg, t.elapsed());
                        // log::debug!("{} | Blocks: {:?}", self.dbg, result.len());
                        Some(result)
                    }
                    None => {
                        log::warn!("{}.eval | Rope position isn't ready", self.dbg);
                        return None;
                    }
                }
            }
            None => None,
        }
    }
}
