use chrono::{DateTime, Utc};
use sal_core::error::Error;
use sal_sync::services::entity::{{Point, PointType}, Status};
///
/// Returns updated points parsed from the data slice from the S7 device,
pub trait ParsePoint: Send {
    ///
    /// Returns the type of the configured point
    fn typ(&self) -> PointType;
    ///
    /// Adding new raw data to be parsed 
    fn add(&mut self, bytes: &[u8], status: Status, timestamp: DateTime<Utc>) -> Result<Vec<Point>, Error> ;
    ///
    /// Returns raw protocol specific address
    fn name(&self) -> String;
    ///
    /// Returns size of the type in the bytes
    fn size(&self) -> usize;
    ///
    /// Returns protocol specific bytes ready to write represents [value]
    fn to_bytes(&self, point: &Point) -> Result<Vec<u8>, String>;
}
