use std::{net::TcpStream, sync::{atomic::{AtomicBool, Ordering}, Arc}, thread::{self}, time::Duration};
use coco::Stack;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::ServiceCycle, sync::Handles, thread_pool::Scheduler};
use crate::{
    domain::net::connection_status::ConnectionStatus, tcp::tcp_stream_write::{OpResult, TcpStreamWrite},
};
///
/// Transfering points from Channel Sender<PointType> to the JdsStream (socket)
pub struct TcpWriteAlive {
    dbg: Dbg,
    cycle: Option<Duration>,
    stream_write: Arc<Stack<TcpStreamWrite>>,
    scheduler: Option<Scheduler>,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
    exit_pair: Arc<AtomicBool>,
}
//
// 
impl TcpWriteAlive {
    ///
    /// Creates new instance of [TcpWriteAlive]
    /// - [parent] - the ID if the parent entity
    /// - [exit] - notification from parent to exit 
    /// - [exitPair] - notification from / to sibling pair to exit 
    pub fn new(
        parent: impl Into<String>,
        cycle: Option<Duration>,
        stream_write: TcpStreamWrite,
        exit: Option<Arc<AtomicBool>>,
        exit_pair: Option<Arc<AtomicBool>>,
        scheduler: Option<Scheduler>,
    ) -> Self {
        let dbg = Dbg::new(parent, "TcpWriteAlive");
        let stream_write_stack = Arc::new(Stack::new());
        stream_write_stack.push(stream_write);
        Self {
            cycle,
            stream_write: stream_write_stack,
            scheduler,
            handles: Handles::new(&dbg),
            dbg,
            exit: exit.unwrap_or(Arc::new(AtomicBool::new(false))),
            exit_pair: exit_pair.unwrap_or(Arc::new(AtomicBool::new(false))),
        }
    }
    ///
    ///
    fn run_(
        dbg: Dbg,
        cycle: Option<Duration>,
        tcp_stream: TcpStream,
        stream_write: Arc<Stack<TcpStreamWrite>>,
        exit: Arc<AtomicBool>,
        exit_pair: Arc<AtomicBool>,
    ) {
        let mut cycle = cycle.map(|cycle| ServiceCycle::new(&dbg, cycle));
        let mut stream = stream_write.pop().unwrap();
        'main: loop {
            if let Some(cycle) = &mut cycle {cycle.start()}
            match stream.write(&tcp_stream) {
                ConnectionStatus::Active(result) => {
                    match result {
                        OpResult::Ok(_) => {
                            if let Some(cycle) = &mut cycle {cycle.wait()}
                        }
                        OpResult::Err(err) => {
                            log::warn!("{}.run | error: {:?}", dbg, err);
                            if let Some(cycle) = &mut cycle {cycle.wait()}
                        }
                        OpResult::Timeout() => {}
                    }
                }
                ConnectionStatus::Closed(err) => {
                    log::warn!("{}.run | error: {:?}", dbg, err);
                    exit_pair.store(true, Ordering::SeqCst);
                    break 'main;
                }
            };
            if exit.load(Ordering::SeqCst) | exit_pair.load(Ordering::SeqCst) {
                break 'main;
            }
        }
        stream_write.push(stream);
        log::info!("{}.run | Exit", dbg);
    }
    /// 
    /// Main loop of the [TcpReadAlive]
    pub fn run(&self, tcp_stream: TcpStream) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let exit = self.exit.clone();
        let exit_pair = self.exit_pair.clone();
        let cycle = self.cycle;
        let stream_write = self.stream_write.clone();
        match &self.scheduler {
            Some(scheduler) => {
                let handle = scheduler.spawn(move || {
                    Self::run_(dbg, cycle, tcp_stream, stream_write, exit, exit_pair);
                })?;
                self.handles.push(handle);
            }
            None => {
                let handle = thread::Builder::new().name(format!("{} - Write", dbg.clone())).spawn(move || {
                    Self::run_(dbg, cycle, tcp_stream, stream_write, exit, exit_pair);
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
    /// Sends exit signal to [TcpWriteAlive]
    #[allow(unused)]
    pub fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }
}
//
//
impl std::fmt::Debug for TcpWriteAlive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TcpWriteAlive")
            .field("dbg", &self.dbg)
            .finish()
    }
}
