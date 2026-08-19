use sal_core::dbg::Dbg;
use crate::services::frdm_service::{Block, BlockArcs, BlockBind, RopeConf};

///
/// 10. Определение опорных точек по длине каната
pub struct Bendings {
    /// Длина каната на лебедке в парковочном положении, мм
    winch_len: f64,
    block_arcs: BlockArcs,
    dbg: Dbg,
}
//
//
impl Bendings {
    ///
    /// Returns [Bendings] new instance
    pub fn new(parent: impl Into<String>, conf: &RopeConf, mut block_arcs: BlockArcs) -> Self {
        let dbg = Dbg::new(parent, "Bendings");
        let rope_len = conf.length.as_mm();
        log::debug!("{dbg}.new | Evaluating parking position...");
        Self {
            // rope_len,
            winch_len: match block_arcs.eval() {
                Some(blocks) => {
                    // для парковочного положения
                    // winch_len = общая длина  - арки - прямые - 1200
                    let len = blocks.iter().fold(rope_len, |len, block| {
                        log::debug!("{dbg}.new | Block[{}] {:?} len {:.3} mm - wrap {:.3} mm - rope {:.3}", block.name, block.bind, len, block.wrap_length, block.rope_len_fwd);
                        match block.bind {
                            BlockBind::Drum => {
                                log::debug!("{dbg}.new | Block[{}] rope bck {:.3}", block.name, block.rope_len_bck);
                                len - block.rope_len_bck - block.rope_len_fwd
                            }
                            _ => len - block.wrap_length - block.rope_len_fwd,
                        }
                        // log::debug!("{dbg}.new | Block[{i}] rope len: {:.3} mm", len);
                    });
                    log::debug!("{dbg}.new | Evaluating parking position - Ok, winch_len: {:.3} mm", len);
                    len
                },
                None => {
                    log::error!("{dbg}.new | Can't evaluate parking position calculations, winch_len set to default 0.0 mm");
                    0.0
                }
            },
            block_arcs,
            dbg,
        }
    }
    ///
    /// Возвращает опорные точки каната в миллиметрах,
    /// то есть точки входа и выхода каната с блоков
    ///
    /// - `rope_pos` - Rope position, mm.
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
    pub fn eval(&mut self, rope_pos: Option<f64>) -> Option<Vec<Block>> {
        // let t = std::time::Instant::now();
        let mut blocks = self.block_arcs.eval()?;
        let Some(rope_pos) = rope_pos else {
            log::warn!("{}.eval | Rope position isn't ready", self.dbg);
            return None;
        };
        // log::debug!("{}.eval | rope pos: {:.3} mm", self.dbg, rope_pos);
        let mut start = 0.0;           // Точка входа каната на блок (по направлению от барабана к крюку)
        let mut end = 0.0;                               // Точка схода каната с блока (по направлению от барабана к крюку)
        let mut prev_bend = start .. end;                 // Первый вход..сход считаем на крюке
        blocks.iter_mut().for_each(|block| {
            if block.skipped { return; }
            // log::debug!("{}.eval | Block {} {:?}, rope_len_fwd: {:.3}, wrap_length: {:.3}", self.dbg, block.name, block.bind, block.rope_len_fwd, block.wrap_length);
            start = match block.bind {
                BlockBind::Drum => (self.winch_len - block.wrap_length - rope_pos).max(0.0),  // На барабане считаем кусочек каната длиной в один сегмент до точки схода,
                BlockBind::Fixed => prev_bend.end,
                BlockBind::Boom(_) => prev_bend.end,
                BlockBind::Hook => prev_bend.end,
            };
            // if block.wrap_delta > f64::EPSILON || block.wrap_delta < -f64::EPSILON {
            //     log::debug!("{}.eval | Block {}: {:?}, wrap_delta: {:.3}", self.dbg, block.name, block.bind, block.wrap_delta);
            // }
            end = match block.bind {
                BlockBind::Drum => start + block.wrap_length + block.wrap_delta,
                BlockBind::Fixed => start + block.wrap_length,  // wrap_delta не добавлена, так как должна автоматически быть учтена по ходу расчета. TODO: Проверь это!
                BlockBind::Boom(_) => start + block.wrap_length,
                BlockBind::Hook => start + block.wrap_length,
            };
            prev_bend = start .. end + block.rope_len_fwd;
            if (end - start).abs() > 0.0 {
                block.bending = start .. end;
            }
        });
        // log::debug!("{}.eval | Elapsed: {:?}", self.dbg, t.elapsed());
        // log::debug!("{} | Blocks: {:?}", self.dbg, result.len());
        Some(blocks)
    }
}
