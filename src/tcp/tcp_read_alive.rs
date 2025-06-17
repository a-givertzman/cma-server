use coco::Stack;
use log::LevelFilter;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{entity::Point, ServiceCycle}, sync::{channel::Sender, Handles}, thread_pool::Scheduler};
use std::{
    io::BufReader, net::TcpStream, 
    sync::{atomic::{AtomicBool, Ordering}, Arc},
    thread::{self}, time::Duration,
};
use crate::{core_::net::connection_status::ConnectionStatus, tcp::tcp_stream_write::OpResult};
use super::steam_read::TcpStreamRead;

///
/// Transfering points from JdsStream (socket) to the Channel Sender<PointType>
pub struct TcpReadAlive {
    dbg: Dbg,
    stream_read: Arc<Stack<Box<dyn TcpStreamRead>>>,
    send: Sender<Point>,
    cycle: Option<Duration>,
    scheduler: Option<Scheduler>,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
    exit_pair: Arc<AtomicBool>,
}
impl TcpReadAlive {
    ///
    /// Creates new instance of [TcpReadAlive]
    /// - [parent] - the ID if the parent entity
    /// - [exit] - notification from parent to exit 
    /// - [exitPair] - notification from / to sibling pair to exit 
    pub fn new(
        parent: impl Into<String>, 
        stream_read: Box<dyn TcpStreamRead>,
        dest: Sender<Point>, 
        cycle: Option<Duration>,
        exit: Option<Arc<AtomicBool>>, 
        exit_pair: Option<Arc<AtomicBool>>,
        scheduler: Option<Scheduler>,
    ) -> Self {
        let dbg = Dbg::new(parent, "TcpReadAlive");
        let stream_read_stack = Arc::new(Stack::new());
        stream_read_stack.push(stream_read);
        Self {
            dbg: dbg.clone(),
            stream_read: stream_read_stack,
            send: dest,
            cycle,
            scheduler,
            handles: Handles::new(&dbg),
            exit: exit.unwrap_or(Arc::new(AtomicBool::new(false))),
            exit_pair: exit_pair.unwrap_or(Arc::new(AtomicBool::new(false))),
        }
    }
    ///
    /// Main loop
    fn run_(
        dbg: Dbg,
        cycle: Option<Duration>,
        tcp_stream: TcpStream,
        send: Sender<Point>,
        stream_read: Arc<Stack<Box<dyn TcpStreamRead>>>,
        exit: Arc<AtomicBool>,
        exit_pair: Arc<AtomicBool>,
    ) {
        let mut cycle = cycle.map(|cycle| ServiceCycle::new(&dbg, cycle));
        let mut tcp_stream = BufReader::new(tcp_stream);
        let mut tcp_stream_read = stream_read.pop().unwrap();
        loop {
            if let Some(cycle) = &mut cycle {cycle.start()}
            match tcp_stream_read.read(&mut tcp_stream) {
                ConnectionStatus::Active(point) => {
                    match point {
                        OpResult::Ok(point) => {
                            // debug!("{}.run | read point: {:?}", dbg, point);
                            match send.send(point) {
                                Ok(_) => {}
                                Err(err) => {
                                    log::warn!("{}.run | write to queue error: {:?}", dbg, err);
                                }
                            };
                            if let Some(cycle) = &mut cycle {cycle.wait()}
                        }
                        OpResult::Err(err) => {
                            if log::max_level() == LevelFilter::Trace {
                                log::warn!("{}.run | error: {:?}", dbg, err);
                            }
                            if let Some(cycle) = &mut cycle {cycle.wait()}
                        }
                        OpResult::Timeout() => {}
                    }
                }
                ConnectionStatus::Closed(err) => {
                    log::warn!("{}.run | error: {:?}", dbg, err);
                    exit_pair.store(true, Ordering::SeqCst);
                    break;
                }
            };
            if exit.load(Ordering::SeqCst) | exit_pair.load(Ordering::SeqCst) {
                break;
            }
        }
        stream_read.push(tcp_stream_read);
        log::info!("{}.run | Exit", dbg);
    }
    ///
    /// Main loop of the [TcpReadAlive]
    pub fn run(&self, tcp_stream: TcpStream) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let cycle = self.cycle;
        let send = self.send.clone();
        let stream_read = self.stream_read.clone();
        let exit = self.exit.clone();
        let exit_pair = self.exit_pair.clone();
        match &self.scheduler {
            Some(scheduler) => {
                let handle = scheduler.spawn(move || {
                    Self::run_(dbg, cycle, tcp_stream, send, stream_read, exit, exit_pair);
                    Ok(())
                })?;
                self.handles.push(handle);

            }
            None => {
                let handle = thread::Builder::new().name(format!("{} - Read", dbg.clone())).spawn(move || {
                    Self::run_(dbg, cycle, tcp_stream, send, stream_read, exit, exit_pair);
                }).map_err(|err| Error::new(&self.dbg, "run").err(err.to_string()))?;
                self.handles.push(handle);
            }
        }
        log::info!("{}.run | started", self.dbg);
        Ok(())
    }
    ///
    /// Waits for main loop being finished
    /// 
    /// call `exit()` to finish main loop
    pub fn wait(&self) -> Result<(), Error> {
        self.handles.wait()
    }
    ///
    /// Returns `true` if main loop has been finished
    #[allow(unused)]
    pub fn is_finished(&self) -> bool {
        self.handles.is_finished()
    }
    ///
    /// Sends exit signal to [TcpReadAlive]
    #[allow(unused)]
    pub fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }
}
//
//
impl std::fmt::Debug for TcpReadAlive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TcpReadAlive")
            .field("dbg", &self.dbg)
            .finish()
    }
}
