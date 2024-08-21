use std::{collections::{hash_map::IntoIter, HashMap}, thread::JoinHandle};

///
/// Holds Services's id & JoinHandle pairs
pub struct ServiceHandles {
    // id: String,
    handles: HashMap<String, JoinHandle<()>>
}
//
// 
impl ServiceHandles {
    ///
    /// 
    pub fn new(handles: Vec<(String, JoinHandle<()>)>) -> Self {
        Self {
            handles: handles.into_iter().collect()
        }
    }
    ///
    /// Returns the nimber of the holding handles
    pub fn len(&self) -> usize {
        self.handles.len()
    }
    ///
    /// inserts new Services's id & JoinHandle 
    /// - if already have such id, current handle will be updated
    pub fn insert(&mut self, id: &str, handle: JoinHandle<()>) {
        self.handles.insert(id.to_owned(), handle);
    }
    // ///
    // /// 
    // pub fn first(&mut self) -> (String, JoinHandle<()>) {
    //     let key = self.handles.keys().next().cloned();
    //     match key {
    //         Some(key) => {
    //             match self.handles.remove_entry(&key) {
    //                 Some(handle) => handle,
    //                 None => {
    //                     panic!("ServiceHandle.first | Handle '{}' - not found", key)
    //                 }
    //             }
    //         }
    //         None => {
    //             panic!("ServiceHandle.first | Handles not found")
    //         }
    //     }
    // }
}
//
// 
impl IntoIterator for ServiceHandles {
    type Item = (String, JoinHandle<()>);

    type IntoIter = IntoIter<String, JoinHandle<()>>;

    fn into_iter(self) -> Self::IntoIter {
        // todo!()
        self.handles.into_iter()
    }
}