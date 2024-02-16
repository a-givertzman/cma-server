use serde::{Serialize, Deserialize};

///
/// The point config history option
///     - None - history parameter was omitted / deactivated
///     - Read - history parameter active for points coming from devicec to the clients
///     - Write - history parameter active for points (commands) coming from clients to the devices
/// ```
///     point Point.Name: 
///         type: 'Real'
///         offset: 8
///         history: None / Read / Write
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PointConfigHistory {
    #[serde(alias = "none", alias = "")]
    None,
    #[serde(rename(serialize = "r"))]
    #[serde(alias = "read", alias = "r")]
    Read,
    #[serde(rename(serialize = "w"))]
    #[serde(alias = "write", alias = "w")]
    Write,
    #[serde(rename(serialize = "rw"))]
    #[serde(alias = "readwrite", alias = "ReadWrite", alias = "rw")]
    ReadWrite
}

impl Default for PointConfigHistory {
    fn default() -> Self {
        Self::None
    }
}