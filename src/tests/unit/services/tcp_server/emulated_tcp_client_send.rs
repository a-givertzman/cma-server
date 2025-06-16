use coco::Stack;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    kernel::state::{Switch, SwitchCondition, SwitchState, SwitchStateChanged},
    services::{
        entity::{Name, Object, Point, PointTxId, ToPoint},
        Service,
    }, sync::channel,
};
use std::{fmt::Debug, io::Write, net::{SocketAddr, TcpStream}, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, Arc}, thread::{self, JoinHandle}, time::Duration};
use testing::entities::test_value::Value;
use crate::{
    core_::{net::protocols::jds::{jds_encode_message::JdsEncodeMessage, jds_serialize::JdsSerialize}, Mutex}, 
    tcp::steam_read::StreamRead,
};
///
/// Jast connects to the tcp socket on [address]
/// - all point from [test_data] will be sent via socket
/// - all received point in the received() method
/// - if [recvLimit] is some then thread exit when riched recvLimit
/// - [disconnect] - contains percentage (0..100) of test_data / iterations, where socket will be disconnected and connected again
pub struct EmulatedTcpClientSend {
    dbg: Dbg,
    name: Name,
    addr: SocketAddr,
    point_path: String,
    test_data: Vec<Value>,
    sent: Arc<Mutex<Vec<Point>>>,
    disconnect: Vec<i8>,
    wait_on_finish: bool,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
}
//
// 
impl EmulatedTcpClientSend {
    pub fn new(parent: impl Into<String>, point_path: impl Into<String>, addr: &str, test_data: Vec<Value>, disconnect: Vec<i8>, wait_on_finish: bool) -> Self {
        let name = Name::new(parent, format!("EmulatedTcpClientSend{}", COUNT.fetch_add(1, Ordering::Relaxed)));
        Self {
            dbg: Dbg::new(name.parent(), name.me()),
            name,
            addr: addr.parse().unwrap(),
            point_path: point_path.into(),
            test_data,
            sent: Arc::new(Mutex::new(vec![])),
            disconnect,
            wait_on_finish,
            handles: Handles::new(&dbg),
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// 
    #[allow(dead_code)]
    pub fn sent(&self) -> Arc<Mutex<Vec<Point>>> {
        self.sent.clone()
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
}
//
// 
impl Object for EmulatedTcpClientSend {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl Debug for EmulatedTcpClientSend {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EmulatedTcpClientSend")
            .field("id", &self.dbg)
            .finish()
    }
}
//
// 
impl Service for EmulatedTcpClientSend {
    //
    //
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let point_path = self.point_path.clone();
        let exit = self.exit.clone();
        let addr = self.addr.clone();
        let mut test_data = self.test_data.clone();
        let total_count = test_data.len();
        let sent = self.sent.clone();
        let disconnect = self.disconnect.iter().map(|v| {(*v as f32) / 100.0}).collect();
        let _wait_on_finish = self.wait_on_finish;
        let handle = thread::Builder::new().name(format!("{}.run Read", dbg)).spawn(move || {
            log::info!("{}.run | Preparing thread Read - ok", dbg);
            let mut switch_state = Self::switch_state(1, disconnect, 1.0);
            'connect: loop {
                match TcpStream::connect(addr) {
                    Ok(mut tcp_stream) => {
                        log::info!("{}.run | connected on: {:?}", dbg, addr);
                        thread::sleep(Duration::from_millis(100));
                        if !test_data.is_empty() {
                            let (send, recv) = channel::unbounded();
                            let mut jds_message = JdsEncodeMessage::new(
                                &dbg,
                                JdsSerialize::new(&dbg, recv)
                            );
                            // let request = PointType::String(Point::new(
                            //     0, 
                            //     &PointName::new(&point_path, "/Subscribe").full(),
                            //     json!(["/test/Jds/test"]).to_string(),
                            //     Status::Ok,
                            //     Cot::Req,
                            //     chrono::offset::Utc::now(),
                            // ));
                            // send.send(request).unwrap();
                            // thread::sleep(Duration::from_millis(100));
                            let tx_id = PointTxId::from_str(&dbg.to_string());
                            let mut sent_count = 0;
                            let mut progress_percent = 0.0;
                            while test_data.len() > 0 {
                                let value = test_data.remove(0);
                                let point = value.to_point(tx_id, &Name::new(&point_path, "/test").join());
                                send.send(point.clone()).unwrap();
                                match jds_message.read() {
                                    Ok(bytes) => {
                                        match &tcp_stream.write(&bytes) {
                                            Ok(_) => {
                                                sent.lock().push(point);
                                                sent_count += 1;
                                                progress_percent = (sent_count as f32) / (total_count as f32);
                                                switch_state.add(progress_percent);
                                                log::debug!("{}.run | sent: {:?}", dbg, value);
                                            }
                                            Err(err) => {
                                                log::warn!("{}.run | socket write error: {:?}", dbg, err);
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        panic!("{}.run | jdsSerialize error: {:?}", dbg, err);
                                    }
                                };
                                // if test_data.is_empty() && waitOnFinish {
                                //     info!("{}.run | waitOnFinish: {}", self_id, waitOnFinish);
                                //     while !exit.load(Ordering::SeqCst) {
                                //         thread::sleep(Duration::from_millis(100));
                                //     }
                                // }
                                if switch_state.changed() {
                                    log::info!("{}.run | state: {} progress percent: {}", dbg, switch_state.state(), progress_percent);
                                    thread::sleep(Duration::from_millis(1000));
                                    tcp_stream.flush().unwrap();
                                    thread::sleep(Duration::from_millis(1000));
                                    tcp_stream.shutdown(std::net::Shutdown::Both).unwrap();
                                    // drop(tcpStream);
                                    thread::sleep(Duration::from_millis(1000));
                                    break;
                                } 
                                if exit.load(Ordering::SeqCst) {
                                    break;
                                }
                            }
                        }
                        if switch_state.is_max() {
                            log::info!("{}.run | switchState.isMax, exiting", dbg);
                            break 'connect;
                        }
                        if test_data.is_empty() {
                            log::info!("{}.run | test_data.is_empty, exiting", dbg);
                            tcp_stream.flush().unwrap();
                            thread::sleep(Duration::from_millis(1000));
                            break 'connect;
                        }
                    }
                    Err(err) => {
                        log::warn!("{}.run | connection error: {:?}", dbg, err);
                        thread::sleep(Duration::from_millis(1000))
                    }
                }
                if switch_state.is_max() {
                    log::info!("{}.run | switchState.isMax, exiting", dbg);
                    break 'connect;
                }
                if test_data.is_empty() {
                    log::info!("{}.run | test_data.is_empty, exiting", dbg);
                    break 'connect;
                }
                if exit.load(Ordering::SeqCst) {
                    log::info!("{}.run | exit detected, exiting", dbg);
                    break 'connect;
                }
            }
            log::info!("{}.run | Exit", dbg);
        });
        match handle {
            Ok(handle) => {
                log::info!("{}.run | Starting - ok", self.dbg);
                self.handle.push(handle);
                Ok(())
            }
            Err(err) => {
                let err = Error::new(&self.dbg, "run").pass_with("Start failed", err.to_string());
                log::warn!("{}", err);
                Err(err)
            }
        }    }
    //
    //
    // fn points(&self) -> Vec<crate::conf::point_config::PointConfig> {
    //     let types = vec!["Bool", "Int", "Real", "Double", "String"];
    //     types.iter().map(|type_| {
    //         let conf = format!(
    //             r#"{}:
    //                 type: {}      # Bool / Int / Real, Double / String / Json
    //                 comment: Auth request, contains token / pass string"#, 
    //             PointName::new(&self.point_path, "/test").full(),
    //             type_,
    //         );
    //         let conf = serde_yaml::from_str(&conf).unwrap();
    //         PointConfig::from_yaml(&self.point_path, &conf)
    //     }).collect()
    // }
    //
    //
    fn wait(&self) -> Result<(), Error> {
        self.handle.wait()
    }
    //
    //
    fn is_finished(&self) -> bool {
        self.handles.is_finished()
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
