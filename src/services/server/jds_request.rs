use std::{collections::HashMap, sync::{Arc, RwLock}, thread, time::Duration};
use concat_string::concat_string;
use log::{debug, error, trace, warn};
use serde_json::json;
use crate::{
    conf::point_config::{name::Name, point_config::PointConfig},
    core_::{
        auth::ssh::auth_ssh::AuthSsh,
        cot::cot::Cot,
        net::protocols::jds::request_kind::RequestKind,
        point::{point_hlr::PointHlr, point::Point},
        status::status::Status,
    }, services::{
        multi_queue::subscription_criteria::SubscriptionCriteria,
        safe_lock::SafeLock,
        server::{jds_routes::RouterReply, jds_cnnection::JdsState},
        services::Services,
    }
};
use super::jds_cnnection::Shared;

pub struct JdsRequest {}
impl JdsRequest {
    ///
    /// Detecting kind of the request stored as json string in the incoming point.
    /// Performs the action depending on the Request kind.
    pub fn handle(parent_id: &str, parent: &Name, tx_id: usize, request: Point, services: Arc<RwLock<Services>>, shared: Arc<RwLock<Shared>>) -> RouterReply {
        let mut shared = shared.write().unwrap();
        let self_id = concat_string!(parent_id, "/JdsRequest");
        let requester_name = &parent.join();
        match RequestKind::from(request.name()) {
            RequestKind::AuthSecret => {
                debug!("{}.handle | Request '{}': \n\t{:?}", self_id, RequestKind::AUTH_SECRET, request);
                let (cot, message) = match &shared.auth {
                    crate::services::server::jds_auth::TcpServerAuth::Secret(auth_secret) => {
                        let secret = match request {
                            Point::String(request) => request.value,
                            _ => String::new(),
                        };
                        if secret == auth_secret.token() {
                            shared.jds_state = JdsState::Authenticated;
                            (Cot::ReqCon, "Authentication successful")
                        } else {
                            (Cot::ReqErr, "Authentication error: Invalid secret or kind of auth request")
                        }
                    }
                    _ => {
                        (Cot::ReqErr, "Authentication error: Invalid secret or kind of auth request")
                    }
                };
                RouterReply::new(
                    None,
                    Some(Point::String(PointHlr::new(
                        tx_id,
                        &Name::new(parent, "/Auth.Secret").join(),
                        message.to_owned(),
                        Status::Ok,
                        cot,
                        chrono::offset::Utc::now(),
                    ))),
                )
            }
            RequestKind::AuthSsh => {
                debug!("{}.handle | Request '{}': \n\t{:?}", self_id, RequestKind::AUTH_SSH, request);
                let (cot, message) = match &shared.auth {
                    crate::services::server::jds_auth::TcpServerAuth::Ssh(auth_ssh_path) => {
                        let secret = match request {
                            Point::String(request) => request.value,
                            _ => String::new(),
                        };
                        match AuthSsh::new(&auth_ssh_path.path()).validate(&secret) {
                            Ok(_) => {
                                shared.jds_state = JdsState::Authenticated;
                                (Cot::ReqCon, "Authentication successful".to_owned())
                            }
                            Err(err) => {
                                (Cot::ReqErr, format!("Authentication error: {}", err))
                            }
                        }
                    }
                    _ => {
                        (Cot::ReqErr, "Authentication error: Invalid secret or kind of auth request".to_owned())
                    }
                };
                RouterReply::new(
                    None,
                    Some(Point::String(PointHlr::new(
                        tx_id,
                        &Name::new(parent, "/Auth.Ssh").join(),
                        message.to_owned(),
                        Status::Ok,
                        cot,
                        chrono::offset::Utc::now(),
                    ))),
                )
            }
            RequestKind::Points => {
                debug!("{}.handle.Points | Request '{}': \n\t{:?}", self_id, RequestKind::POINTS, request);
                let points = services.rlock(&self_id).points(requester_name).then(
                    |points| points,
                    |err| {
                        error!("{}.handle.Points | Requesting points error: {:?}", self_id, err);
                        vec![]
                    },
                );
                let points: HashMap<String, &PointConfig> = points.iter().map(|conf| {
                    (conf.name.clone(), conf)
                }).collect();
                let points_len = points.len();
                let points = json!(points).to_string();
                let reply = RouterReply::new(
                    None,
                    Some(Point::String(PointHlr::new(
                        tx_id,
                        &Name::new(parent, "/Points").join(),
                        points,
                        Status::Ok,
                        Cot::ReqCon,
                        chrono::offset::Utc::now(),
                    ))),
                );
                debug!("{}.handle.Points | Reply: {:?} points", self_id, points_len);
                trace!("{}.handle.Points | Reply: \n\t{:#?}", self_id, reply);
                reply
            }
            RequestKind::Subscribe => {
                debug!("{}.handle.Subscribe | Request '{}': Point( name: {:?}, status: {:?}, cot: {:?}, timestamp: {:?})", self_id, RequestKind::SUBSCRIBE, request.name(), request.status(), request.cot(), request.timestamp());
                trace!("{}.handle.Subscribe | Request '{}': \n\t{:?}", self_id, RequestKind::SUBSCRIBE, request);
                let points = match serde_json::from_str(&request.value().as_string()) {
                    Ok(points) => {
                        let points: serde_json::Value = points;
                        match points.as_array() {
                            Some(points) => {
                                debug!("{}.handle.Subscribe | 'Subscribe' request (multicast)", self_id);
                                trace!("{}.handle.Subscribe | 'Subscribe' request (multicast): {:?}", self_id, request);
                                points.iter().fold(vec![], |mut points, point| {
                                    if let Some(point_name) = point.as_str() {
                                        points.extend(
                                            Self::map_points_to_creteria(point_name, vec![Cot::Inf, Cot::ActCon, Cot::ActErr])
                                        );
                                    }
                                    points
                                })
                            }
                            None => {
                                debug!("{}.handle.Subscribe | 'Subscribe' request (broadcast)", self_id);
                                trace!("{}.handle.Subscribe | 'Subscribe' request (broadcast): {:?}", self_id, request);
                                services.rlock(&self_id).points(requester_name).then(|points| points, |err| {
                                    error!("{}.handle.Subscribe | Requesting points error: {:?}", self_id, err);
                                    vec![]
                                })
                                .iter().fold(vec![], |mut points, point_conf| {
                                    points.extend(
                                        Self::map_points_to_creteria(&point_conf.name, vec![Cot::Inf, Cot::ActCon, Cot::ActErr])
                                    );
                                    points
                                })
                            }
                        }
                    }
                    Err(err) => {
                        warn!("{}.handle.Subscribe | 'Subscribe' request parsing error: {:?}\n\t request: {:?}", self_id, err, request);
                        services.rlock(&self_id).points(requester_name).then(|points| points, |err| {
                            error!("{}.handle.Subscribe | Requesting points error: {:?}", self_id, err);
                            vec![]
                        })
                        .iter().fold(vec![], |mut points, point_conf| {
                            points.extend(
                                Self::map_points_to_creteria(&point_conf.name, vec![Cot::Inf, Cot::ActCon, Cot::ActErr])
                            );
                            points
                        })
                    }
                };
                // let receiver_name = Name::new(parent, &shared.connection_id).join();
                let receiver_name = shared.subscribe_receiver.clone();
                debug!("{}.handle.Subscribe | extending subscription for receiver: '{}'", self_id, receiver_name);
                trace!("{}.handle.Subscribe |                              points: {:#?}", self_id, points);
                let (cot, message) = if points.is_empty() {
                    let message = format!("{}.handle.Subscribe | SUbscribe failed - points not found in the application", self_id);
                    warn!("{}", message);
                    (Cot::ReqErr, message)
                } else {
                    match services.wlock(&self_id).extend_subscription(&shared.subscribe, &receiver_name, &points) {
                        Ok(_) => (Cot::ReqCon, "".to_owned()),
                        Err(err) => {
                            let message = format!("{}.handle.Subscribe | Extend subscription failed with error: {:?}", self_id, err);
                            warn!("{}", message);
                            (Cot::ReqErr, message)
                        }
                    }
                };
                match shared.cache.clone() {
                    // TODO add named subscription
                    Some(cache_service) => Self::yield_gi(&self_id, &receiver_name, services, &cache_service, &[], &mut shared),
                    None => warn!("{}.handle.Subscribe | Gi skipped, cache service not configured", self_id),
                }
                let reply = RouterReply::new(
                    None,
                    Some(Point::String(PointHlr::new(
                        tx_id,
                        &Name::new(parent, "/Subscribe").join(),
                        message,
                        Status::Ok,
                        cot,
                        chrono::offset::Utc::now(),
                    ))),
                );
                debug!("{}.handle.Subscribe | Reply: {:?}", self_id, reply);
                reply
            }
            RequestKind::Unknown => {
                debug!("{}.handle | Unknown request: \n\t{:?}", self_id, request);
                warn!("{}.handle | Unknown request name: {:?}", self_id, request.name());
                RouterReply::new(None, None)
            }
        }
    }
    ///
    ///
    fn yield_gi(self_id: &str, receiver_name: &str, services: Arc<RwLock<Services>>, cache_service: &str, points: &[SubscriptionCriteria], shared: &mut Shared) {
        match services.rlock(self_id).get(cache_service) {
            Some(cache) => {
                let recv = cache.slock(self_id).gi(receiver_name, points);
                match shared.req_reply_send.pop() {
                    Some(send) => {
                        shared.req_reply_send.push(send.clone());
                        let self_id_clone = self_id.to_owned();
                        thread::spawn(move || {
                            thread::sleep(Duration::from_millis(32));
                            for point in recv.iter() {
                                if let Err(err) =  send.send(point) {
                                    error!("{}.handle.Subscribe | Send error: {:#?}", self_id_clone, err);
                                }
                            }
                        });
                    }
                    None => {
                        error!("{}.handle.Subscribe | Cant get req_reply_send", self_id)
                    }
                }
            }
            None => {
                warn!("{}.handle.Subscribe | Cache service '{}' - not found", self_id, cache_service)
            }
        }
        // match cache.slock() {}
    }
    ///
    /// Creates list of SubscriptionCriteria contains all variations of given [point_name] and Cot's
    fn map_points_to_creteria<'a>(point_name: &'a str, cots: Vec<Cot>) -> Box<dyn Iterator<Item = SubscriptionCriteria> + 'a> {
        Box::new(cots.into_iter().map(|cot| {
            SubscriptionCriteria::new(point_name.to_string(), cot)
        }))
    }
}