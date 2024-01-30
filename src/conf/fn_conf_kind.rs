use super::{fn_config::FnConfig, point_config::point_config::PointConfig};

///
/// The type of the Function config, incapsulating config it self
#[derive(Debug, Clone, PartialEq)]
pub enum FnConfKind {
    Fn(FnConfig),
    Var(FnConfig),
    Const(FnConfig),
    Point(FnConfig),
    Param(String),
}
///
/// 
impl FnConfKind {
    ///
    /// Returns the name of the incapsulated config
    pub fn name(&self) -> String {
        match self {
            FnConfKind::Fn(conf) => conf.name.clone(),
            FnConfKind::Var(conf) => conf.name.clone(),
            FnConfKind::Const(conf) => conf.name.clone(),
            FnConfKind::Point(conf) => conf.name.clone(),
            FnConfKind::Param(conf) => conf.clone(),
        }
    }
    ///
    /// Returns list of configurations of the defined points of the incapsulated config
    pub fn points(&self) -> Vec<PointConfig> {
        match self {
            FnConfKind::Fn(conf) => conf.points(),
            FnConfKind::Var(conf) => conf.points(),
            FnConfKind::Const(conf) => conf.points(),
            FnConfKind::Point(conf) => conf.points(),
            FnConfKind::Param(conf) => panic!("FnConfKind.points | Param {} - does not have points", conf),
        }
    }
}