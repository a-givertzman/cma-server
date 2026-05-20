use std::marker::PhantomData;

///
/// Позволяет безопасно пробросить любую структуру в поток.
/// Даже если внутри структуры Rc / RefCell и подобные синхронные сущности,
/// которые не имплементят `Send`
pub struct SendWrapper<T> {
    ptr: *mut T,
    _p: PhantomData<T>,
}
//
impl<T> SendWrapper<T> {
    ///
    /// Заворачиват синхронный тип `T` что бы передать его в поток
    pub fn wrap(v: T) -> Self {
        Self {
            // Box::into_raw возвращает чистый указатель *mut T
            ptr: Box::into_raw(Box::new(v)),
            _p: PhantomData,
        }
    }
    ///
    /// Извлекает оригинал `T` из обертки
    pub fn extract(self) -> T {
        // Используем std::mem::ManuallyDrop, чтобы Drop обертки 
        // случайно не удалил память, которую мы только что извлекли
        let this = std::mem::ManuallyDrop::new(self);
        unsafe { *Box::from_raw(this.ptr) }
    }
}
// Защита от утечки памяти: если extract() не вызвали, чистим память вручную
impl<T> Drop for SendWrapper<T> {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.ptr);
        }
    }
}
// Что передать в поток
unsafe impl<T> Send for SendWrapper<T> {}
