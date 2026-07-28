use serde::Deserialize;

/// Tables used for storing diagnostic risults into database
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Tables {
    pub faults: String,
    pub trends: String,
}
