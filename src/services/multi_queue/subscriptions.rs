use log::{warn, trace};
use std::{collections::HashMap, fmt::Debug, hash::BuildHasherDefault, sync::mpsc::Sender};
use hashers::fx_hash::FxHasher;
use crate::core_::{point::point::Point, types::map::HashMapFxHasher};
///
/// Unique id of the service receiving the Point's by the subscription
/// This id used to identify the service produced the Points. 
/// To avoid send back self produced Point's.
type ReceiverId = usize;
///
/// Destination of the point,
/// Currently it's just a concat of the Point.cot & Point.id 
type PointDest = String; 
///
/// Contains map of Sender's
/// - Where Sender - is pair of String ID & Sender<PointType>
#[derive(Clone)]
pub struct Subscriptions {
    id: String,
    multicast: HashMapFxHasher<PointDest, HashMapFxHasher<ReceiverId, Sender<Point>>>,
    broadcast: HashMapFxHasher<ReceiverId, Sender<Point>>,
    empty: HashMapFxHasher<ReceiverId, Sender<Point>>,
}
//
// 
impl Subscriptions {
    ///
    /// Creates new instance of Subscriptions
    pub fn new(parent: impl Into<String>, ) -> Self {
        Self {
            id: format!("{}/Subscriptions", parent.into()),
            multicast: HashMap::with_hasher(BuildHasherDefault::<FxHasher>::default()),
            broadcast: HashMap::with_hasher(BuildHasherDefault::<FxHasher>::default()),
            empty: HashMap::with_hasher(BuildHasherDefault::<FxHasher>::default()),
        }
    }
    ///
    /// Adds subscription for receiver_id with destination 
    pub fn add_multicast(&mut self, receiver_id: usize, destination: &str, sender: Sender<Point>) {
        self.multicast
            .entry(destination.to_owned())
            .or_insert(HashMap::with_hasher(BuildHasherDefault::<FxHasher>::default()))
            .insert(receiver_id, sender);
    }
    ///
    /// Extends subscription if exists, otherwise returns error
    pub fn extend_multicast(&mut self, receiver_id: usize, destination: &str) -> Result<(), String> {
        match self.multicast.iter().find_map(|(_, senders)| senders.get(&receiver_id)) {
            Some(sender) => {
                self.add_multicast(receiver_id, destination, sender.clone());
                Ok(())
            }
            None => {
                let message = format!("{}.extend_multicast | Receiver '{}' - not found in subscriptions", self.id, receiver_id);
                warn!("{}", message);
                Err(message)
            }
        }
    }
    ///
    /// Adds subscription for receiver_id without destination, all destinations will be received
    pub fn add_broadcast(&mut self, receiver_id: usize, sender: Sender<Point>) {
        self.broadcast.insert(
            receiver_id,
            sender,
        );
    }
    ///
    /// Returns map of Senders
    pub fn iter(&self, point_id: &str) -> impl Iterator<Item = (&usize, &Sender<Point>)> {
        match self.multicast.get(point_id) {
            Some(multicast) => {
                trace!("{}.iter | \n\t Multicast: {:?} \n\t Broadcast: {:?}", self.id, multicast, self.broadcast);
                multicast.iter().chain(&self.broadcast)
            }
            None => {
                trace!("{}.iter | \n\t Broadcast: {:?}", self.id, self.broadcast);
                self.broadcast.iter().chain(&self.empty)
            }
        }
    }
    ///
    /// Removes single subscription by Point Id for receiver ID
    pub fn remove(&mut self, receiver_id: &usize, point_id: &str) -> Result<(), String> {
        match self.multicast.get_mut(point_id) {
            Some(senders) => {
                match senders.remove(receiver_id) {
                    Some(_) => Ok(()),
                    None => Err(format!("{}.run | Subscription '{}', receiver '{}' - not found", self.id, point_id, receiver_id)),
                }
            }
            None => Err(format!("{}.run | Subscription '{}' - not found", self.id, point_id)),
        }
    }
    ///
    /// Removes all subscriptions for receiver ID
    pub fn remove_all(&mut self, receiver_id: &usize) -> Result<(), String> {
        let mut changed = false;
        let mut messages = vec![];
        let keys: Vec<String> = self.multicast.keys().cloned().collect();
        for point_id in keys {
            match self.multicast.get_mut(&point_id) {
                Some(senders) => {
                    match senders.remove(receiver_id) {
                        Some(_) => {
                            changed |= true;
                        }
                        None => {
                            messages.push(format!("{}.run | Multicast Subscription '{}', receiver '{}' - not found", self.id, point_id, receiver_id));
                        }
                    }
                }
                None => {
                    messages.push(format!("{}.run | Multicast Subscription '{}' - not found", self.id, point_id));
                }
            }
        }
        match self.broadcast.remove(receiver_id) {
            Some(_) => {
                changed |= true;
            }
            None => {
                messages.push(format!("{}.run | Broadcast Subscription by receiver '{}' - not found", self.id, receiver_id));
            }
        }
        if changed {
            Ok(())
        } else {
            Err(messages.join("\n"))
        }
    }
    ///
    /// Removes all subscriptions
    pub fn exit(&mut self) {
        self.broadcast.clear();
        self.multicast.clear();
    }
}
//
// 
impl Debug for Subscriptions {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Subscriptions")
            .field("id", &self.id)
            .finish()
    }
}
