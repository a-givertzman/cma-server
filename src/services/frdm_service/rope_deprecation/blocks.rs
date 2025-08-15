use sal_core::dbg::Dbg;
use sal_sync::collections::FxIndexMap;
use crate::services::frdm_service::{rotate_xy, Block, BlockBind, BlockConf, Boom, Booms, Offset};

///
/// Evaluation for the crane boom's collection
pub struct Blocks {
    items: Vec<Block>,
    booms: Booms,
    dbg: Dbg,
}
//
//
impl Blocks {
    ///
    /// Returns [Boom] new instance
    pub fn new(parent: impl Into<String>, conf: &Vec<(String, BlockConf)>, booms: Booms) -> Self {
        Self {
            items: conf.iter().map(|(key, conf)| Block::new(
                key,
                Offset::new(conf.lf.x.as_mm(), conf.lf.y.as_mm()),
                conf.d.as_mm(),
                conf.scheme,
                conf.bind,
            )).collect(),
            booms,
            dbg: Dbg::new(parent, "Blocks"),
        }
    }
    ///
    /// Evaluates Boom's values using passed new parameters
    pub fn eval(&mut self, inputs: &FxIndexMap<String, f64>) -> Option<Vec<Block>> {
        self.booms.eval(inputs).map(|booms| {
            self.blocks_pos(&booms);
            self.items.clone()
        })
    }
    ///
    /// 4. Координаты блоков X, Y
    fn blocks_pos(&mut self, booms: &Vec<Boom>) {
        for (idx, block) in self.items.iter_mut().enumerate() {
            match block.bind {
                BlockBind::Fixed => {
                    // Формула из алгоритма:
                    let Offset{x: dx1, y: dy1} = rotate_xy(- block.lf.x, block.lf.y, 0.0);
                    let Offset{x: dx2, y: dy2} = rotate_xy(booms[0].l4, booms[0].l3, 90.0);  // от первой стрелы
                    let x = dx1 + dx2;
                    let y = dy1 + dy2;
                    block.pos.x = x;
                    block.pos.y = y;
                }
                BlockBind::Boom(boom_index) => {
                    // Определяем номер стрелы
                    // boom_num = int(feature.split()[0]) - 1
                    log::debug!("Блок {idx}");
                    let base_point = booms[boom_index].gpt;  // точка G
                    let Offset{x: dx, y: dy} = rotate_xy(block.lf.x, block.lf.y, booms[boom_index].alpha);
                    block.pos = Offset::new(base_point.x + dx, base_point.y + dy);
                }
                BlockBind::Hook => {
                    block.pos.x = f64::NAN;
                    block.pos.y = f64::NAN;
                }
            }
        }
    }

}
