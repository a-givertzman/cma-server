use frdm_tools::GeometryDefectType;

/// 
/// A atomic part of a rope, used for rope deprecation rate calculation
pub struct RopeSlice {
    state: RopeSliceSate,
}
//
//
impl RopeSlice {
    pub fn new() -> Self {
        Self {
            state: RopeSliceSate::Ok,
        }
    }
}
///
/// State of the [RopeSlice]
enum RopeSliceSate {
    Defect(GeometryDefectType),
    Ok,
}