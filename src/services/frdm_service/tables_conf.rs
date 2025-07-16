use serde::Deserialize;

///
/// Database tables used for FRDM
#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct TablesConf {
    pub defect: String,
    #[serde(rename="defect_image")]
    pub defect_image: String,
    pub deprication: String,
}