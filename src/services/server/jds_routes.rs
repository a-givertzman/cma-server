use std::{fmt::Debug, io::BufReader, net::TcpStream, sync::{mpsc::Sender, Arc, RwLock}};
use log::{error, warn, LevelFilter};
use crate::{
    conf::point_config::name::Name, core_::{
        net::{connection_status::ConnectionStatus, protocols::jds::jds_deserialize::JdsDeserialize}, 
        object::object::Object, 
        point::point::Point,
    }, 
    services::{server::jds_cnnection::Shared, services::Services},
    tcp::{steam_read::TcpStreamRead, tcp_stream_write::OpResult},
};
use concat_string::concat_string;
///
/// Container used in the JdsRoutes, contains of:
///     - pass - Point to be transmitted to the MultiQueue - returned from read method
///     - reply - Point to be sent back to the Client, contains reply in the value
#[derive(Debug)]
pub struct RouterReply {
    pass: Option<Point>,
    retply: Option<Point>,
}
impl RouterReply {
    pub fn new(pass: Option<Point>, retply: Option<Point>) -> Self {
        Self { pass, retply }
    }
}
///
/// Reaction on the JDS request. 
/// Implements behavior for different kind of JDS requests
/// - Incoming points reads from [jds_deserialize]
/// - Incoming points passed as a parameter into the [routes(parent, point, services)] calback function
/// - [routes] calback function returns RouterReply, which contains of:
///     - pass - Point to be transmitted to the MultiQueue - returned from read method
///     - reply - Point to be sent back to the Client, contains reply in the value
pub struct JdsRoutes<F> {
    parent_id: String,
    id: String,
    name: Name,
    services: Arc<RwLock<Services>>,
    jds_deserialize: JdsDeserialize,
    req_reply_send: Sender<Point>,
    rautes: F,
    shared: Arc<RwLock<Shared>>,
}
//
// 
impl<F> JdsRoutes<F> {
    ///
    /// 
    pub fn new(
        parent_id: &str,
        parent: &Name, 
        services: Arc<RwLock<Services>>, 
        jds_deserialize: JdsDeserialize, 
        req_reply_send: Sender<Point>, 
        rautes: F, 
        shared: Arc<RwLock<Shared>>,
    ) -> Self {
        Self {
            parent_id: parent_id.to_owned(),
            id: format!("{}/JdsRoutes", parent_id), 
            name: parent.clone(),
            services,
            jds_deserialize,
            req_reply_send,
            rautes,
            shared,
        }
    }
}
impl<F> Debug for JdsRoutes<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JdsRoutes")
            .field("id", &self.id)
            .finish()
    }
}
//
// 
impl<F> Object for JdsRoutes<F> {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl<F> TcpStreamRead for JdsRoutes<F> where
    //    parent_id, name
    F: Fn(String, Name, Point, Arc<RwLock<Services>>, Arc<RwLock<Shared>>) -> RouterReply,
    F: Send + Sync {
    ///
    /// Reads single point from source
    fn read(&mut self, tcp_stream: &mut BufReader<TcpStream>) -> ConnectionStatus<OpResult<Point, String>, String> {
        match self.jds_deserialize.read(tcp_stream) {
            ConnectionStatus::Active(point) => {
                match point {
                    OpResult::Ok(point) => {
                        let result = (self.rautes)(self.parent_id.clone(), self.name.clone(), point, self.services.clone(), self.shared.clone());
                        if let Some(point) = result.retply {
                            if let Err(err) = self.req_reply_send.send(point) {
                                error!("{}.read | Send reply error: {:?}", self.id, err)
                            }
                        };
                        match result.pass {
                            Some(point) => ConnectionStatus::Active(OpResult::Ok(point)),
                            None => ConnectionStatus::Active(OpResult::Err(concat_string!(self.id, ".read | Filtered by routes"))),
                        }
                    }
                    OpResult::Err(err) => {
                        if log::max_level() == LevelFilter::Trace {
                            warn!("{}.read | error: {:?}", self.id, err);
                        }
                        ConnectionStatus::Active(OpResult::Err(err))
                    }
                    OpResult::Timeout() => ConnectionStatus::Active(OpResult::Timeout()),
                }
            }
            ConnectionStatus::Closed(err) => {
                warn!("{}.read | error: {:?}", self.id, err);
                ConnectionStatus::Closed(err)
            }
        }
    }
}