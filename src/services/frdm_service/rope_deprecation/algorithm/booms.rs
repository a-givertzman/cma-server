use sal_core::dbg::Dbg;
use sal_sync::collections::FxIndexMap;
use crate::services::frdm_service::{rope_deprecation::rotate_xy, Boom, BoomConf, InputKind, Offset};

///
/// Evaluation for the crane boom's collection
pub struct Booms {
    items: Vec<Boom>,
    dbg: Dbg,
}
impl Booms {
    ///
    /// Returns [Boom] new instance
    pub fn new(parent: impl Into<String>, conf: &Vec<(String, BoomConf)>, inputs: &mut FxIndexMap<String, f64>) -> Self {
        Self {
            items: conf.iter().map(|(name, conf)| {
                let alpha = match &conf.angle {
                    InputKind::Const(len) => InputKind::Const(len.as_mm()),
                    InputKind::Point(key) => {
                        inputs.insert(key.clone(), 0.0);
                        InputKind::Point(key.clone())
                    }
                };
                let len = match &conf.len {
                    InputKind::Const(len) => InputKind::Const(len.as_mm()),
                    InputKind::Point(key) => {
                        inputs.insert(key.clone(), 0.0);
                        InputKind::Point(key.clone())
                    }
                };
                Boom::new(
                    name,
                    alpha,
                    len,
                    conf.l1.as_mm(),
                    conf.l2.as_mm(),
                    conf.l3.as_mm(),
                    conf.l4.as_mm(),
                )
            }).collect(),
            dbg: Dbg::new(parent, "Booms"),
        }
    }
    ///
    /// Evaluates Boom's values using passed new parameters
    pub fn eval(&mut self, inputs: &FxIndexMap<String, f64>) -> Option<Vec<Boom>> {
        match self.angles(inputs) {
            Some(_) => self.boom_d_g_points(inputs).map(|_| self.items.clone()),
            None => None,
        }
    }
    ///
    /// 2. Угол наклона к горизонту каждой стрелы (alpha_boom)
    fn angles(&mut self, inputs: &FxIndexMap<String, f64>) -> Option<()> {
        let mut alpha_sum = 0.0;
        for (i, boom) in self.items.iter_mut().enumerate() {
            let alpha_rel = match &boom.alpha_input {
                Some(input) => match inputs.get(input) {
                    Some(alpha) => alpha,
                    None => {
                        log::warn!("{}.angles | Boom[{i}] '{}':  Input '{:?}' - Not found", self.dbg, boom.name, boom.alpha_input);
                        return None
                    }
                }
                None => &boom.alpha_rel,
            };
            alpha_sum += *alpha_rel;
            boom.alpha = alpha_sum - (i as f64) * 180.0;
            // log::debug!("{}.angles | Boom[{i}] '{}':  absolute alpha: {}", self.dbg, boom.name, boom.alpha);
        }
        Some(())
    }
    ///
    /// 3. D и G для каждой стрелы
    fn boom_d_g_points(&mut self, inputs: &FxIndexMap<String, f64>) -> Option<()> {
        match self.items.first() {
            Some(first) => {
                let mut prev_gpt = first.gpt;
                let mut prev_alpha = first.alpha;
                for (i, boom) in self.items.iter_mut().enumerate() {
                    // Начало стрелы
                    let (x0, y0, alpha_prime) = if i == 0 {
                        (0.0, 0.0, 90.0)
                    } else {
                        (prev_gpt.x, prev_gpt.y, prev_alpha)
                    };
                    let Offset{x: wx, y: wy} = rotate_xy(boom.l4, boom.l3, alpha_prime);
                    let start = Offset::new(x0 + wx, y0 + wy);
                    // log::debug!(f"Стрела {i}: start={start}")
                    // Точка D
                    let Offset{x: dx, y: dy} = rotate_xy(- boom.l2, boom.l1, boom.alpha);
                    let dpt = Offset::new(start.x + dx, start.y + dy);
                    // log::debug!(f"\t dpt={dpt}")
                    // Точка G
                    let boom_len = match &boom.len_input {
                        Some(input) => match inputs.get(input) {
                            Some(len) => len,
                            None => {
                                log::warn!("{}.angles | Boom[{i}] '{}':  Input '{:?}' - Not found", self.dbg, boom.name, boom.len_input);
                                return None
                            }
                        },
                        None => &boom.len,
                    };
                    let Offset{x: gx, y: gy} = rotate_xy(boom_len - boom.l2, boom.l1, boom.alpha);
                    let gpt = Offset::new(start.x + gx, start.y + gy);
                    // log::debug!(f"\t gpt={gpt}")
                    boom.dpt = dpt;
                    boom.gpt = gpt;
                    prev_gpt = boom.gpt;
                    prev_alpha = boom.alpha;
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