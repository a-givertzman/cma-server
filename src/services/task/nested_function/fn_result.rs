///
/// Result returning from the Task FnOut
#[derive(Clone, Debug)]
pub enum FnResult<T, E> {
    Ok(T),
    None,
    Err(E),
}
//
//
impl<T, E: std::fmt::Debug> FnResult<T, E> {
    ///
    /// Returns the contained [Ok] value, consuming the self value.
    /// Otherwise panic.
    ///
    /// Because this function may panic, its use is generally discouraged.
    /// Instead, prefer to use pattern matching and handle the [None] case explicitly,
    /// or call unwrap_or, unwrap_or_else, or unwrap_or_default.
    #[allow(unused)]
    pub fn unwrap(self) -> T {
        match self {
            Self::Ok(v) => v,
            Self::None => panic!("FnResult.unwrap | Called on a None value"),
            Self::Err(err) => panic!("FnResult.unwrap | Called on a error: {:#?}", err),
        }
    }
}