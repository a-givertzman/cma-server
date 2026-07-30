use std::{sync::Arc, time::Duration};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{kernel::state::ExitNotify, services::{EventValueAccess, RegistryConf, Service, ServiceWaiting, Services, SubscriptionCriteria, conf::ServicesConf, entity::{Cot, Name, Object, Point}}, sync::Handles, thread_pool::{Scheduler, ThreadPool}};
use crate::{domain::{RECV_TIMEOUT, unbounded, FxSccHashMap, Receiver, Sender}};

/// ### Event Aggregation
/// 
/// Accumulates incoming events into an in-memory concurrent map as `f64` values,
/// providing calculation services with a lock-free reactive state snapshot.
pub struct EventValues {
    name: Name,
    /// Name of the service for subscribing to RPM event's
    subscribe: String,
    /// Current state of the registered events
    state: Arc<FxSccHashMap<String, Option<f64>>>,
    /// Receivers will have all incoming events
    listeners: Arc<FxSccHashMap<String, Sender<Point>>>,
    /// Provides access to the all application services
    services: Arc<Services>,
    /// Thread scheduler
    scheduler: Scheduler,
    /// Handles of the internaly executed threads 
    handles: Handles<()>,
    /// Exit signal
    exit: Arc<ExitNotify>,
    dbg: Dbg,
}
//
//
impl EventValues {
    ///
    /// ### Returns [Inputs] new instance
    /// - `parent` - Parent entity identifier (for debugging).
    /// - `subscribe` - Target service name for subscribing to the necessary event's
    pub fn new(
        parent: impl Into<String>,
        subscribe: impl Into<String>,
        services: Arc<Services>,
        scheduler: Scheduler,
        exit: Arc<ExitNotify>,
    ) -> Self {
        let name = Name::new(parent, "Inputs");
        let dbg = Dbg::new(name.parent(), name.me());
        Self {
            name,
            subscribe: subscribe.into(),
            state: Arc::new(FxSccHashMap::default()),
            listeners: Arc::new(FxSccHashMap::default()),
            services,
            scheduler,
            handles: Handles::new(&dbg),
            exit,
            dbg,
        }
    }
    ///
    /// Returns fake [Inputs] new instance for testing purposes
    /// - `init` - Collection with pairs key - value, represents an initial state of required values
    #[cfg(test)]
    pub(crate) fn fake(
        parent: impl Into<String>,
        init: impl IntoIterator<Item = (impl Into<String>, f64)>,
        exit: Arc<ExitNotify>,
    ) -> Self {
        let name = Name::new(parent, "Inputs");
        let dbg = Dbg::new(name.parent(), name.me());
        let tp = ThreadPool::new(&dbg, Some(4));
        let state = Arc::new(FxSccHashMap::default());
        for (key, val) in init {
            _ = state.upsert_sync(key.into(), Some(val));
        }
        Self {
            name: name.clone(),
            subscribe: String::new(),
            state,
            listeners: Arc::new(FxSccHashMap::default()),
            services: Arc::new(Services::new(
                &dbg,
                ServicesConf { name, retain: RegistryConf { path: None, point: None } },
                None,
            ).unwrap()),
            scheduler: tp.scheduler(),
            handles: Handles::new(&dbg),
            exit,
            dbg,
        }
    }
    ///
    /// ### Apply new value into the current state
    /// Used for internal or testing purposes only.
    /// In nornal operation events should be received by the subscription.
    #[cfg(test)]
    pub(crate) fn insert(&self, key: impl Into<String>, val: f64) {
        _ = self.state.upsert_sync(key.into(), Some(val));
    }
    /// ### Real-time event broadcast stream
    /// Returns a receiving channel that forwards all points processed by the internal loop.
    /// Automatically cleans up dropped or disconnected receivers.
    pub fn listen(&self) -> Receiver<Point> {
        let key = format!("listener-{}", self.listeners.len());
        let (send, recv) = unbounded();
        _ = self.listeners.insert_sync(key, send);
        recv
    }
}
//
impl EventValueAccess<str, f64> for EventValues {    
    //
    fn subscribe(&self, key: &str) {
        if self.handles.is_finished() {
            log::error!("Subscriptions must be completed before the service is launched!");
        }
        _ = self.state.upsert_sync(key.into(), None);
    }
    //
    fn get(&self, key: &str) -> Option<f64> {
        let Some(val) = self.state.read_sync(key, |_, v| *v) else {
            log::warn!("{}.get | Unknown '{}' event value  requested", self.dbg, key);
            return None;
        };
        val
    }
}
//
impl Object for EventValues {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
// 
impl std::fmt::Debug for EventValues {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Inputs")
            .field("dbg", &self.dbg)
            .finish()
    }
}
//
impl Service for EventValues where {
    // 
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let name = self.name.clone();
        let state = self.state.clone();
        let listeners = self.listeners.clone();
        let wait_started = Some(Duration::from_millis(1));
        let service_waiting = ServiceWaiting::new(&name, wait_started);
        let service_release = service_waiting.release();
        let services = self.services.clone();
        let subscribe = self.subscribe.clone();
        let exit = self.exit.clone();
        let mut subscriptions = Vec::with_capacity(self.state.len());
        self.state.iter_sync(|key, _| {
            let subscription = SubscriptionCriteria::new(key, Cot::Inf);
            // log::trace!("{dbg}.run | Subscription: {:?}", subscription);
            subscriptions.push(subscription);
            true
        });
        log::trace!("{dbg}.run | Subscriptions: {:?}", subscriptions);
        let handle = self.scheduler.spawn(move || {
            let dbg = &dbg;
            let (_, recv) = services.subscribe(&subscribe, &name.join(), &subscriptions);
            service_release.add(Ok(()));
            while !exit.get() {
                log::trace!("{dbg}.run | Receiving points...");
                match recv.recv_timeout(RECV_TIMEOUT) {
                    Ok(event) => {
                        let key = event.name();
                        let is_updated = state.update_sync(&key, |key, value| {
                            log::debug!("{dbg}.run | Event '{}', value: {:?}", key, event.value());
                            match &event {
                                Point::Bool(_) => log::warn!("{dbg}.run | Event '{}' - expected numeric type, but has 'Bool'", key),
                                Point::Int(point) => _ = value.replace(point.value as f64),
                                Point::Real(point) => _ = value.replace(point.value as f64),
                                Point::Double(point) => _ = value.replace(point.value),
                                Point::String(_) => log::warn!("{dbg}.run | Event '{}' - expected numeric type, but has 'String'", key),
                                Point::Bytes(_) => log::warn!("{dbg}.run | Event '{}' - expected numeric type, but has 'Bytes'", key),
                            }
                            listeners.iter_mut_sync(|listener| {
                                if let Err(err) = listener.send(event.clone()) {
                                    log::warn!("{dbg}.run | Send error {:?}", err);
                                    _ = listener.consume();
                                }
                                true
                            });
                        }).is_some();
                        if !is_updated {
                            log::warn!("{dbg}.run | Unexpected Event '{}'", key);
                        }
                    }
                    Err(crate::domain::RecvTimeoutError::Timeout) => {},
                    Err(err) => {
                        log::warn!("{dbg}.run | Receive error: {:?}", err);
                        break;
                    }
                }
            }
            log::info!("{dbg}.run | Exit");
        });
        match handle {
            Ok(handle) => self.handles.push(handle),
            Err(err) => return Err(Error::new(&self.dbg, "run").pass_with("Start failed", err.to_string())),
        }
        let r = match wait_started {
            Some(_) => {
                log::info!("{}.run | Waiting while starting...", self.dbg);
                service_waiting.wait()
            }
            None => Ok(()),
        };
        log::info!("{}.run | Starting - ok", self.dbg);
        r
    }
    //
    fn wait(&self) -> Result<(), Error> {
        self.handles.wait()
    }
    //
    fn is_finished(&self) -> bool {
        self.handles.is_finished()
    }
    //
    fn exit(&self) {
        self.exit.exit();
    }    
}
///
/// Basic Tests
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_inputs_state_flow() {
        // 1. Инициализируем сигнал завершения и начальные фейковые данные
        let exit = Arc::new(ExitNotify::new("test_inputs_state_flow", None, None));
        let initial_data = vec![
            ("sensor.temperature", 22.5),
            ("sensor.pressure", 101.3),
        ];
        // 2. Создаем тестируемый объект через ваш новый метод `fake`
        let inputs = EventValues::fake("test_parent", initial_data, exit);
        // 3. Проверяем корректность начального состояния через `get`
        assert_eq!(inputs.get("sensor.temperature"), Some(22.5));
        assert_eq!(inputs.get("sensor.pressure"), Some(101.3));
        // Значение для незарегистрированного ключа должно вернуть None
        assert_eq!(inputs.get("sensor.humidity"), None);
        // 4. Обновляем существующие значения через `insert`
        inputs.insert("sensor.temperature", 23.8);
        inputs.insert("sensor.pressure", 100.1);
        // 5. Проверяем, что кэш `scc::HashMap` успешно обновился
        assert_eq!(inputs.get("sensor.temperature"), Some(23.8));
        assert_eq!(inputs.get("sensor.pressure"), Some(100.1));
    }
}
