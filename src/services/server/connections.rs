use std::{any::Any, collections::HashMap, net::TcpStream, sync::mpsc::{SendError, Sender}, thread::JoinHandle};
use log::{error, info};
use testing::stuff::wait::WaitTread;
///
/// 
pub enum Action {
    Continue(TcpStream),
    Exit,
}
///
/// Keep TCP Server's connection's:
/// - thread JoinHandle
/// - Sender<Action>
#[derive(Debug)]
struct Connection {
    handle: JoinHandle<()>,
    send: Sender<Action>,
}
//
// 
impl Connection {
    pub fn new(handle: JoinHandle<()>, send: Sender<Action>,) -> Self {
        Self {
            handle,
            send,
        }
    }
    ///
    /// 
    pub fn send(&self, action: Action) -> Result<(), SendError<Action>> {
        self.send.send(action)
    }
    ///
    /// 
    pub fn wait(self) -> Result<(), Box<dyn Any + Send>> {
        self.handle.wait()
    }
    ///
    /// 
    pub fn is_active(&self) -> bool {
        !self.handle.is_finished()
    }
    ///
    /// 
    pub fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }
}



///
/// Contains a map of Connection's
#[derive(Debug)]
pub struct TcpServerConnections {
    id: String,
    connections: HashMap<String, Connection>,
}
//
// 
impl TcpServerConnections {
    ///
    /// 
    pub fn new(parent: impl Into<String>) -> Self {
        Self { 
            id: format!("{}/TcpServerConnections", parent.into()),
            connections: HashMap::new(),
        }
    }
    ///
    /// Inserts a new connection, if connection_id olready exists, connection will be updated
    pub fn insert(&mut self, connection_id: &str, handle: JoinHandle<()>, send: Sender<Action>) {
        info!("{}.insert | connection: '{}'", self.id, connection_id);
        self.connections.insert(
            connection_id.to_string(),
            Connection::new(
                handle,
                send,
            )
        );
    }
    ///
    /// Returns a TcpServerConnection if exists
    pub fn repair(&self, connection_id: &str, stream: TcpStream) -> Result<(), String> {
        match self.connections.get(connection_id) {
            Some(conn) => {
                if conn.is_active() {
                    match conn.send(Action::Continue(stream)) {
                        Ok(_) => {
                            // info!("{}.run | Keeped connection '{}' repaired", self_id, connectionId);
                            Ok(())
                        }
                        Err(err) => {
                            Err(format!("{}.repair | Keeped connection repair error {:?}", self.id, err))
                        }
                    }
                } else {
                    Err(format!("{}.repair | Keeped connection '{}' - exceeded", self.id, connection_id))
                }
            }
            None => {
                Err(format!("{}.repair | Keeped connection '{}' - not found", self.id, connection_id))
            }
        }
    }    
    ///
    /// Waits for all connections handles being finished
    pub fn wait(&mut self) {
        while !self.connections.is_empty() {
            let keys: Vec<String> = self.connections.keys().map(|k| {k.to_string()}).collect();
            info!("{}.run | Wait for connections:", self.id);
            for key in &keys {
                info!("{}.run | \tconnection: {:?}\t isActive: {}", self.id, key, self.connections.get(key).unwrap().is_active());
            }
            match keys.first() {
                Some(key) => {
                    let connection = self.connections.remove(key).unwrap();
                    connection.send(Action::Exit).unwrap_or_else(|_| {info!("{}.run | Connection '{}' - already finished", self.id, key)});
                    connection.wait().unwrap();
                }
                None => {
                    break;
                }
            };
        }
    }
    ///
    /// Chech if finished connection threads are present in the self.connection
    /// - removes finished connections
    pub fn clean(&mut self) {
        let mut to_remove = vec![];
        info!("{}.clean | Cleaning connections...", self.id);
        for (name, connection) in &self.connections {
            info!("{}.clean | Checking connection '{}' \t '{}' - finished: {}", self.id, name, connection.handle.thread().name().unwrap_or("unnamed"), connection.is_finished());
            if connection.is_finished() {
                to_remove.push(name.clone());
            }
        }
        info!("{}.clean | Finished connections found: {:#?}", self.id, to_remove);
        for name in to_remove {
            match self.connections.remove(&name) {
                Some(connection) => {
                    match connection.handle.wait() {
                        Ok(_) => info!("{}.clean | Connection '{}' removed successful", self.id, name),
                        Err(err) => error!("{}.clean | Connection '{}' wait error: {:#?}", self.id, name, err),
                    }
                }
                None => error!("{}.clean | Connection '{}':", self.id, name),
            }
        }
        info!("{}.clean | Cleaning connections - ok", self.id);
    }
}
