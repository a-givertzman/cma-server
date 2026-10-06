use std::{cell::Cell, sync::{Arc, atomic::{AtomicBool, Ordering}}, time::Duration};
use sal_core::dbg::Dbg;
use crate::domain::{Receiver, RwLock, Sender};
///
/// - Contains local/parents's [exit] signal
/// - Contains partner's [exit_pair] signal
/// - If [exit] is true, service exits main thread
/// - Rase [exit_pair] to true when partner service must exit main thread
pub struct ExitNotify {
    #[allow(unused)]
    id: Dbg,
    exit: Arc<AtomicBool>,
    exit_pair: Arc<AtomicBool>,
    exit_parent: Arc<AtomicBool>,
    send: RwLock<Sender<()>>,
    recv: RwLock<Receiver<()>>,
}
//
//
impl ExitNotify {
    ///
    /// Creates new instance of the ExitNotify
    pub fn new(
        parent: impl AsRef<str>,
        exit: Option<Arc<AtomicBool>>,
        exit_pair: Option<Arc<AtomicBool>>,
    ) -> Self {
        let (send, recv) = crate::domain::unbounded();
        Self {
            id: Dbg::new(parent, "ExitNotify"),
            exit: Arc::new(AtomicBool::new(false)),
            exit_pair: exit_pair.unwrap_or(Arc::new(AtomicBool::new(false))),
            exit_parent: exit.unwrap_or(Arc::new(AtomicBool::new(false))),
            send: RwLock::new(send),
            recv: RwLock::new(recv),
        }
    }
    ///
    /// Returns true if exit signal exists localy or from the partner
    pub fn get(&self) -> bool {
        self.exit.load(Ordering::Acquire) ||
        self.exit_pair.load(Ordering::Acquire) ||
        self.exit_parent.load(Ordering::Acquire)
    }
    ///
    /// Sends exit signal localy only
    pub fn exit(&self) {
        self.exit.store(true, Ordering::Release);
        let _ = self.send.read().close();
    }
    ///
    /// Sends exit signal to the partner only
    pub fn exit_pair(&self) {
        self.exit_pair.store(true, Ordering::Release);
    }
    ///
    /// Sends exit signal localy and to the partner
    pub fn exit_all(&self) {
        self.exit_pair.store(true, Ordering::Release);
        self.exit.store(true, Ordering::Release);
        let _ = self.send.read().close();
    }
    /// Returns `Backoff` object, connected with current `ExitNotify` and ready to terminate immediately on exit signal
    pub fn get_backoff(&self, delay: Duration, limit: Duration) -> Backoff {
        Backoff::new(delay, limit).with_exit(self.recv.read().clone())
    }
}

/// ### BackoffStrategy
/// 
/// Реализует паузу, которая увеличивается при каждой неудачной попытке (например, при повторном подключении к сети).
/// 
/// Так же этот объект может быть связан с `ExitNotify`, что позволяет немедленно прервать текущую паузу при завершении.
pub struct Backoff {
    init: Duration,
    delay: Cell<Duration>,
    limit: Duration,
    /// Все время жизни держим канал открытым для реализации recv_timeout,
    /// закрываем канал немедленно в случае завершения работы.
    #[allow(unused)]
    _send: Sender<()>,
    _recv: Receiver<()>,
    factor: f32,
}
impl Backoff {
    /// Создает новый инстанс `Backoff` с коэффициентом увеличения времени задержки в 2 раза.
    pub fn new(delay: Duration, limit: Duration) -> Self {
        let (_send, _recv) = crate::domain::unbounded();
        Self{
            init: delay,
            delay: Cell::new(delay),
            limit,
            _send,
            _recv,
            factor: 2.0
        }
    }
    /// Связывает текущий `Backoff` с `ExitNotify` для немедленного прерывания при сигнале `exit`.
    pub fn with_exit(mut self, recv: Receiver<()>) -> Self {
        self._recv = recv;
        self
    }
    /// Задает коэффициент возрастания времени паузы.
    pub fn with_factor(mut self, factor: impl Into<f32>) -> Self {
        self.factor = factor.into();
        self
    }
    /// Подождать заданное время.
    /// 
    /// Это блокирующий вызов, прервется при истечении заложенного `delay` или немедленно при сигнале exit.
    pub fn delay(&self) {
        let _ = self._recv.recv_timeout(self.delay.get());
    }
    /// Подождать заданное время и увеличить его.
    /// 
    /// Это блокирующий вызов, прервется при истечении заложенного `delay` или немедленно при сигнале exit.
    /// 
    /// Внутренний таймаут будет увеличен в backoff `factor` раз. По умолчанию в 2x.
    pub fn next_delay(&self) {
        let _ = self._recv.recv_timeout(self.delay.get());
        self.delay.update(|t| std::cmp::min(t.mul_f32(self.factor), self.limit));
    }
    /// ### Сброс в исходное состояние
    pub fn reset(&self) {
        self.delay.set(self.init);
    }
}
