use sal_core::dbg::Dbg;
use sal_sync::{collections::FxIndexMap, services::conf::ConfDistance};
use crate::services::frdm_service::{Block, BlockArcs};

///
/// 10. Определение опорных точек по длине каната
pub struct Bendings {
    pos_input: String,
    winch_rope_len0: ConfDistance,
    block_arcs: BlockArcs,
    dbg: Dbg,
}
//
//
impl Bendings {
    ///
    /// Returns [Bendings] new instance
    /// - `pos_input` - Name of input og the `Rope` position, mm
    /// rope_results,
    /// rope_loose_sections: list[RopeLooseSection],
    /// block_results
    pub fn new(parent: impl Into<String>, pos_input: String, winch_rope_len0: ConfDistance, block_arcs: BlockArcs) -> Self {
        Self {
            pos_input,
            winch_rope_len0,
            block_arcs,
            dbg: Dbg::new(parent, "BlockArcs"),
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
    pub fn eval(&mut self, inputs: &FxIndexMap<String, f64>) -> Option<Vec<Block>> {
        match self.block_arcs.eval(inputs) {
            Some(blocks) => {
                let mut result = vec![];
                match inputs.get(&self.pos_input) {
                    Some(rope_pos) => {
                        let mut enter = 0.0;                                       // Точка входа каната на блок
                        let mut exit = self.winch_rope_len0.as_mm() - *rope_pos;   // Точка схода каната с барабана, а в общем с блока
                        let mut bend = enter .. exit;                       // Первый сход считаем с барабана
                        for block in blocks {
                            enter = bend.end + block.rope_len_bck;
                            exit = enter + block.wrap_length;
                            bend = enter .. exit;
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
                        Some(result)
                    }
                    None => {
                        log::warn!("{}.eval | Input '{:?}' - Not found", self.dbg, self.pos_input);
                        return None;
                    }
                }
            }
            None => None,
        }
        // let L_winch = rope_results["L_winch"];                // мм
        // let l_sections = [r.l_rope for r in rope_loose_sections];       // мм, 6 прямых отрезков
        // let arcs       = block_results["arc_lengths"];        // мм, 6 значений
        // let F = [L_winch];   // F1 (мм)
    
        //  // первые 5 пролётов: "прямая -> дуга"
        // for i in range(5):
        //     F.append(F[-1] + l_sections[i])   // после прямой
        //     F.append(F[-1] + arcs[i+1]) 
    
        //  // шестой пролёт: только "прямая"
        // F.append(F[-1] + l_sections[5])
    
        //  //logging.debug(f"Опорные точки (м):{F}")
        // return F
    }
    
}
