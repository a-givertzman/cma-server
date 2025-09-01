use serde::Deserialize;

///
/// Database tables used for Rope Defect Detection
#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct TablesConf {
    pub defect: String,
    #[serde(rename = "defect-image")]
    pub defect_image: String,
}