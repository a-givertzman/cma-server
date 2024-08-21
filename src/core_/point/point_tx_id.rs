#![allow(non_snake_case)]

use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}};

///
/// Creates hash from string for Point.txId
pub struct PointTxId {}
//
// 
impl PointTxId {
    /// 
    /// Returns hash from string for Point.txId
    pub fn from_str(id: &str) -> usize {
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
        hasher.finish().try_into().unwrap()
    }
}
