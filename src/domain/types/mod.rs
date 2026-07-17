mod channel;
mod collections;
mod fn_in_out_ref;
mod sync;

pub use channel::*;
pub use collections::*;
pub use fn_in_out_ref::*;
pub use sync::*;

///
/// ### Извлекает короткое имя типа без пути модуля.
/// Например: "cma_server::task::FnRetain" -> "FnRetain"
pub fn short_type_name<T: ?Sized>() -> String {
    pretty_type_name::pretty_type_name::<T>()
}
pub use short_type_name as me;
