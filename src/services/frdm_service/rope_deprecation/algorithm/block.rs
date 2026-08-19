use std::{ops::Range, str::FromStr, sync::Arc};
use regex::Regex;
use sal_core::error::Error;
use crate::services::frdm_service::Offset;

///
/// Схема схода каната с блоком к следующему
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(usize)]
pub enum BlockScheme {
    /// Schema "1", Rope exits from top of the block, enters to the next on the top
    TopTop((f64, f64)) = 1,
    /// Schema "2", Rope exits from top of the block, enters to the next on the bottom
    TopBottom((f64, f64)) = 2,
    /// Schema "3", Rope exits from bottom of the block, enters to the next on the top
    BottomTop((f64, f64)) = 3,
    /// Schema "4", Rope exits from bottom of the block, enters to the next on the bottom
    BottomBottom((f64, f64)) = 4,
}
impl BlockScheme {
    // let (k, j) = match block.scheme {
    //     super::BlockScheme::TopTop => (-1.0, 1.0),
    //     super::BlockScheme::TopBottom => (1.0, 1.0),
    //     super::BlockScheme::BottomTop => (1.0, -1.0),
    //     super::BlockScheme::BottomBottom => (-1.0, -1.0),
    // };
    ///
    /// Returns tuple (k, j) - coefficients depends on rope transition kind between blocks
    pub fn kj(&self) -> (f64, f64) {
        match self {
            BlockScheme::TopTop(kj) => *kj,
            BlockScheme::TopBottom(kj) => *kj,
            BlockScheme::BottomTop(kj) => *kj,
            BlockScheme::BottomBottom(kj) => *kj,
        }
    }
    ///
    /// Schema "1", Rope exits from top of the block, enters to the next on the top
    #[allow(unused)]
    pub fn top_top() -> Self {
        Self::TopTop((-1.0, 1.0))
    }
    ///
    /// Schema "2", Rope exits from top of the block, enters to the next on the bottom
    #[allow(unused)]
    pub fn top_bottom() -> Self {
        Self::TopBottom((1.0, 1.0))
    }
    ///
    /// Schema "3", Rope exits from bottom of the block, enters to the next on the top
    #[allow(unused)]
    pub fn bottom_top() -> Self {
        Self::BottomTop((1.0, -1.0))
    }
    ///
    /// Schema "4", Rope exits from bottom of the block, enters to the next on the bottom
    #[allow(unused)]
    pub fn bottom_bottom() -> Self {
        Self::BottomBottom((-1.0, -1.0))
    }

}
impl FromStr for BlockScheme {
    type Err = Error;
    ///
    /// Retirns [BlockScheme] from str like `TopTop`, `TopBottom`, `BottomTop`, `BottomBottom`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TopTop" => Ok(Self::TopTop((-1.0, 1.0))),
            "TopBottom" => Ok(Self::TopBottom((1.0, 1.0))),
            "BottomTop" => Ok(Self::BottomTop((1.0, -1.0))),
            "BottomBottom" => Ok(Self::BottomBottom((-1.0, -1.0))),
            _ => Err(Error::new("BlockScheme", "from_str").err(format!("Unknown variant '{s}'"))),
        }
    }
}
///
/// Привязка блока стреле (нумерация с 0)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockBind {
    /// Барабан лебедки
    Drum,
    /// Неподвижный блок вне стрелы
    Fixed,
    /// Блок на стреле
    Boom(usize),
    // /// Блок на стреле, работает впаре, подразумевается что пара соседних блоков имеет такой тип
    // BoomPair(usize),
    /// Блок на подвесе (крюке)
    Hook,
}
//
//
impl BlockBind {
    ///
    /// Returns Boom from corresponding string
    fn boom(s: &str) -> Result<Self, Error> {
        let re = Regex::new(r"(boom|boompair)[ \t](\d+)").unwrap();
        let caps = re.captures(s)
            .ok_or(Error::new("BlockBind", "from_str").err(format!("Wrong format '{s}', Expected string like 'Boom 0'")))?;
        let kind = caps.get(1)
            .ok_or(Error::new("BlockBind", "from_str").err(format!("Wrong format '{s}', Expected string like 'Boom 0 / BoomPair 0'")))?;
        let bind = caps.get(2)
            .ok_or(Error::new("BlockBind", "from_str").err(format!("Wrong format '{s}', Expected string like 'Boom 0'")))?;
        let bind = bind.as_str().parse()
            .map_err(|_| Error::new("BlockBind", "from_str").err(format!("Wring Block number in '{s}', Expecting integer >= 0")))?;
        match kind.as_str() {
            "boom" => Ok(Self::Boom(bind)),
            // "boompair" => Ok(Self::BoomPair(bind)),
            _ => Err(Error::new("BlockBind", "from_str").err(format!("Wrong format '{s}', Expected string like 'Boom 0 / BoomPair 0'"))),
        }
    }
    ///
    /// Returns `true` if `self` and `other` has same kind
    #[allow(unused)]
    pub fn is(&self, other: Self) -> bool {
        match (self, other) {
            (BlockBind::Drum, BlockBind::Drum) => true,
            (BlockBind::Fixed, BlockBind::Fixed) => true,
            (BlockBind::Boom(_), BlockBind::Boom(_)) => true,
            // (BlockBind::BoomPair(_), BlockBind::BoomPair(_)) => true,
            (BlockBind::Hook, BlockBind::Hook) => true,
            _ => false,
        }
    }
}
impl FromStr for BlockBind {
    type Err = Error;
    ///
    /// Retirns [BlockBind] from str like `Fixed`, `Boom(0)`, `Hook`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase() {
            key if key == "drum" => Ok(Self::Drum),
            key if key == "fixed" => Ok(Self::Fixed),
            key if key.starts_with("boom") => Self::boom(&key),
            key if key == "hook" => Ok(Self::Hook),
            _ => Err(Error::new("BlockBind", "from_str").err(format!("Unknown variant '{s}'"))),
        }
    }
}
///
/// Crane Block
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub name: Arc<str>,
    /// Block position relative to boom G (end of boom).
    pub lf: Offset<f64>,
    /// Block diameter, mm.
    pub diameter: f64,
    /// Схема схода каната с блоком к следующему.
    pub scheme: BlockScheme,
    /// Привязка блока к стреле (нумерация с 0).
    pub bind: BlockBind,
    /// Координаты блока в ГСК.
    pub pos: Offset<f64>,
    /// Угол линии каната между текущим блоком и следующим к горизонту, градусы.
    pub rope_alpha_fwd: f64,
    /// Угол линии каната между текущим блоком и предыдущим к горизонту, градусы.
    pub rope_alpha_bck: f64,
    /// угол обхвата каната огибающего блок, градусы.
    pub wrap_alpha: f64,
    /// Длина каната огибающего блок, для барабана длина каната на барабане до точки схода, мм.
    pub wrap_length: f64,
    /// Разница длины каната на блоке по отношению к базовому (парковочному) положению, мм.
    pub wrap_delta: f64,
    /// Длина каната от точки схода с текущего блока до точки входа на следующий, мм.
    pub rope_len_fwd: f64,
    /// Длина каната от точки входа на текущий блок до точки схода с предыдущего, мм.
    pub rope_len_bck: f64,
    /// Текущие точки входа и схода каната с блока, считая от его начала каната.
    pub bending: Range<f64>,
    /// Угол перекидывания, град. Блок включается в работу только когда стрела проходит положение перекидывания.
    pub deflector: Option<f64>,
    /// Блок исключен из вычислений.
    pub skipped: bool,
}
//
//
impl Block {
    ///
    /// Returns [Block] new instance
    /// - `lF` - Растояние от **конца** стрелы до оси блока, мм
    /// - `D` - Диаметры блоков, мм
    /// - `schemes` - Схема схода каната на блоке
    /// - `boom` - К какой стреле относится блок (нумерация с 0)
    /// - `rope_alpha_fwd` - Угол линии каната между текущим блоком и следующим к горизонту, градусы
    /// - `rope_alpha_bck` - Угол линии каната между текущим блоком и предыдущим к горизонту, градусы
    /// - `wrap_alpha` - Угол обхвата каната огибающего блок, градусы
    /// - `wrap_length` - Длина дуги каната огибающего блок, мм
    /// - `rope_len_fwd` - Длина каната от точки схода с текущего блока до точки входа на следующий, мм
    /// - `rope_len_bck` - Длина каната от точки входа на текущий блок до точки схода с предыдущего, мм
    /// - `bending` - Текущие точки входа и схода каната с блока, считая от его начала каната.
    /// - `deflector` - Угол перекидывания, град. Блок включается в работу только когда стрела проходит положение перекидывания.
    pub fn new(
        name: impl Into<String>,
        lf: Offset<f64>,
        diameter: f64,
        scheme:BlockScheme,
        bind: BlockBind,
        deflector: Option<f64>,
    ) -> Self {
        Self {
            name: Arc::from(name.into()),
            lf,
            diameter,
            scheme,
            bind,
            pos: Offset::new(0.0, 0.0),
            rope_alpha_fwd: 0.0,
            rope_alpha_bck: 0.0,
            wrap_alpha: 0.0,
            wrap_length: 0.0,
            wrap_delta: 0.0,
            rope_len_fwd: 0.0,
            rope_len_bck: 0.0,
            bending: 0.0..0.0,
            deflector,
            skipped: false,
        }
    }
    ///
    /// Returns [Block] with specified `wrap_alpha`
    /// - `val` - Угол обхвата каната огибающего блок, градусы.
    #[allow(unused)]
    pub fn with_wrap_alpha(mut self, val: f64) -> Self {
        self.wrap_alpha = val;
        self
    }
    ///
    /// Returns [Block] with specified `wrap_length`
    /// - `val` - Длина каната огибающего блок, для барабана длина каната на барабане до точки схода, мм.
    #[allow(unused)]
    pub fn with_wrap_length(mut self, val: f64) -> Self {
        self.wrap_length = val;
        self
    }
    ///
    /// Returns [Block] with specified `wrap_delta`
    /// - `val` - Разница длины каната на блоке по отношению к базовому (парковочному) положению, мм.
    #[allow(unused)]
    pub fn with_wrap_delta(mut self, val: f64) -> Self {
        self.wrap_delta = val;
        self
    }
    ///
    /// Returns [Block] with specified `bending`
    /// - `val` - Текущие точки входа и схода каната с блока, считая от его начала каната.
    #[allow(unused)]
    pub fn with_bending(mut self, val: Range<f64>) -> Self {
        self.bending = val;
        self
    }
}
//
//
impl Default for Block {
    fn default() -> Self {
        Self {
            name: Default::default(),
            lf: Offset::new(0.0, 0.0),
            diameter: Default::default(),
            scheme: BlockScheme::top_top(),
            bind: BlockBind::Drum,
            pos: Offset::new(0.0, 0.0),
            rope_alpha_fwd: Default::default(),
            rope_alpha_bck: Default::default(),
            wrap_alpha: Default::default(),
            wrap_length: Default::default(),
            wrap_delta: Default::default(),
            rope_len_fwd: Default::default(),
            rope_len_bck: Default::default(),
            bending: Default::default(),
            deflector: Default::default(),
            skipped: Default::default(),
        }
    }
}
