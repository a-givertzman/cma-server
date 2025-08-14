use sal_core::dbg::Dbg;
use sal_sync::{collections::FxIndexMap, services::entity::Point};
use crate::services::frdm_service::{BoomConf, CraneConf, RopeSlice};

#[derive(Clone)]
struct Offset<T> {
    pub x: T,
    pub y: T,
}
impl<T> Offset<T> {
    pub fn new(x: T, y: T) -> Self {
        Self {
            x,
            y,
        }
    }
}
impl<T: std::fmt::Display> std::fmt::Display for Offset<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Offset({}, {})", self.x, self.y)
    }
}
impl<T: std::fmt::Display> std::fmt::Debug for Offset<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Offset({}, {})", self.x, self.y)
    }
}
///
/// Crane Boom
#[derive(Debug, Clone)]
pub struct Boom {
    // Углы наклона стрел (относительно предыдыдущей) в градусах
    alpha_rel: f64,
    // Углы наклона стрел (относительно ГСК) в градусах
    alpha: f64,
    len: f64,
    l1: f64,
    l2: f64,
    l3: f64,
    l4: f64,
    /// Boom D point - root of boom
    dpt: Offset<f64>,
    /// Boom G point - end of boom
    gpt: Offset<f64>,
}
impl Boom {
    ///
    /// Returns [Boom] new instance
    /// - `alpha` - Относительный угол наклона стрел (относительно предыдыдущей) в градусах
    /// - `len` - Длины стрел, мм
    /// - `l1` - Вертикальное смещение точки D, мм
    /// - `l2` - Горизонтальное смещение точки D, мм
    /// - `l3` - Вертикальное смещение начала стрелы относительно..., мм
    /// - `l4` - Горизонтальное смещение начала стрелы относительно..., мм
    pub fn new(l1: f64, l2: f64, l3: f64, l4: f64) -> Self {
        Self {
            alpha_rel: 0.0,
            alpha: 0.0,
            len: 0.0,
            l1,
            l2,
            l3,
            l4,
            dpt: Offset::new(0.0, 0.0),
            gpt: Offset::new(0.0, 0.0),
        }
    }
    // pub fn new(conf: &Vec<(String, BoomConf)>) -> FxIndexMap<String, Self> {
    //     conf.iter().map(|(key, conf)| (key.to_owned(), Self {
    //         alpha_rel: 0.0,
    //         alpha: 0.0,
    //         len: 0.0,
    //         l1: conf.l1.as_mm(),
    //         l2: conf.l2.as_mm(),
    //         l3: conf.l3.as_mm(),
    //         l4: conf.l4.as_mm(),
    //         dpt: Offset::new(0.0, 0.0),
    //         gpt: Offset::new(0.0, 0.0),
    //     })).collect()
    // }
}
///
/// Evaluation for the crane boom's collection
pub struct Booms {
    items: FxIndexMap<String, Boom>,
    dbg: Dbg,
}
impl Booms {
    ///
    /// Returns [Boom] new instance
    pub fn new(parent: impl Into<String>, conf: &Vec<(String, BoomConf)>) -> Self {
        Self {
            items: conf.iter().map(|(key, conf)| (key.to_owned(), Boom::new(
                // alpha_rel: 0.0,
                // alpha: 0.0,
                // len: 0.0,
                conf.l1.as_mm(),
                conf.l2.as_mm(),
                conf.l3.as_mm(),
                conf.l4.as_mm(),
                // dpt: Offset::new(0.0, 0.0),
                // gpt: Offset::new(0.0, 0.0),
            ))).collect(),
            dbg: Dbg::new(parent, "Booms"),
        }
    }
    ///
    /// Evaluates Boom's values using passed new parameters
    pub fn eval(&mut self) -> FxIndexMap<String, Boom> {
        self.angles();
        self.boom_d_g_points();
        self.items.clone()
    }
    ///
    /// 2. Угол наклона к горизонту каждой стрелы (alpha_boom)
    fn angles(&mut self) {
        let mut alpha_sum = 0.0;
        for (i, (_key, boom)) in self.items.iter_mut().enumerate() {
            alpha_sum += boom.alpha_rel;
            boom.alpha = alpha_sum - (i as f64) * 180.0;
            log::debug!("{}.angles | Boom[{i}] '{_key}':  absolute alpha: {}", self.dbg, boom.alpha);
        }
    }
    ///
    /// 3. D и G для каждой стрелы
    fn boom_d_g_points(&mut self) {
        let prev = self.items.first().map(|(_, boom)| boom.clone());
        match prev {
            Some(mut prev) => {
                for (i, (_key, boom)) in self.items.iter_mut().skip(1).enumerate() {
                    // Начало стрелы
                    let (x0, y0, alpha_prime) = if i == 0 {
                        (0.0, 0.0, 90.0)
                    } else {
                        (prev.gpt.x, prev.gpt.y, prev.alpha)
                    };
                    let Offset{x: wx, y: wy} = Self::rotate_xy(boom.l4, boom.l3, alpha_prime);
                    let start = Offset::new(x0 + wx, y0 + wy);
                    // log::debug!(f"Стрела {i}: start={start}")
                    // Точка D
                    let Offset{x: dx, y: dy} = Self::rotate_xy(- boom.l2, boom.l1, boom.alpha);
                    let dpt = Offset::new(start.x + dx, start.y + dy);
                    // log::debug!(f"\t dpt={dpt}")
                    // Точка G
                    let Offset{x: gx, y: gy} = Self::rotate_xy(boom.len - boom.l2, boom.l1, boom.alpha);
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
    ///
    /// 
    fn rotate_xy(lx: f64, ly: f64, alpha: f64) -> Offset<f64> {
        let angle_rad = alpha.to_radians();
        Offset::new(lx * angle_rad.cos() - ly * angle_rad.sin(), lx * angle_rad.sin() + ly * angle_rad.cos())
    }
}
///
/// Схема схода каната с блоком к следующему
pub enum BlockScheme {
    TopTop,
    TopBottom,
    BottomTop,
    BottomBottom,
}
///
/// Привязка блока стреле (нумерация с 0)
pub enum BlockBind {
    /// Блок вне стрелы, барабан
    BlockBindFixed,
    /// Блок на стреле
    BlockBindBoom(usize),
    /// Блок на подвесе (крюке)
    BlockBindHook,
}
/// 
/// Crane Block
pub struct Block {
    /// Block position relative to boom G (end of boom)
    lf: Offset<f64>,
    /// Block diameter
    d: f64,
    /// Схема схода каната с блоком к следующему
    scheme: BlockScheme,
    /// Привязка блока стреле (нумерация с 0)
    bind: BlockBind,
    /// Координаты блока в ГСК
    coord: Offset<f64>
}
impl Block {
    ///
    /// Returns [Block] new instance
    /// - `lF` - Растояние от **конца** стрелы до оси блока, мм
    /// - `D` - Диаметры блоков, мм
    /// - `schemes` - Схема схода каната на блоке
    /// - `boom` - К какой стреле относится блок (нумерация с 0)
    pub fn new(lf: Offset<f64>, D: f64, scheme:BlockScheme, bind: BlockBind) -> Self {
        Self {
            lf,
            d: D,
            scheme: scheme,
            bind: bind,
            coord: Offset::new(0.0, 0.0),
        }
    }
}

///
/// The collection of [RopeSlice]
/// - Devide rope by specified in the config number of slices
/// - Calculate deprecation for each slice
pub struct RopeSlices<'a> {
    inputs: FxIndexMap<String, f64>,
    booms: Booms,
    slices: Vec<RopeSlice>,
    conf: CraneConf,
    deprecation: Box<dyn Fn(usize, f64) + 'a>,
    dbg: Dbg,
}
//
//
impl<'a> RopeSlices<'a> {
    ///
    /// Returns [RopeSlices] new instance
    /// - `deprecation` - Here will be passed evaluated deprecation for each [RopeSlice] with it's index,
    pub fn new(parent: impl Into<String>, conf: CraneConf, deprecation: impl Fn(usize, f64) + 'a) -> Self {
        let dbg = Dbg::new(parent, "RopeSlices");
        let slices = (conf.rope.length.as_m() / conf.rope.segment.as_m()).ceil() as usize;
        log::debug!("{dbg}.new | Rope: {} m, slices: {slices}, devided by {:.2} mm", conf.rope.length.as_m(), conf.rope.segment.as_mm());
        Self {
            inputs: FxIndexMap::default(),
            booms: Booms::new(&dbg, &conf.booms),
            slices: (0..slices).map(|slice| {
                let offset = (slice as f64) * conf.rope.segment.as_m();
                log::trace!("{dbg}.new | Slice: {slice}: offset: {:.2}", offset);
                RopeSlice::new(slice, &conf.bendings, offset)
            }).collect(),
            conf,
            deprecation: Box::new(deprecation),
            dbg,
        }
    }
    ///
    /// ### Use this method to pass a new Event contains a value for the calculation
    /// - Expected boom len / angle, rope pos / load events, for example:
    ///     - [Load.MainBoomAngle], current angle of the boom (relative axis), degrees
    ///     - [Load.RotaryBoomLen], length of the rotary boom, meter
    ///     - [Winch.EncoderBR2], current rope position, meter
    ///     - [Winch.Load], current rope load, tonn
    /// - Event mast have proper name, defined in the configured inputs, else it will be ignored
    /// - Event mast have value in proper units:
    ///     - angle: degrees
    ///     - distances: millimeters
    ///     - weight: tonn
    /// - Event can have type (else it will be ignores):
    ///     - `Int`
    ///     - `Real`
    ///     - `Double`
    fn add(&mut self, event: &Point) {
        match self.inputs.get_mut(&event.name()) {
            Some(input) => {
                match event {
                    Point::Bool(_) => log::warn!("{}.new | Point '{}' - expected numeric type, but has 'Bool'", self.dbg, event.name()),
                    Point::Int(point) => *input = point.value as f64,
                    Point::Real(point) => *input = point.value as f64,
                    Point::Double(point) => *input = point.value,
                    Point::String(_) => log::warn!("{}.new | Point '{}' - expected numeric type, but has 'String'", self.dbg, event.name()),
                    Point::Bytes(_) => log::warn!("{}.new | Point '{}' - expected numeric type, but has 'Bytes'", self.dbg, event.name()),
                }
            }
            None => log::warn!("{}.new | Unexpected Point '{}'", self.dbg, event.name()),
        }
    }
    
    ///
    /// Evaluates rope slices deprication depend on boom len / angle, rope pos / load event was received,
    /// New deprecation result can be evaluated and passed via `deprication` callback
    pub fn eval(&mut self, event: &Point) {
        self.add(event);
        match point.name() {
            name if name == conf.crane.rope.pos => {
                let pos = point.to_double().as_double().value;
                log::debug!("{dbg}.run | Received rope pos: {:.4?} m", pos);
                rope_pos.store((pos * 1000.0).round() as usize, Ordering::SeqCst);
                rope_pos_ok.store(true, Ordering::SeqCst);
                rope_slices.eval(Some(pos), None);
            }
            name if name == conf.crane.rope.load => {
                let load = point.to_double().as_double().value;
                log::debug!("{dbg}.run | Received rope load: {:.4?} tonn", load);
                rope_slices.eval(None, Some(load));
            }
            // name if name == conf.crane.booms.main_angle => {
            //     let main_angle = point.to_double().as_double().value;
            //     log::debug!("{dbg}.run | Received boom.main_angle: {:.4?}", main_angle);
            // }
            // name if name == conf.crane.booms.rotary_angle => {
            //     let rotary_angle = point.to_double().as_double().value;
            //     log::debug!("{dbg}.run | Received boom.rotary_angle: {:.4?}", rotary_angle);
            // }
            _ => log::warn!("{dbg}.run | Unknown point name: {:?}", point.name()),
        }

        match (pos, load) {
            (None, None) => {},
            (None, Some(load)) => for slice in &mut self.slices { slice.add_load(load) },
            (Some(pos), None) => for slice in &mut self.slices { slice.add_pos(pos) },
            (Some(pos), Some(load)) => {
                for slice in &mut self.slices {
                    slice.add_pos(pos);
                    slice.add_load(load);
                }
            }
        }
        for slice in &mut self.slices {
            if let Some(deprecation) = slice.deprecation(&self.conf.bendings, pos, load) {
                (self.deprecation)(slice.id(), deprecation);
            }
        }
    }
}