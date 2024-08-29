use std::{collections::HashMap, fmt::Debug};

use sal_sync::services::types::type_of::TypeOf;
///
/// 
trait Debug3: Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}
//
//
impl<T> Debug3 for Vec<T> where 
    T: Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ds = f.debug_struct(self.type_of());
        let dsr = self.iter().take(3).enumerate().fold(&mut ds, |ds, (index, item)| {
            ds.field(&index.to_string(), item)
        });
        dsr.finish()
    }
}
//
//
impl<T, U> Debug3 for HashMap<T, U> where 
    T: Debug, U: Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ds = f.debug_struct(self.type_of());
        let dsr = self.iter().take(3).fold(&mut ds, |ds, (index, item)| {
            ds.field(&format!("{:?}", index), item)
        });
        dsr.finish()
    }
}