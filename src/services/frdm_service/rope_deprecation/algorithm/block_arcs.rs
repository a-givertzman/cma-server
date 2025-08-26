use std::f64::consts::PI;

use sal_core::dbg::Dbg;
use sal_sync::collections::FxIndexMap;
use crate::services::frdm_service::{Block, LooseRopeSection, LooseRopeSections};

///
/// 7. Углы обхвата и длины дуг каждого блока
pub struct BlockArcs {
    loose_rope_sections: LooseRopeSections,
    dbg: Dbg,
}
//
//
impl BlockArcs {
    ///
    /// Returns [BlockArcs] new instance
    pub fn new(parent: impl Into<String>, loose_rope_sections: LooseRopeSections) -> Self {
        Self {
            loose_rope_sections,
            dbg: Dbg::new(parent, "BlockArcs"),
        }
    }
    ///
    /// Evaluates Boom's values using passed new parameters
    pub fn eval(&mut self, inputs: &FxIndexMap<String, f64>) -> Option<Vec<Block>> {
        match self.loose_rope_sections.eval(inputs) {
            Some(blocks) => {
                let mut wrap_angles = vec![];
                let mut arc_lengths = vec![];
                let mut l_sys_arc = 0.0;
                let mut prev_alpha = None;  // предыдущий угол
                for block in blocks {
                    // let alpha_rope = block.alpha_rope_fwd;
                    log::debug!("{}.eval | alpha_rope: {}", self.dbg, block.alpha_rope_fwd);

                    // Формируем alpha_rope_list
                    let alpha_wrap = match prev_alpha {
                        Some(prev_alpha) => {
                            f64::abs(block.alpha_rope_fwd - prev_alpha)
                        }
                        None => 0.0,
                    };
                    let l_arc = (PI * block.d * 0.5 * alpha_wrap) / 180.0;
                    l_sys_arc += l_arc;
                    wrap_angles.push(alpha_wrap);
                    arc_lengths.push(l_arc);

                    prev_alpha = Some(block.alpha_rope_fwd);  // обновляем предыдущий угол
                }
                None
                // return {
                //     "wrap_angles": wrap_angles,
                //     "arc_lengths": arc_lengths,
                //     "L_sys_arc": L_sys_arc
                // }

            }
            None => None,
        }
    }
}
