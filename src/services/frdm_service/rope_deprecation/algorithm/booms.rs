use std::sync::Arc;
use sal_core::dbg::Dbg;
use crate::services::frdm_service::{rope_deprecation::rotate_xy, Boom, BoomConf, InputKind, Inputs, Offset};

///
/// Evaluation for the crane boom's collection
/// 2. Угол наклона к горизонту каждой стрелы (alpha_boom)
/// 3. Матрица T (D и G для каждой стрелы)
pub struct Booms {
    items: Vec<Boom>,
    inputs: Arc<Inputs>,
    dbg: Dbg,
}
impl Booms {
    ///
    /// Returns [Booms] new instance
    /// - `subscribe` - List of `Event` names, wich required for calculation, and will acessed from the `inpurs`
    pub fn new(parent: impl Into<String>, conf: &Vec<(String, BoomConf)>, inputs: Arc<Inputs>) -> Self {
        Self {
            items: conf.iter().map(|(name, conf)| {
                let alpha = match &conf.angle {
                    InputKind::Const(angle) => InputKind::Const(*angle),
                    InputKind::Point(key) => {
                        inputs.subscribe(key);
                        InputKind::Point(key.clone())
                    }
                };
                let len = match &conf.len {
                    InputKind::Const(len) => InputKind::Const(len.as_mm()),
                    InputKind::Point(key) => {
                        inputs.subscribe(key);
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
            inputs,
            dbg: Dbg::new(parent, "Booms"),
        }
    }
    ///
    /// Evaluates Boom's values using passed new parameters
    pub fn eval(&mut self,) -> Option<Vec<Boom>> {
        match self.angles() {
            Some(booms) => {
                for (i, boom) in booms.iter().enumerate() {
                    log::debug!("{}.eval | Boom[{i}] '{}':  alpha: {:.4}", self.dbg, boom.name, boom.alpha);
                }
                self.boom_d_g_points(booms)
            }
            None => None,
        }
    }
    ///
    /// 2. Угол наклона к горизонту каждой стрелы (alpha_boom)
    fn angles(&mut self) -> Option<Vec<Boom>> {
        let mut alpha_sum = 0.0;
        let mut result = vec![];
        for (i, boom) in self.items.iter_mut().enumerate() {
            let alpha_rel = match &boom.alpha_input {
                Some(input) => match self.inputs.get(input) {
                    Some(alpha) => alpha,
                    None => {
                        log::warn!("{}.angles | Boom[{i}] '{}':  Input '{}' - Not found", self.dbg, boom.name, input);
                        return None
                    }
                }
                None => boom.alpha_rel,
            };
            alpha_sum += alpha_rel;
            boom.alpha = alpha_sum - (i as f64) * 180.0;
            result.push(boom.clone());
            // log::debug!("{}.angles | Boom[{i}] '{}':  absolute alpha: {}", self.dbg, boom.name, boom.alpha);
        }
        Some(result)
    }
    ///
    /// 3. D и G для каждой стрелы
    fn boom_d_g_points(&mut self, mut booms: Vec<Boom>) -> Option<Vec<Boom>> {
        match booms.first() {
            Some(first) => {
                let mut prev_gpt = first.gpt;
                let mut prev_alpha = first.alpha;
                for (i, boom) in booms.iter_mut().enumerate() {
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
                        Some(input) => match self.inputs.get(input) {
                            Some(len) => len,
                            None => {
                                log::warn!("{}.angles | Boom[{i}] '{}':  Input '{:?}' - Not found", self.dbg, boom.name, boom.len_input);
                                return None
                            }
                        },
                        None => boom.len,
                    };
                    let Offset{x: gx, y: gy} = rotate_xy(boom_len - boom.l2, boom.l1, boom.alpha);
                    let gpt = Offset::new(start.x + gx, start.y + gy);
                    // log::debug!(f"\t gpt={gpt}")
                    boom.dpt = dpt;
                    boom.gpt = gpt;
                    prev_gpt = boom.gpt;
                    prev_alpha = boom.alpha;
                }
                Some(booms)
            }
            None => {
                log::warn!("{}.boom_d_g_points | No Boom's found", self.dbg);
                None
            }
        }
    }
}
