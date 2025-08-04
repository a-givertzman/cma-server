use serde::{Deserialize, Serialize};
///
/// The API configuration parameters 
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiClientConf {
    pub address: String,
    pub auth_token: String,
    pub database: String,
}
//
//
impl ApiClientConf {
    ///
    /// 
    pub fn new(address: impl Into<String>, auth_token: impl Into<String>, database: impl Into<String>) -> Self {
        Self { 
            address: address.into(),
            auth_token: auth_token.into(),
            database: database.into()
        }
    }
}
//
//
impl Default for ApiClientConf {
    ///
    /// **Returns `ApiClientConf` with the default walues**
    /// 
    /// ```ignore
    /// ApiClientConf {
    ///    address: "0.0.0.0:8080",
    ///    auth_token: "123!@#",
    ///    database: "cma",
    /// }
    /// ```
    fn default() -> Self {
        Self {
            address: "0.0.0.0:8080".to_owned(),
            auth_token: "123!@#".to_owned(),
            database: "cma".to_owned(),
        }
    }
}
