pub mod lock_timer;
use std::{net::TcpStream, sync::{atomic::AtomicUsize, mpsc::Receiver, Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard}, time::Duration};
use log::{info, trace};
use sal_sync::services::{service::service::Service, subscription::subscriptions::Subscriptions, types::type_of::TypeOf};
use crate::{
    services::safe_lock::lock_timer::LockTimer, 
    tcp::{steam_read::TcpStreamRead, tcp_read_alive::TcpReadAlive, tcp_stream_write::TcpStreamWrite, tcp_write_alive::TcpWriteAlive},
};
use super::{server::connections::TcpServerConnections, services::Services};
///
/// Defines methods to wrap the lock method on the kind of Mutex / RwLock.. etc...
///  - for measure lock by timer to detect the dedlock 
///  - for debugging purposes
pub trait SafeLock<T> where T: ?Sized {
    ///
    /// Returns MutaxGuard on the Mutax 
    fn slock(&self, parent: impl Into<String>) -> MutexGuard<T>;
    ///
    /// Returns RwLockReadGuard on the RwLock
    fn rlock<'a>(&'a self, parent: impl Into<String>) -> RwLockReadGuard<'a, T> {
        _ = parent;
        panic!("SafeLock.slock | Does not implemented for '{}'", self.type_of())
    }
    ///
    /// Returns RwLockWriteGuard on the RwLock
    fn wlock<'a>(&'a self, parent: impl Into<String>) -> RwLockWriteGuard<'a, T> {
        _ = parent;
        panic!("SafeLock.slock | Does not implemented for '{}'", self.type_of())
    }
}
/// Counter of Lock's on the Services
static SERVICES_LOCK_COUNT: AtomicUsize = AtomicUsize::new(0);
//
// 
impl SafeLock<dyn Service> for Arc<Mutex<dyn Service>> {
    fn slock(&self, parent: impl Into<String>) -> MutexGuard<'_, (dyn Service + 'static)> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(100));
        info!("SafeLock.slock | Lock from '{}' on service: '{:?}'...", parent.into(), self_id);
        let mutax_guard = self.lock().unwrap();
        info!("SafeLock.slock | Lock service: '{:?}' - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
}
//
// 
impl SafeLock<dyn Service> for Arc<RwLock<dyn Service>> {
    fn slock(&self, parent: impl Into<String>) -> MutexGuard<'_, (dyn Service + 'static)> {
        panic!("SafeLock.slock | Lock from {} on '{}' - Does not implemented", parent.into(), self.type_of())
    }
    fn rlock<'a>(&'a self, parent: impl Into<String>) -> RwLockReadGuard<'a, (dyn Service + 'static)> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(10_000));
        lock_timer.run().unwrap();
        info!("SafeLock.rlock | Lock from '{}' on '{}'...", parent.into(), self_id);
        let rwlock_guard = self.read().unwrap();
        info!("SafeLock.rlock | Lock '{}' - ok", self_id);
        lock_timer.exit();
        rwlock_guard
    }
    fn wlock<'a>(&'a self, parent: impl Into<String>) -> RwLockWriteGuard<'a, (dyn Service + 'static)> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(10_000));
        lock_timer.run().unwrap();
        info!("SafeLock.wlock | Lock from '{}' on '{}'...", parent.into(), self_id);
        let mutax_guard = self.write().unwrap();
        info!("SafeLock.wlock | Lock '{}' - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
}
//
// 
impl SafeLock<dyn Service + Send> for Arc<RwLock<dyn Service + Send>> {
    fn slock(&self, parent: impl Into<String>) -> MutexGuard<'_, (dyn Service + Send + 'static)> {
        panic!("SafeLock.slock | Lock from {} on '{}' - Does not implemented", parent.into(), self.type_of())
    }
    fn rlock<'a>(&'a self, parent: impl Into<String>) -> RwLockReadGuard<'a, (dyn Service + Send + 'static)> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(10_000));
        lock_timer.run().unwrap();
        info!("SafeLock.rlock | Lock from '{}' on '{}'...", parent.into(), self_id);
        let rwlock_guard = self.read().unwrap();
        info!("SafeLock.rlock | Lock '{}' - ok", self_id);
        lock_timer.exit();
        rwlock_guard
    }
    fn wlock<'a>(&'a self, parent: impl Into<String>) -> RwLockWriteGuard<'a, (dyn Service + Send + 'static)> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(10_000));
        lock_timer.run().unwrap();
        info!("SafeLock.wlock | Lock from '{}' on '{}'...", parent.into(), self_id);
        let mutax_guard = self.write().unwrap();
        info!("SafeLock.wlock | Lock '{}' - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
}
//
// 
impl SafeLock<dyn Service + Send> for Arc<Mutex<dyn Service + Send>> {
    fn slock(&self, parent: impl Into<String>) -> MutexGuard<'_, (dyn Service + Send + 'static)> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(100));
        info!("SafeLock.slock | Lock from '{}' on service: '{:?}'...", parent.into(), self_id);
        let mutax_guard = self.lock().unwrap();
        info!("SafeLock.slock | Lock service: '{:?}' - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
}
//
//
impl SafeLock<Services> for Arc<Mutex<Services>> {
    fn slock(&self, parent: impl Into<String>) -> MutexGuard<'_, Services> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(10_000));
        lock_timer.run().unwrap();
        SERVICES_LOCK_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let count = SERVICES_LOCK_COUNT.load(std::sync::atomic::Ordering::SeqCst);
        info!("SafeLock.slock | Lock ({}) from '{}' on {:?}...", count, parent.into(), self_id);
        let mutax_guard = self.lock().unwrap();
        info!("SafeLock.slock | Lock {:?} - ok", self_id);
        SERVICES_LOCK_COUNT.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        lock_timer.exit();
        mutax_guard
    }
}
//
//
impl SafeLock<Services> for Arc<RwLock<Services>> {
    fn slock(&self, parent: impl Into<String>) -> MutexGuard<Services> {
        panic!("SafeLock.slock | Lock from {} on '{}' - Does not implemented", parent.into(), self.type_of())
    }
    fn rlock<'a>(&'a self, parent: impl Into<String>) -> RwLockReadGuard<'a, Services> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(10_000));
        lock_timer.run().unwrap();
        SERVICES_LOCK_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let count = SERVICES_LOCK_COUNT.load(std::sync::atomic::Ordering::SeqCst);
        info!("SafeLock.rlock | Lock ({}) from '{}' on {:?}...", count, parent.into(), self_id);
        let mutax_guard = self.read().unwrap();
        info!("SafeLock.rlock | Lock {:?} - ok", self_id);
        SERVICES_LOCK_COUNT.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        lock_timer.exit();
        mutax_guard
    }
    fn wlock<'a>(&'a self, parent: impl Into<String>) -> RwLockWriteGuard<'a, Services> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(10_000));
        lock_timer.run().unwrap();
        SERVICES_LOCK_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let count = SERVICES_LOCK_COUNT.load(std::sync::atomic::Ordering::SeqCst);
        info!("SafeLock.wlock | Lock ({}) from '{}' on {:?}...", count, parent.into(), self_id);
        let mutax_guard = self.write().unwrap();
        info!("SafeLock.wlock | Lock {:?} - ok", self_id);
        SERVICES_LOCK_COUNT.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        lock_timer.exit();
        mutax_guard
    }
}
//
//
impl SafeLock<Subscriptions> for Arc<Mutex<Subscriptions>> {
    fn slock(&self, parent: impl Into<String>) -> MutexGuard<'_, Subscriptions> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(100));
        lock_timer.run().unwrap();
        trace!("SafeLock.slock | Lock from '{}' on {:?}...", parent.into(), self_id);
        let mutax_guard = self.lock().unwrap();
        trace!("SafeLock.slock | Lock {:?} - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
}
//
//
impl SafeLock<TcpServerConnections> for Arc<Mutex<TcpServerConnections>> {
    fn slock(&self, parent: impl Into<String>) -> MutexGuard<'_, TcpServerConnections> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(100));
        lock_timer.run().unwrap();
        info!("SafeLock.slock | Lock from '{}' on {:?}...", parent.into(), self_id);
        let mutax_guard = self.lock().unwrap();
        info!("SafeLock.slock | Lock {:?} - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
}
//
//
impl SafeLock<TcpReadAlive> for Arc<Mutex<TcpReadAlive>> {
    fn slock(&self, parent: impl Into<String>) -> MutexGuard<'_, TcpReadAlive> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(100));
        lock_timer.run().unwrap();
        info!("SafeLock.slock | Lock from '{}' on {:?}...", parent.into(), self_id);
        let mutax_guard = self.lock().unwrap();
        info!("SafeLock.slock | Lock {:?} - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
}
//
//
impl SafeLock<TcpWriteAlive> for Arc<Mutex<TcpWriteAlive>> {
    fn slock(&self, parent: impl Into<String>) -> MutexGuard<'_, TcpWriteAlive> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(100));
        lock_timer.run().unwrap();
        info!("SafeLock.slock | Lock from '{}' on {:?}...", parent.into(), self_id);
        let mutax_guard = self.lock().unwrap();
        info!("SafeLock.slock | Lock {:?} - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
}
//
//
impl SafeLock<TcpStreamWrite> for Arc<Mutex<TcpStreamWrite>> {
    fn slock(&self, parent: impl Into<String>) -> MutexGuard<'_, TcpStreamWrite> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(100));
        lock_timer.run().unwrap();
        info!("SafeLock.slock | Lock from '{}' on {:?}...", parent.into(), self_id);
        let mutax_guard = self.lock().unwrap();
        info!("SafeLock.slock | Lock {:?} - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
}
//
// 
impl SafeLock<dyn TcpStreamRead> for Arc<Mutex<dyn TcpStreamRead>> {
    fn slock(&self, parent: impl Into<String>) -> MutexGuard<'_, (dyn TcpStreamRead + 'static)> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(100));
        lock_timer.run().unwrap();
        info!("SafeLock.slock | Lock from '{}' on '{:?}'...", parent.into(), self_id);
        let mutax_guard = self.lock().unwrap();
        info!("SafeLock.slock | Lock: '{:?}' - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
}
//
// 
impl SafeLock<Receiver<bool>> for Arc<Mutex<Receiver<bool>>> {
    fn slock(&self, parent: impl Into<String>) -> MutexGuard<'_, Receiver<bool>> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(100));
        lock_timer.run().unwrap();
        info!("SafeLock.slock | Lock from '{}' on '{:?}'...", parent.into(), self_id);
        let mutax_guard = self.lock().unwrap();
        info!("SafeLock.slock | Lock: '{:?}' - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
}
//
// 
impl SafeLock<Vec<TcpStream>> for Arc<Mutex<Vec<TcpStream>>> {
    fn slock(&self, parent: impl Into<String>) -> MutexGuard<'_, Vec<TcpStream>> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(100));
        lock_timer.run().unwrap();
        info!("SafeLock.slock | Lock from '{}' on '{:?}'...", parent.into(), self_id);
        let mutax_guard = self.lock().unwrap();
        info!("SafeLock.slock | Lock: '{:?}' - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
}