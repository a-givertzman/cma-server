use frdm_tools::GeometryDefectType;
use sal_sync::services::entity::Point;

/// 
/// A atomic part of a rope, used for rope deprecation rate calculation
pub struct RopeSlice {
    state: RopeSliceSate,
    pos: Option<f64>,
    load: Option<f64>,
    changed: Option<()>,
}
//
//
impl RopeSlice {
    pub fn new() -> Self {
        Self {
            state: RopeSliceSate::Ok,
            pos: None,
            load: None,
            changed: None,
        }
    }
    ///
    /// Registering new `pos` value, 
    /// So new deprication result can be evaluated
    pub fn add_pos(&mut self, val: Point) {
        self.pos = Some(val.to_double().as_double().value);
        self.changed = Some(());
    }
    ///
    /// Registering new `load` value, 
    /// So new deprication result can be evaluated
    pub fn add_load(&mut self, val: Point) {
        self.pos = Some(val.to_double().as_double().value);
        self.changed = Some(());
    }
    ///
    /// Evaluates [RopeSlice] deprication, returns `Some` if was added `pos` or `load`
    /// - can be evaluated only once per new `pos` or `load`, else returns `None`
    pub fn deprication(&mut self) -> Option<f64> {
        match self.changed {
            Some(_) => {
                self.changed = None;
                match (self.pos, self.load) {
                    (None, None) => None,
                    (None, Some(_)) => None,
                    (Some(_), None) => None,
                    (Some(pos), Some(load)) => todo!(),
                }
            },
            None => None,
        }
    }
}
///
/// State of the [RopeSlice]
enum RopeSliceSate {
    Defect(GeometryDefectType),
    Ok,
}