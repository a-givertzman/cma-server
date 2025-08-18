use std::str::FromStr;
use regex::Regex;
use sal_core::error::Error;
use crate::services::frdm_service::Offset;

///
/// Схема схода каната с блоком к следующему
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockScheme {
    TopTop,
    TopBottom,
    BottomTop,
    BottomBottom,
}
impl FromStr for BlockScheme {
    type Err = Error;
    ///
    /// Retirns [BlockScheme] from str like `TopTop`, `TopBottom`, `BottomTop`, `BottomBottom`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TopTop" => Ok(Self::TopTop),
            "TopBottom" => Ok(Self::TopBottom),
            "BottomTop" => Ok(Self::BottomTop),
            "BottomBottom" => Ok(Self::BottomBottom),
            _ => Err(Error::new("BlockScheme", "from_str").err(format!("Unknown variant '{s}'"))),
        }
    }
}
///
/// Привязка блока стреле (нумерация с 0)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockBind {
    /// Блок вне стрелы, барабан
    Fixed,
    /// Блок на стреле
    Boom(usize),
    /// Блок на подвесе (крюке)
    Hook,
}
impl FromStr for BlockBind {
    type Err = Error;
    ///
    /// Retirns [BlockBind] from str like `Fixed`, `Boom(0)`, `Hook`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase() {
            key if key == "fixed" => Ok(Self::Fixed),
            key if key.starts_with("boom") => {
                let re = Regex::new(r"Boom[ \t](\d+)").unwrap();
                let caps = re.captures(s)
                    .ok_or(Error::new("BlockBind", "from_str").err(format!("Wrong format '{s}', Expected string like 'Boom 0'")))?;
                let bind = caps.get(1)
                    .ok_or(Error::new("BlockBind", "from_str").err(format!("Wrong format '{s}', Expected string like 'Boom 0'")))?;
                let bind = bind.as_str().parse()
                    .map_err(|_| Error::new("BlockBind", "from_str").err(format!("Wring Block number in '{s}', Expecting integer >= 0")))?;
                Ok(Self::Boom(bind))
            }
            key if key == "hook" => Ok(Self::Hook),
            _ => Err(Error::new("BlockBind", "from_str").err(format!("Unknown variant '{s}'"))),
        }
    }
}
/// 
/// Crane Block
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    name: String,
    /// Block position relative to boom G (end of boom)
    pub lf: Offset<f64>,
    /// Block diameter
    pub d: f64,
    /// Схема схода каната с блоком к следующему
    scheme: BlockScheme,
    /// Привязка блока стреле (нумерация с 0)
    pub bind: BlockBind,
    /// Координаты блока в ГСК
    pub pos: Offset<f64>
}
impl Block {
    ///
    /// Returns [Block] new instance
    /// - `lF` - Растояние от **конца** стрелы до оси блока, мм
    /// - `D` - Диаметры блоков, мм
    /// - `schemes` - Схема схода каната на блоке
    /// - `boom` - К какой стреле относится блок (нумерация с 0)
    pub fn new(name: impl Into<String>, lf: Offset<f64>, d: f64, scheme:BlockScheme, bind: BlockBind) -> Self {
        Self {
            name: name.into(),
            lf,
            d,
            scheme: scheme,
            bind: bind,
            pos: Offset::new(0.0, 0.0),
        }
    }
}
