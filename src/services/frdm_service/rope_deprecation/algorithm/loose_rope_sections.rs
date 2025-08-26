use sal_core::dbg::Dbg;
use sal_sync::collections::FxIndexMap;
use crate::services::frdm_service::{Block, Blocks, LooseRopeSection, Offset};

///
/// Rope Loose Sections
/// 6. Расчёт участков каната между блоками
pub struct LooseRopeSections {
    blocks: Blocks,
    dbg: Dbg,
}
impl LooseRopeSections {
    ///
    /// Returns [LooseRopeSections] new instance
    pub fn new(parent: impl Into<String>, blocks: Blocks) -> Self {
        Self {
            blocks,
            dbg: Dbg::new(parent, "LooseRopeSections"),
        }
    }
    ///
    /// Evaluates Boom's values using passed new parameters
    pub fn eval(&mut self, inputs: &FxIndexMap<String, f64>) -> Option<Vec<LooseRopeSection>> {
        match self.blocks.eval(inputs) {
            Some(blocks) => {
                Some(blocks.windows(2).map(|pair| {
                    let (block1, block2) = (&pair[0], &pair[1]);
                    let (k, j) = match block1.scheme {
                        super::BlockScheme::TopTop => (-1.0, 1.0),
                        super::BlockScheme::TopBottom => (1.0, 1.0),
                        super::BlockScheme::BottomTop => (1.0, -1.0),
                        super::BlockScheme::BottomBottom => (-1.0, -1.0),
                    };
                    let l_block = block1.pos.distance(block2.pos);
                    let alpha_block = Self::alpha_horiz(block1.pos, block2.pos);
                    let alpha_rope = alpha_block + j * (0.5 * (block1.d + k * block2.d) / l_block).asin().to_degrees();
                    let block1_x = block1.pos.x + j * 0.5 * block1.d * alpha_rope.to_radians().sin();
                    let block1_y = block1.pos.y + j * 0.5 * block1.d * alpha_rope.to_radians().cos();
                    let block2_x = block2.pos.x - j * k * 0.5 * block2.d * alpha_rope.to_radians().sin();
                    let block2_y = block2.pos.y - j * k * 0.5 * block2.d * alpha_rope.to_radians().cos();
                    let l_rope = Offset::new(block1_x, block1_y).distance(Offset::new(block2_x, block2_y));
                    LooseRopeSection {
                        l_block,
                        alpha_block,
                        alpha_rope,
                        block1_x,
                        block1_y,
                        block2_x,
                        block2_y,
                        l_rope,
                        alpha_rope_list: vec![],
                    }
                }).collect())
            }
            None => None,
        }
    }
    ///
    /// Угол наклона прямой к горизонту (в градусах)
    fn alpha_horiz(dot1: Offset<f64>, dot2: Offset<f64>) -> f64 {
        // Длина отрезка
        let length = dot1.distance(dot2);
        if length == 0.0 {
            return 0.0
        }
        let a =  ((dot1.y - dot2.x).asin() / length).to_degrees();
        if dot1.x <= dot2.x {
            a
        } else {
            180.0 - a
        }
    }
}