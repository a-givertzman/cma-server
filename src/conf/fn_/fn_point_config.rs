use sal_sync::services::entity::point::point_config::PointConfig;
use crate::conf::fn_::fn_conf_kind::FnConfKind;
///
/// Represents configuration of the point in the NestedFn
///  - send-to - Service.Queue where the point will be sent
///  - input - the source of the point value  
///  - enable: const bool true                 # Optional, default true
///  - changes-only: const bool false          # Optional, default false
#[derive(Debug, PartialEq, Clone)]
pub struct FnPointConfig {
    pub conf: PointConfig,
    pub send_to: Option<String>,
    pub enable: Option<Box<FnConfKind>>,
    pub input: Option<Box<FnConfKind>>,
    pub changes_only: Option<Box<FnConfKind>>,
}
//
// 
impl FnPointConfig {
    ///
    /// Returns list of configurations of the defined points
    pub fn points(&self) -> Vec<PointConfig> {
        match &self.input {
            Some(input) => {
                let mut points = input.points();
                points.push(self.conf.clone());
                points
            },
            None => vec![self.conf.clone()],
        }
    }
}