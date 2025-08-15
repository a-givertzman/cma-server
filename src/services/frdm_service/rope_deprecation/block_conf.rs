use std::str::FromStr;
use regex::Regex;
use sal_core::dbg::Dbg;
use sal_sync::services::{conf::{ConfDistance, ConfTree, ConfTreeGet}, entity::Name};
use crate::services::frdm_service::{BlockBind, BlockScheme, Offset};

///
/// ## The configuration parameters for the crane's block
/// 
/// ### Example:
/// ```yaml
/// block:
///     lf: 1830.0 mm, 710.0 mm     # Растояние (x, y) от **конца** стрелы до оси блока, мм
///     D: 844.0 mm                 # Диаметры блоков, мм
///     schemes: TopTop             # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
///     bind: Boom 1                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct BlockConf {
    /// Block position relative to boom G (end of boom)
    pub lf: Offset<ConfDistance>,
    /// Block diameter
    pub d: ConfDistance,
    /// Схема схода каната с блоком к следующему
    pub scheme: BlockScheme,
    /// Привязка блока стреле (нумерация с 0)
    pub bind: BlockBind,
}
//
// 
impl BlockConf {
    ///
    /// Returns [BlockConf] built from `ConfTree`:
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let parent = parent.into();
        let me = "BlockConf";
        let dbg = Dbg::new(&parent, me);
        log::trace!("{}.new | conf: {:?}", dbg, conf);
        let name = Name::new(parent, me);
        log::trace!("{}.new | name: {:?}", dbg, name);
        let lf: String = conf.get("lf").expect(&format!("{dbg}.new | 'lf' - not found or wrong config"));
        let re = Regex::new("(.+),[ \t](.+)").unwrap();
        let lf_caps = re.captures(&lf).expect(&format!("{dbg}.new | 'lf' - not found or wrong config"));
        let lfx = ConfDistance::from_str(lf_caps.get(1).expect(&format!("{dbg}.new | 'lf.x' - wrong config")).as_str())
            .expect(&format!("{dbg}.new | 'lf.x' - wrong config"));
        let lfy = ConfDistance::from_str(lf_caps.get(2).expect(&format!("{dbg}.new | 'lf.y' - wrong config")).as_str())
            .expect(&format!("{dbg}.new | 'lf.y' - wrong config"));
        let d = conf.get_distance("d").expect(&format!("{dbg}.new | 'd' - not found or wrong config"));
        let scheme: String = conf.get("scheme").expect(&format!("{dbg}.new | 'scheme' - not found or wrong config"));
        let scheme = BlockScheme::from_str(&scheme).expect(&format!("{dbg}.new | 'scheme' - wrong config"));
        let bind: String = conf.get("bind").expect(&format!("{dbg}.new | 'bind' - not found or wrong config"));
        let bind = BlockBind::from_str(&bind).expect(&format!("{dbg}.new | 'bind' - wrong config"));
        Self {
            lf: Offset::new(lfx, lfy),
            d,
            scheme,
            bind,
        }
    }
}
