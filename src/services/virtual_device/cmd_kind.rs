
///
/// Commands | Can be executed before test started or after the test finished
#[derive(Debug, Clone, PartialEq)]
pub enum CmdKind {
    Sql(String)
}
