use std::{net::TcpStream, sync::{atomic::AtomicUsize, mpsc::Receiver, Arc, RwLock, RwLockReadGuard, RwLockWriteGuard}, time::Duration};
use sal_sync::services::{service::service::Service, subscription::subscriptions::Subscriptions, types::type_of::TypeOf};
use crate::{
    services::{safe_lock::lock_timer::LockTimer, server::connections::TcpServerConnections, services::Services},
    tcp::{tcp_read_alive::TcpReadAlive, tcp_stream_write::TcpStreamWrite, tcp_write_alive::TcpWriteAlive},
};
///
/// Defines methods to wrap the lock method on the RwLock
///  - for measure lock by timer to detect the dedlock 
///  - for debugging purposes
pub trait SafeLock<T> where T: ?Sized {
    ///
    /// Returns RwLockReadGuard on the RwLock
    fn rlock<'a>(&'a self, parent: impl Into<String>) -> RwLockReadGuard<'a, T> {
        _ = parent;
        panic!("SafeLock.rlock | Does not implemented for '{}'", self.type_of())
    }
    ///
    /// Returns RwLockWriteGuard on the RwLock
    fn wlock<'a>(&'a self, parent: impl Into<String>) -> RwLockWriteGuard<'a, T> {
        _ = parent;
        panic!("SafeLock.wlock | Does not implemented for '{}'", self.type_of())
    }
}
/// Counter of Lock's on the Services
static SERVICES_LOCK_COUNT: AtomicUsize = AtomicUsize::new(0);
//
// 
impl SafeLock<dyn Service> for Arc<RwLock<dyn Service>> {
    fn rlock<'a>(&'a self, parent: impl Into<String>) -> RwLockReadGuard<'a, (dyn Service + 'static)> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(10_000));
        lock_timer.run().unwrap();
        log::info!("SafeLock.rlock | Lock from '{}' on '{}'...", parent.into(), self_id);
        let rwlock_guard = self.read().unwrap();
        log::info!("SafeLock.rlock | Lock '{}' - ok", self_id);
        lock_timer.exit();
        rwlock_guard
    }
    fn wlock<'a>(&'a self, parent: impl Into<String>) -> RwLockWriteGuard<'a, (dyn Service + 'static)> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(10_000));
        lock_timer.run().unwrap();
        log::info!("SafeLock.wlock | Lock from '{}' on '{}'...", parent.into(), self_id);
        let mutax_guard = self.write().unwrap();
        log::info!("SafeLock.wlock | Lock '{}' - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
}
//
// 
impl SafeLock<dyn Service + Send> for Arc<RwLock<dyn Service + Send>> {
    fn rlock<'a>(&'a self, parent: impl Into<String>) -> RwLockReadGuard<'a, (dyn Service + Send + 'static)> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(10_000));
        lock_timer.run().unwrap();
        log::info!("SafeLock.rlock | Lock from '{}' on '{}'...", parent.into(), self_id);
        let rwlock_guard = self.read().unwrap();
        log::info!("SafeLock.rlock | Lock '{}' - ok", self_id);
        lock_timer.exit();
        rwlock_guard
    }
    fn wlock<'a>(&'a self, parent: impl Into<String>) -> RwLockWriteGuard<'a, (dyn Service + Send + 'static)> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(10_000));
        lock_timer.run().unwrap();
        log::info!("SafeLock.wlock | Lock from '{}' on '{}'...", parent.into(), self_id);
        let mutax_guard = self.write().unwrap();
        log::info!("SafeLock.wlock | Lock '{}' - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
}
//
//
impl SafeLock<Services> for Arc<RwLock<Services>> {
    fn rlock<'a>(&'a self, parent: impl Into<String>) -> RwLockReadGuard<'a, Services> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(10_000));
        lock_timer.run().unwrap();
        SERVICES_LOCK_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let count = SERVICES_LOCK_COUNT.load(std::sync::atomic::Ordering::SeqCst);
        log::info!("SafeLock.rlock | Lock ({}) from '{}' on {:?}...", count, parent.into(), self_id);
        let mutax_guard = self.read().unwrap();
        log::info!("SafeLock.rlock | Lock {:?} - ok", self_id);
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
        log::info!("SafeLock.wlock | Lock ({}) from '{}' on {:?}...", count, parent.into(), self_id);
        let mutax_guard = self.write().unwrap();
        log::info!("SafeLock.wlock | Lock {:?} - ok", self_id);
        SERVICES_LOCK_COUNT.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        lock_timer.exit();
        mutax_guard
    }
}
//
//
impl SafeLock<Subscriptions> for Arc<RwLock<Subscriptions>> {
    fn rlock<'a>(&'a self, parent: impl Into<String>) -> RwLockReadGuard<'a, Subscriptions> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(100));
        lock_timer.run().unwrap();
        log::trace!("SafeLock.rlock | Lock from '{}' on {:?}...", parent.into(), self_id);
        let mutax_guard = self.read().unwrap();
        log::trace!("SafeLock.rlock | Lock {:?} - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
    fn wlock<'a>(&'a self, parent: impl Into<String>) -> RwLockWriteGuard<'a, Subscriptions> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(100));
        lock_timer.run().unwrap();
        log::trace!("SafeLock.wlock | Lock from '{}' on {:?}...", parent.into(), self_id);
        let mutax_guard = self.write().unwrap();
        log::trace!("SafeLock.wlock | Lock {:?} - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
}
//
//
impl SafeLock<TcpStreamWrite> for Arc<RwLock<TcpStreamWrite>> {
    fn rlock<'a>(&'a self, parent: impl Into<String>) -> RwLockReadGuard<'a, TcpStreamWrite> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(100));
        lock_timer.run().unwrap();
        log::info!("SafeLock.rlock | Lock from '{}' on {:?}...", parent.into(), self_id);
        let mutax_guard = self.read().unwrap();
        log::info!("SafeLock.rlock | Lock {:?} - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
    fn wlock<'a>(&'a self, parent: impl Into<String>) -> RwLockWriteGuard<'a, TcpStreamWrite> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(100));
        lock_timer.run().unwrap();
        log::info!("SafeLock.wlock | Lock from '{}' on {:?}...", parent.into(), self_id);
        let mutax_guard = self.write().unwrap();
        log::info!("SafeLock.wlock | Lock {:?} - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
}
//
// 
impl SafeLock<Receiver<bool>> for Arc<RwLock<Receiver<bool>>> {
    fn rlock<'a>(&'a self, parent: impl Into<String>) -> RwLockReadGuard<'a, Receiver<bool>> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(100));
        lock_timer.run().unwrap();
        log::info!("SafeLock.rlock | Lock from '{}' on '{:?}'...", parent.into(), self_id);
        let mutax_guard = self.read().unwrap();
        log::info!("SafeLock.rlock | Lock: '{:?}' - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
    fn wlock<'a>(&'a self, parent: impl Into<String>) -> RwLockWriteGuard<'a, Receiver<bool>> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(100));
        lock_timer.run().unwrap();
        log::info!("SafeLock.wlock | Lock from '{}' on '{:?}'...", parent.into(), self_id);
        let mutax_guard = self.write().unwrap();
        log::info!("SafeLock.wlock | Lock: '{:?}' - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
}
//
// 
impl SafeLock<Vec<TcpStream>> for Arc<RwLock<Vec<TcpStream>>> {
    fn rlock<'a>(&'a self, parent: impl Into<String>) -> RwLockReadGuard<'a, Vec<TcpStream>> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(100));
        lock_timer.run().unwrap();
        log::info!("SafeLock.rlock | Lock from '{}' on '{:?}'...", parent.into(), self_id);
        let mutax_guard = self.read().unwrap();
        log::info!("SafeLock.rlock | Lock: '{:?}' - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
    fn wlock<'a>(&'a self, parent: impl Into<String>) -> RwLockWriteGuard<'a, Vec<TcpStream>> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(100));
        lock_timer.run().unwrap();
        log::info!("SafeLock.wlock | Lock from '{}' on '{:?}'...", parent.into(), self_id);
        let mutax_guard = self.write().unwrap();
        log::info!("SafeLock.wlock | Lock: '{:?}' - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
}
//
//
impl SafeLock<TcpServerConnections> for Arc<RwLock<TcpServerConnections>> {
    fn rlock<'a>(&'a self, parent: impl Into<String>) -> RwLockReadGuard<'a, TcpServerConnections> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(100));
        lock_timer.run().unwrap();
        log::info!("SafeLock.rlock | Lock from '{}' on {:?}...", parent.into(), self_id);
        let mutax_guard = self.read().unwrap();
        log::info!("SafeLock.rlock | Lock {:?} - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
    fn wlock<'a>(&'a self, parent: impl Into<String>) -> RwLockWriteGuard<'a, TcpServerConnections> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(100));
        lock_timer.run().unwrap();
        log::info!("SafeLock.wlock | Lock from '{}' on {:?}...", parent.into(), self_id);
        let mutax_guard = self.write().unwrap();
        log::info!("SafeLock.wlock | Lock {:?} - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
}
//
//
impl SafeLock<TcpReadAlive> for Arc<RwLock<TcpReadAlive>> {
    fn rlock<'a>(&'a self, parent: impl Into<String>) -> RwLockReadGuard<'a, TcpReadAlive> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(100));
        lock_timer.run().unwrap();
        log::info!("SafeLock.rlock | Lock from '{}' on {:?}...", parent.into(), self_id);
        let mutax_guard = self.read().unwrap();
        log::info!("SafeLock.rlock | Lock {:?} - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
    fn wlock<'a>(&'a self, parent: impl Into<String>) -> RwLockWriteGuard<'a, TcpReadAlive> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(100));
        lock_timer.run().unwrap();
        log::info!("SafeLock.wlock | Lock from '{}' on {:?}...", parent.into(), self_id);
        let mutax_guard = self.write().unwrap();
        log::info!("SafeLock.wlock | Lock {:?} - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
}
//
//
impl SafeLock<TcpWriteAlive> for Arc<RwLock<TcpWriteAlive>> {
    fn rlock<'a>(&'a self, parent: impl Into<String>) -> RwLockReadGuard<'a, TcpWriteAlive> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(100));
        lock_timer.run().unwrap();
        log::info!("SafeLock.rlock | Lock from '{}' on {:?}...", parent.into(), self_id);
        let mutax_guard = self.read().unwrap();
        log::info!("SafeLock.rlock | Lock {:?} - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
    fn wlock<'a>(&'a self, parent: impl Into<String>) -> RwLockWriteGuard<'a, TcpWriteAlive> {
        let self_id = format!("{:?}/SafeLock", self.type_of());
        let lock_timer = LockTimer::new(&self_id, self.type_of(), Duration::from_millis(100));
        lock_timer.run().unwrap();
        log::info!("SafeLock.wlock | Lock from '{}' on {:?}...", parent.into(), self_id);
        let mutax_guard = self.write().unwrap();
        log::info!("SafeLock.wlock | Lock {:?} - ok", self_id);
        lock_timer.exit();
        mutax_guard
    }
}
