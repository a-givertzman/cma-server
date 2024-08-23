use log::debug;
use crate::{core_::auth::{secret::auth_secret::AuthSecret, ssh::auth_ssh_path::AuthSshPath}, conf::conf_tree::ConfTree};
///
/// Jds-protocol specific kind of auturization on the TcpServer
#[derive(Debug, Clone, PartialEq)]
pub enum TcpServerAuth {
    None,
    Secret(AuthSecret),
    Ssh(AuthSshPath),
}
//
// 
impl TcpServerAuth {
    ///
    /// 
    pub fn new(value: ConfTree) -> Self {
        debug!("TcpServerAuth.new | value: {:?}", value);
        match value.key.to_lowercase().as_str() {
            "auth" => {
                match value.conf.as_str() {
                    Some("none" | "None") => TcpServerAuth::None,
                    Some(value) => panic!("TcpServerAuth.new | Unknown value '{}' in 'auth', 'none' or 'None' expected", value),
                    _ => panic!("TcpServerAuth.new | Unknown value type in 'auth', 'none' or 'None' expected"),
                }
            }
            "auth-secret" => {
                let token = match value.as_str("pass") {
                    Ok(token) => token,
                    Err(_) => panic!("TcpServerAuth.new | 'pass' - not found in 'auth-secret'"),
                };
                TcpServerAuth::Secret(AuthSecret::new(token))
            }
            "auth-ssh" => {
                let path = match value.as_str("path") {
                    Ok(path) => path,
                    Err(_) => panic!("TcpServerAuth.new | 'path' - not found in 'auth-ssh'"),
                };
                TcpServerAuth::Ssh(AuthSshPath::new(path))
            }
            _ => panic!("TcpServerAuth.new | Unknown auth: '{}'", value.key),
        }
    }
}
