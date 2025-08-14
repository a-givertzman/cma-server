use sal_core::dbg::Dbg;
use sal_sync::services::{conf::{ConfDistance, ConfTree}, entity::Name};
///
/// ## The configuration parameters for the crane's boom
/// 
/// ### Example:
/// ```yaml
/// boom:
///     l1: 0.0 mm                  # Растояние от продольной оси стрелы до точки A (оси ее поворота), константа
///     l2: 0.0 mm                  # Растояние по продольной оси стрелы от точки D (корня стрелы) до точки A (оси ее поворота), константа
///     l3: 0.0 mm                  # Расстояние от точки A (ось поворота) стрелы до продольной оси предыдущей стрелы (до ГСК для первой срелы), константа
///     l4: 10330.0 mm              # Расстояние от точки A (ось поворота) стрелы до перпендикуляра к продольной оси через точку G предыдущей стрелы (до ГСК для первой срелы), константа
///     len: 11200.0 mm             # Length of the boom
///     angle: point real 'App/MultiQueue/Load.MainBoomAngle'   # degrees, current angle of the boom (relative axis)
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct BoomConf {
    /// Растояние от продольной оси стрелы до точки A (оси ее поворота), константа
    pub l1: ConfDistance,
    /// Растояние по продольной оси стрелы от точки D (корня стрелы) до точки A (оси ее поворота), константа
    pub l2: ConfDistance,
    /// Расстояние от точки A (ось поворота) стрелы до продольной оси предыдущей стрелы (до ГСК для первой срелы), константа
    pub l3: ConfDistance,
    /// Расстояние от точки A (ось поворота) стрелы до перпендикуляра к продольной оси через точку G предыдущей стрелы (до ГСК для первой срелы), константа
    pub l4: ConfDistance,
    /// Length of the boom
    pub len: ConfDistance,
    /// Current angle of the boom (relative axis), degrees
    pub angle: String,
}
//
// 
impl BoomConf {
    ///
    /// Returns [BoomConf] built from `ConfTree`:
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let parent = parent.into();
        let me = "BoomConf";
        let dbg = Dbg::new(&parent, me);
        log::trace!("{}.new | conf: {:?}", dbg, conf);
        let name = Name::new(parent, me);
        log::trace!("{}.new | name: {:?}", dbg, name);
        let l1 = conf.get_distance("l1").expect(&format!("{dbg}.new | 'l1' - not found or wrong config"));
        let l2 = conf.get_distance("l2").expect(&format!("{dbg}.new | 'l2' - not found or wrong config"));
        let l3 = conf.get_distance("l3").expect(&format!("{dbg}.new | 'l3' - not found or wrong config"));
        let l4 = conf.get_distance("l4").expect(&format!("{dbg}.new | 'l4' - not found or wrong config"));
        let len = conf.get_distance("len").expect(&format!("{dbg}.new | 'len' - not found or wrong config"));
        let angle = conf.get_fn_config(&dbg, "angle", &mut vec![]).expect(&format!("{dbg}.new | 'angle' - not found or wrong config")).name();
        Self {
            l1,
            l2,
            l3,
            l4,
            len,
            angle,
        }
    }
}
