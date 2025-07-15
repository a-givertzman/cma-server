///
/// Holds simple string auth secret
#[derive(Debug, Clone, PartialEq)]
pub struct AuthSecret {
    token: String,
}
//
// 
impl AuthSecret {
    ///
    /// Creates new instance of AuthSecret
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_string(),
        }
    }
    ///
    /// Returns secret token
    pub fn token(&self) -> String {
        self.token.clone()
    }
}