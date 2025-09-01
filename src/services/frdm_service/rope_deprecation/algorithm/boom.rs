use crate::services::frdm_service::{InputKind, Offset};

///
/// Crane Boom
#[derive(Debug, Clone)]
pub struct Boom {
    pub name: String,
    /// The name of the input `Point` contains current `alpha_rel` value
    pub alpha_input: Option<String>,
    /// The name of the input `Point` contains current `len` value
    pub len_input: Option<String>,
    // Углы наклона стрел (относительно предыдыдущей) в градусах
    pub alpha_rel: f64,
    // Углы наклона стрел (относительно ГСК) в градусах
    pub alpha: f64,
    pub len: f64,
    pub l1: f64,
    pub l2: f64,
    pub l3: f64,
    pub l4: f64,
    /// Boom D point - root of boom
    pub dpt: Offset<f64>,
    /// Boom G point - end of boom
    pub gpt: Offset<f64>,
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
    pub fn new(name: impl Into<String>, alpha_input: InputKind<f64>, len_input: InputKind<f64>, l1: f64, l2: f64, l3: f64, l4: f64) -> Self {
        Self {
            name: name.into(),
            alpha_input: match &alpha_input {
                InputKind::Const(_) => None,
                InputKind::Point(val) => Some(val.clone()),
            },
            len_input: match &len_input {
                InputKind::Const(_) => None,
                InputKind::Point(val) => Some(val.clone()),
            },
            alpha_rel: match &alpha_input {
                InputKind::Const(val) => *val,
                InputKind::Point(_) => 0.0,
            },
            alpha: 0.0,
            len: match len_input {
                InputKind::Const(val) => val,
                InputKind::Point(_) => 0.0,
            },
            l1,
            l2,
            l3,
            l4,
            dpt: Offset::new(0.0, 0.0),
            gpt: Offset::new(0.0, 0.0),
        }
    }
}
