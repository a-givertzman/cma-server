use log::{info, trace, warn, debug};
use sal_sync::services::{
    entity::{name::Name, object::Object, point::point::Point},
    service::{service::Service, service_handles::ServiceHandles},
};
use std::{fmt::Debug, io::Write, net::{SocketAddr, TcpStream}, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, Arc, Mutex}, thread, time::Duration};
use testing::entities::test_value::Value;
use crate::{
    core_::{
        net::{
            connection_status::ConnectionStatus,
            protocols::jds::{jds_decode_message::JdsDecodeMessage, jds_deserialize::JdsDeserialize},
        }, state::{switch_state::{Switch, SwitchCondition, SwitchState}, switch_state_changed::SwitchStateChanged}
    }, tcp::tcp_stream_write::OpResult
};


///
/// Jast connects to the tcp socket on [address]
/// - all point from [test_data] will be sent via socket
/// - all received point in the received() method
/// - if [recvLimit] is some then thread exit when riched recvLimit
/// - [disconnect] - contains percentage (0..100) of test_data / iterations, where socket will be disconnected and connected again
pub struct EmulatedTcpClientRecv {
    id: String,
    name: Name,
    addr: SocketAddr,
    received: Arc<Mutex<Vec<Point>>>,
    recv_limit: Option<usize>,
    must_received: Option<Value>,
    disconnect: Vec<i8>,
    marker_received: Arc<AtomicBool>,
    exit: Arc<AtomicBool>,
}
//
//
impl EmulatedTcpClientRecv {
    pub fn new(parent: impl Into<String>, addr: &str, recv_limit: Option<usize>, must_received: Option<Value>, disconnect: Vec<i8>) -> Self {
        let name = Name::new(parent, format!("EmulatedTcpClientRecv{}", COUNT.fetch_add(1, Ordering::Relaxed)));
        Self {
            id: name.join(),
            name,
            addr: addr.parse().unwrap(),
            received: Arc::new(Mutex::new(vec![])),
            recv_limit,
            must_received,
            disconnect,
            marker_received: Arc::new(AtomicBool::new(false)),
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    ///
    pub fn id(&self) -> String {
        self.id.clone()
    }
    ///
    ///
    pub fn received(&self) -> Arc<Mutex<Vec<Point>>> {
        self.received.clone()
    }
    ///
    ///
    fn switch_state<T: std::cmp::PartialOrd + Clone + 'static>(initial: u8, steps: Vec<T>, fin: T) -> SwitchStateChanged<u8, T> {
        fn switch<T: std::cmp::PartialOrd + Clone + 'static>(state: &mut u8, input: Option<T>) -> Switch<u8, T> {
            let state_ = *state;
            *state = *state + 1;
            let target = state;
            Switch{
                state: state_,
                conditions: vec![
                    SwitchCondition {
                        condition: Box::new(move |value| {
                            match input.clone() {
                                Some(input) => value >= input,
                                None => false,
                            }
                        }),
                        target: *target,
                    },
                ],
            }
        }
        let mut state: u8 = initial;
        let mut switches: Vec<Switch<u8, T>> = steps.into_iter().map(|input| {switch(&mut state, Some(input))}).collect();
            let state_ = state;
            state = state + 1;
            let target = state;
        switches.push(
            Switch{
                state: state_,
                conditions: vec![
                    SwitchCondition {
                        condition: Box::new(move |value| { value == fin}),
                        target: target,
                    },
                ],
            }
        );
        let switch_state: SwitchStateChanged<u8, T> = SwitchStateChanged::new(
            SwitchState::new(
                initial,
                switches,
            ),
        );
        switch_state
    }
    ///
    ///
    pub fn wait_all_received(&self) {
        let recv_limit = self.recv_limit.unwrap_or(0);
        info!("{}.waitAllReceived | wait all beeng received: {}/{}", self.id(), self.received.lock().unwrap().len(), recv_limit);
        loop {
            if self.received.lock().unwrap().len() >= recv_limit {
                break;
            }
            thread::sleep(Duration::from_millis(100));
            trace!("{}.waitAllReceived | wait all beeng received: {}/{}", self.id(), self.received.lock().unwrap().len(), recv_limit);
        }
    }
    ///
    ///
    pub fn wait_marker_received(&self) {
        match &self.must_received {
            Some(must_received) => {
                info!("{}.waitMarkerReceived | Wait for {:?} marker beeng received", self.id, must_received);
                loop {
                    if self.marker_received.load(Ordering::SeqCst) {
                        break;
                    }
                    thread::sleep(Duration::from_millis(100));
                    trace!("{}.waitMarkerReceived | wait for {:?} marker beeng received", self.id, self.must_received);
                }
            }
            None => {}
        }
    }
}
//
//
impl Object for EmulatedTcpClientRecv {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
//
impl Debug for EmulatedTcpClientRecv {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EmulatedTcpClientRecv")
            .field("id", &self.id)
            .finish()
    }
}
//
//
impl Service for EmulatedTcpClientRecv {
    //
    //
    fn get_link(&mut self, _name: &str) -> std::sync::mpsc::Sender<Point> {
        panic!("{}.get_link | Does not support static producer", self.id())
        // match self.rxSend.get(name) {
        //     Some(send) => send.clone(),
        //     None => panic!("{}.run | link '{:?}' - not found", self.id, name),
        // }
    }
    //
    //
    fn run(&mut self) -> Result<ServiceHandles<()>, String> {
        info!("{}.run | Starting...", self.id);
        let self_id = self.id.clone();
        let exit = self.exit.clone();
        let marker_received = self.marker_received.clone();
        let addr = self.addr.clone();
        let received = self.received.clone();
        let recv_limit = self.recv_limit.clone();
        let must_received = self.must_received.clone();
        let disconnect = self.disconnect.iter().map(|v| {(*v as f32) / 100.0}).collect();
        let handle = thread::Builder::new().name(format!("{}.run Read", self_id)).spawn(move || {
            info!("{}.run | Preparing thread Read - ok", self_id);
            let mut switch_state = Self::switch_state(1, disconnect, 1.0);
            let mut switch_state_changed = false;
            'connect: loop {
                match TcpStream::connect(addr) {
                    Ok(mut tcp_stream) => {
                        info!("{}.run | connected on: {:?}", self_id, addr);
                        let mut jds_deserialize = JdsDeserialize::new(
                            self_id.clone(),
                            JdsDecodeMessage::new(
                                self_id.clone(),
                            ),
                        );
                        match recv_limit {
                            Some(recv_limit) => {
                                if recv_limit > 0 {
                                    let mut progress_percent = 0.0;
                                    let mut received_count = 0;
                                    loop {
                                        match jds_deserialize.read(&tcp_stream) {
                                            ConnectionStatus::Active(result) => {
                                                trace!("{}.run | received: {:?}", self_id, result);
                                                match result {
                                                    OpResult::Ok(point) => {
                                                        debug!("{}.run | received: {:?}", self_id, point);
                                                        received.lock().unwrap().push(point.clone());
                                                        received_count += 1;
                                                        progress_percent = (received_count as f32) / (recv_limit as f32);
                                                        switch_state.add(progress_percent);
                                                        if let Some(must_received) = &must_received {
                                                            let marker_received_ = match must_received {
                                                                Value::Bool(value) => value == &point.as_bool().value.0,
                                                                Value::Int(value) => value == &point.as_int().value,
                                                                Value::Real(value) => value == &point.as_real().value,
                                                                Value::Double(value) => value == &point.as_double().value,
                                                                Value::String(value) => value == &point.as_string().value,
                                                            };
                                                            if marker_received_ {
                                                                info!("{}.run | received marker {:?}, exiting...", self_id, point);
                                                                marker_received.store(marker_received_, Ordering::SeqCst);
                                                                break;
                                                            }
                                                        }
                                                    }
                                                    OpResult::Err(err) => {
                                                        warn!("{}.run | read socket error: {:?}", self_id, err);
                                                    }
                                                    OpResult::Timeout() => {}
                                                }
                                            }
                                            ConnectionStatus::Closed(err) => {
                                                warn!("{}.run | socket connection closed: {:?}", self_id, err);
                                                break;
                                            }
                                        };
                                        if switch_state_changed {
                                            switch_state_changed = false;
                                            info!("{}.run | state: {} progress percent: {}", self_id, switch_state.state(), progress_percent);
                                            tcp_stream.shutdown(std::net::Shutdown::Both).unwrap();
                                            drop(tcp_stream);
                                            thread::sleep(Duration::from_millis(1000));
                                            break;
                                        }
                                        if switch_state.changed() {
                                            info!("{}.run | state: {} progress percent: {}", self_id, switch_state.state(), progress_percent);
                                            switch_state_changed = true;
                                            tcp_stream.flush().unwrap();
                                        }
                                        if received_count >= recv_limit {
                                            exit.store(true, Ordering::SeqCst);
                                            break;
                                        }
                                        if exit.load(Ordering::SeqCst) {
                                            break;
                                        }
                                    }
                                }
                            }
                            None => {
                                loop {
                                    match jds_deserialize.read(&tcp_stream) {
                                        ConnectionStatus::Active(result) => {
                                            trace!("{}.run | received: {:?}", self_id, result);
                                            match result {
                                                OpResult::Ok(point) => {
                                                    received.lock().unwrap().push(point);
                                                }
                                                OpResult::Err(err) => {
                                                    warn!("{}.run | read socket error: {:?}", self_id, err);
                                                }
                                                OpResult::Timeout() => {}
                                            }
                                        }
                                        ConnectionStatus::Closed(err) => {
                                            warn!("{}.run | socket connection closed: {:?}", self_id, err);
                                            break;
                                        }
                                    };
                                    if exit.load(Ordering::SeqCst) {
                                        break;
                                    }
                                }
                            }
                        };
                    }
                    Err(err) => {
                        warn!("{}.run | connection error: {:?}", self_id, err);
                        thread::sleep(Duration::from_millis(1000))
                    }
                }
                if exit.load(Ordering::SeqCst) {
                    break 'connect;
                }
            }
            info!("{}.run | Exit thread Recv", self_id);
        });
        match handle {
            Ok(handle) => {
                info!("{}.run | Starting - ok", self.id);
                Ok(ServiceHandles::new(vec![(self.id.clone(), handle)]))
            }
            Err(err) => {
                let message = format!("{}.run | Start failed: {:#?}", self.id, err);
                warn!("{}", message);
                Err(message)
            }
        }
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }
}
///
/// Global static counter of FnOut instances
static COUNT: AtomicUsize = AtomicUsize::new(0);
