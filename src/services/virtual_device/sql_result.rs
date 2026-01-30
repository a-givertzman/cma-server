use sal_sync::services::conf::ConfDuration;
use serde::Deserialize;

///
/// Test Result being fetched from database
/// - by specified SQL
/// - after specified delay
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SqlResult {
    sql: String,
    delay: ConfDuration,
}