use crate::services::SqlResult;

///
/// Test Results can be fetched by SQL or received from configured Events
#[derive(Debug, Clone, PartialEq)]
pub enum ResultKind {
    Event(sal_sync::services::entity::PointConf),
    Sql(SqlResult)
}
