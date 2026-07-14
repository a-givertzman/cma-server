use sal_sync::services::{conf::ConfDuration, entity::PointType};
use serde::Deserialize;

///
/// ## Test Result being fetched from database
/// - by specified `sql`
/// - after specified `delay`
/// - `name` - required for writing result value to the table
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SqlResult {
    pub name: String,
    pub typ: PointType,
    pub sql: String,
    pub delay: ConfDuration,
}