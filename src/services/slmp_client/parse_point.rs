use chrono::{DateTime, Utc};
use sal_sync::services::entity::{point::{point::Point, point_config_address::PointConfigAddress, point_config_type::PointConfigType}, status::status::Status};
///
/// Returns updated points parsed from the data slice from the S7 device,
pub trait ParsePoint: Send {
    ///
    /// Returns the type of the configured point
    fn type_(&self) -> PointConfigType;
    ///
    /// Returns new point parsed from the data slice [bytes] with current timestamp and Status::Ok
    fn next_simple(&mut self, bytes: &[u8]) -> Option<Point>;
    ///
    /// Returns new point parsed from the data slice [bytes] with the given [timestamp] and Status::Ok
    fn next(&mut self, bytes: &[u8], timestamp: DateTime<Utc>) -> Option<Point>;
    ///
    /// Returns new point (prevously parsed) with the given [status]
    fn next_status(&mut self, status: Status) -> Option<Point>;
    ///
    /// Returns true if value or status was updated since last call [addRaw()]
    fn is_changed(&self) -> bool;
    ///
    /// Returns raw protocol specific address
    fn address(&self) -> PointConfigAddress;
    ///
    /// Returns size of the type in the bytes
    fn size(&self) -> usize;
    ///
    /// Returns protocol specific bytes ready to write represents [value]
    fn to_bytes(&self, point: &Point) -> Result<Vec<u8>, String>;
}
