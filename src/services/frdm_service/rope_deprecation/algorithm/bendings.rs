use std::{sync::Arc, time::Instant};
use sal_core::dbg::Dbg;
use crate::services::frdm_service::{Block, BlockArcs, BlockBind, Inputs, RopeConf};

///
/// 10. Определение опорных точек по длине каната
pub struct Bendings {
    /// Total working length of the rope, mm
    rope_len: f64,
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
                match inputs.rope_pos() {
                    Some(rope_pos) => {
                        let mut start = self.rope_len - rope_pos;           // Точка входа каната на блок (по направлению от барабана к крюку)
                        let mut end =  start;                               // Точка схода каната с блока (по направлению от барабана к крюку)
                        let mut prev_bend = start .. end;                 // Первый вход..сход считаем на крюке
                        let mut result: Vec<Block> = blocks.into_iter().rev().filter_map(|mut block| {
                            log::debug!("{}.eval | Block {} {:?}, rope_len_fwd: {:.3}, wrap_length: {:.3}", self.dbg, block.name, block.bind, block.rope_len_fwd, block.wrap_length);
                            match block.skipped {
                                true => None,
                                false => {
                                    end = prev_bend.start - block.rope_len_fwd;
                                    start = match block.bind {
                                        BlockBind::Fixed => 0.0, // На барабане считаем весь канат от начала до точки схода,
                                        BlockBind::Boom(_) => end - block.wrap_length,
                                        BlockBind::BoomPair(_) => end - block.wrap_length,
                                        BlockBind::Hook => end - block.wrap_length,
                                    };
                                    prev_bend = start .. end;
                                    match (end - start).abs() > 0.0 {
                                        true => {
                                            block.bending = start .. end;
                                            Some(block)
                                        }
                                        false => None,
                                    }
                                }
                            }
                        }).collect();
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
