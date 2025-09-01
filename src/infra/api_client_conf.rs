use std::time::Duration;
use sal_core::dbg::Dbg;
use sal_sync::services::{conf::{ConfTree, ConfTreeGet}, entity::Name};
///
/// The API configuration parameters 
#[derive(Debug, Clone, PartialEq)]
pub struct ApiClientConf {
    pub name: Name,
    pub wait_started: Option<Duration>,
    pub address: String,
    pub auth_token: String,
    pub database: String,
}
//
//
impl ApiClientConf {
    ///
    /// 
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let name = Name::new(parent, "ApiClient");
        let dbg = Dbg::new(name.parent(), "ApiClientConf");
        let wait_started: Option<Duration> = conf.get_duration("wait-started").ok();
        let address: String = conf.get("address").expect(&format!("{dbg}.new | 'address' - not found or wrong configuration"));
        let auth_token: String = conf.get("auth-token").expect(&format!("{dbg}.new | 'auth-token' - not found or wrong configuration"));
        let database: String = conf.get("database").expect(&format!("{dbg}.new | 'database' - not found or wrong configuration"));
        Self {
            name,
            wait_started,
            address,
            auth_token,
            database,
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
    ///    wait-started: Some(10 ms),
    ///    address: "0.0.0.0:8080",
    ///    auth-token: "123!@#",
    ///    database: "cma",
    /// }
    /// ```
    fn default() -> Self {
        Self {
            name: Name::new("ApiClientConf", ""),
            wait_started: Some(Duration::from_millis(10)),
            address: "0.0.0.0:8080".to_owned(),
            auth_token: "123!@#".to_owned(),
            database: "cma".to_owned(),
        }
    }
}
