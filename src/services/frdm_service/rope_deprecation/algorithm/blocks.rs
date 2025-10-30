use sal_core::dbg::Dbg;
use crate::services::frdm_service::{rope_deprecation::rotate_xy, Block, BlockBind, BlockConf, Boom, Booms, Offset};

///
/// Evaluation for the crane `Block`'s collection
/// 4. Координаты блоков X, Y
/// - First one is always a `Winch drum`
/// - Next - are regular block from `Winch` towards `Hook`
pub struct Blocks {
    items: Vec<Block>,
    hook_l: f64,
    booms: Booms,
    dbg: Dbg,
}
//
//
impl Blocks {
    ///
    /// Returns [Boom] new instance
    pub fn new(parent: impl Into<String>, hook_l: f64, conf: &Vec<(String, BlockConf)>, booms: Booms) -> Self {
        Self {
            hook_l,
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
                self.blocks_pos(&booms).map(|_| {
                    // log::debug!("{} | Blocks: {:?}", self.dbg, self.items.len());
                    self.items.clone()
                })
            },
            None => None,
        }
    }
    ///
    /// 4. Координаты блоков X, Y
    fn blocks_pos(&mut self, booms: &Vec<Boom>) -> Option<()> {
        match self.items.first() {
            Some(first) => {
                let mut prev = first.pos;
                let mut prev_d = first.diameter;
                for (_, block) in self.items.iter_mut().enumerate() {
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
                            let base_point = booms[boom_index].gpt;  // точка G
                            let Offset{x: dx, y: dy} = rotate_xy(block.lf.x, block.lf.y, booms[boom_index].alpha);
                            block.pos = Offset::new(base_point.x + dx, base_point.y + dy);
                        }
                        BlockBind::BoomPair(boom_index) => {
                            let base_point = booms[boom_index].gpt;  // точка G
                            let Offset{x: dx, y: dy} = rotate_xy(block.lf.x, block.lf.y, booms[boom_index].alpha);
                            block.pos = Offset::new(base_point.x + dx, base_point.y + dy);
                        }
                        BlockBind::Hook => {
                            block.pos.x = prev.x + 0.5 * prev_d;
                            block.pos.y = prev.y - self.hook_l;
                        }
                    }
                    // log::debug!("{}.blocks_pos | Блок {} [{idx}]: pos: {:.4}, {:.4}", self.dbg, block.name, block.pos.x, block.pos.y);
                    prev = block.pos;
                    prev_d = block.diameter;
                }
                Some(())
            }
            None => {
                log::warn!("{}.boom_d_g_points | No Boom's found", self.dbg);
                None
            }
        }
    }

}
