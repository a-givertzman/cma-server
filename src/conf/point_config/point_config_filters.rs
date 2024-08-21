use serde::{Serialize, Deserialize};
///
/// Set of the prefilters - executed during parsing data points from the protocol line
///     - [threshold] - float parameter for data points to be filtered
///     - [factor] - integral factor
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PointConfigFilter {
    // #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub factor: Option<f64>,
}
// ///
// /// 
// impl PointConfigAddress {
//     pub fn empty() -> Self {
//         Self { offset: None, bit: None }
//     }
// }
