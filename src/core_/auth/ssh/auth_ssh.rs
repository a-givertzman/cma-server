///
/// Represents path to the ssh.pub keys folder
#[derive(Debug, Clone, PartialEq)]
pub struct AuthSsh {
    path: String,
}
//
// 
impl AuthSsh {
    ///
    /// Creates new instance of AuthSsh
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_owned(),
        }
    }
    ///
    /// Returns Ok if SSH auth success
    #[allow(unused)]
    pub fn validate(&self, secret: &str) -> Result<(), String> {
        Err("AuthSsh.validate | Not implemented yet".to_owned())
    }
}