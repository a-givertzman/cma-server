///
/// Позволяет пробросить локальную структуру в поток.
/// 
/// Даже если внутри структуры Rc / RefCell и подобные синхронные сущности, которые не имплементят `Send`
/// 
/// **Внимание!** Это можно делать только если структура лоуальная и болностью живет в одном потоке! В противном случае ошибки панмяти!
pub struct SendWrapper<T>(T);
//
impl<T> SendWrapper<T> {
    ///
    /// Заворачиват синхронный тип `T` что бы передать его в поток
    pub fn wrap(v: T) -> Self {
        Self(v)
    }
    ///
    /// Извлекает оригинал `T` из обертки
    pub fn extract(self) -> T {
        self.0
    }
}
//
impl<T> std::ops::Deref for SendWrapper<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
//
impl<T> std::ops::DerefMut for SendWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
// Что бы передать в поток
unsafe impl<T> Send for SendWrapper<T> {}
