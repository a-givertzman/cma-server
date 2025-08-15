use sal_core::dbg::Dbg;
use crate::services::frdm_service::{rotate_xy, Boom, BoomConf, Offset};

///
/// Evaluation for the crane boom's collection
pub struct Booms {
    items: Vec<Boom>,
    dbg: Dbg,
}
impl Booms {
    ///
    /// Returns [Boom] new instance
    pub fn new(parent: impl Into<String>, conf: &Vec<(String, BoomConf)>) -> Self {
        Self {
            items: conf.iter().map(|(name, conf)| Boom::new(
                name,
                // alpha_rel: 0.0,
                // alpha: 0.0,
                // len: 0.0,
                conf.l1.as_mm(),
                conf.l2.as_mm(),
                conf.l3.as_mm(),
                conf.l4.as_mm(),
                // dpt: Offset::new(0.0, 0.0),
                // gpt: Offset::new(0.0, 0.0),
            )).collect(),
            dbg: Dbg::new(parent, "Booms"),
        }
    }
    ///
    /// Evaluates Boom's values using passed new parameters
    pub fn eval(&mut self) -> Vec<Boom> {
        self.angles();
        self.boom_d_g_points();
        self.items.clone()
    }
    ///
    /// 2. Угол наклона к горизонту каждой стрелы (alpha_boom)
    fn angles(&mut self) {
        let mut alpha_sum = 0.0;
        for (i, boom) in self.items.iter_mut().enumerate() {
            alpha_sum += boom.alpha_rel;
            boom.alpha = alpha_sum - (i as f64) * 180.0;
            log::debug!("{}.angles | Boom[{i}] '{}':  absolute alpha: {}", self.dbg, boom.name, boom.alpha);
        }
    }
    ///
    /// 3. D и G для каждой стрелы
    fn boom_d_g_points(&mut self) {
        let prev = self.items.first().map(|boom| boom.clone());
        match prev {
            Some(mut prev) => {
                for (i, boom) in self.items.iter_mut().skip(1).enumerate() {
                    // Начало стрелы
                    let (x0, y0, alpha_prime) = if i == 0 {
                        (0.0, 0.0, 90.0)
                    } else {
                        (prev.gpt.x, prev.gpt.y, prev.alpha)
                    };
                    let Offset{x: wx, y: wy} = rotate_xy(boom.l4, boom.l3, alpha_prime);
                    let start = Offset::new(x0 + wx, y0 + wy);
                    // log::debug!(f"Стрела {i}: start={start}")
                    // Точка D
                    let Offset{x: dx, y: dy} = rotate_xy(- boom.l2, boom.l1, boom.alpha);
                    let dpt = Offset::new(start.x + dx, start.y + dy);
                    // log::debug!(f"\t dpt={dpt}")
                    // Точка G
                    let Offset{x: gx, y: gy} = rotate_xy(boom.len - boom.l2, boom.l1, boom.alpha);
                    let gpt = Offset::new(start.x + gx, start.y + gy);
                    // log::debug!(f"\t gpt={gpt}")
                    boom.dpt = dpt;
                    boom.gpt = gpt;
                    prev = boom.clone();
                }
            }
            _ => log::warn!("{}.boom_d_g_points | No Boom's found", self.dbg),
        }
    }
}