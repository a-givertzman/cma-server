use serde::{Serialize, Deserialize};

///
/// General implementation of the PointConfig.address
/// For specific protocols can have custom implementations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PointConfigAddress {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bit: Option<u8>,
}
//
// 
impl PointConfigAddress {
    pub fn empty() -> Self {
        Self { offset: None, bit: None }
    }
}
