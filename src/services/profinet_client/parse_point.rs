use chrono::{DateTime, Utc};
use sal_sync::services::entity::{Point, PointConfAddress, Status};
///
/// Returns updated points parsed from the data slice from the S7 device,
pub trait ParsePoint {
    ///
    /// Returns new point parsed from the data slice `bytes` with the given `timestamp` and Status::Ok
    fn next(&mut self, bytes: &[u8], ts: DateTime<Utc>) -> Option<Point>;
    ///
    /// Returns new point (prevously parsed) with the given [status] and `timestamp`
    fn next_status(&mut self, status: Status, ts: DateTime<Utc>) -> Option<Point>;
    ///
    /// Returns raw protocol specific address
    fn address(&self) -> PointConfAddress;
}
