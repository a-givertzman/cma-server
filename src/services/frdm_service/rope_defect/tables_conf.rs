use serde::Deserialize;

///
/// Database tables used for Rope Defect Detection
#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct TablesConf {
    pub defect: String,
    #[serde(rename = "defect-image")]
    pub defect_image: String,
}
//
//
impl Default for TablesConf {
    fn default() -> Self {
        Self {
            defect: Default::default(),
            defect_image: Default::default(),
        }
    }
}