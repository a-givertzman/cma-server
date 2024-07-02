#[cfg(test)]
mod tests {
    use log::debug;
    use std::{sync::Once, str::FromStr};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::conf::conf_keywd::{ConfKeywd, ConfKeywdValue, ConfKind};
    ///
    ///
    static INIT: Once = Once::new();
    ///
    /// once called initialisation
    fn init_once() {
        INIT.call_once(|| {
            // implement your initialisation code to be called only once for current test file
        })
    }
    ///
    /// returns:
    ///  - ...
    fn init_each() -> () {}
    ///
    ///
    #[test]
    fn test_create_valid() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!("test_create_valid");
        // let (initial, switches) = init_each();
        let test_data = vec![
            ("service ApiClient", ConfKeywd::Service( ConfKeywdValue {prefix: format!(""), kind: ConfKind::Service, name: format!("ApiClient"), sufix: format!("")} )),
            ("service ApiClient ApiClient1", ConfKeywd::Service( ConfKeywdValue {prefix: format!(""), kind: ConfKind::Service, name: format!("ApiClient"), sufix: format!("ApiClient1")} )),
            ("service MultiQueue", ConfKeywd::Service( ConfKeywdValue {prefix: format!(""), kind: ConfKind::Service, name: format!("MultiQueue"), sufix: format!("")} )),
            ("task Task1", ConfKeywd::Task( ConfKeywdValue {prefix: format!(""), kind: ConfKind::Task, name: format!("Task1"), sufix: format!("")} )),
            ("task task1", ConfKeywd::Task( ConfKeywdValue {prefix: format!(""), kind: ConfKind::Task, name: format!("task1"), sufix: format!("")} )),
            ("in queue queue1", ConfKeywd::Queue( ConfKeywdValue {prefix: format!("in"), kind: ConfKind::Queue, name: format!("queue1"), sufix: format!("")} )),
            ("in link link", ConfKeywd::Link( ConfKeywdValue {prefix: format!("in"), kind: ConfKind::Link, name: format!("link"), sufix: format!("")} )),
            ("in queue in-queue", ConfKeywd::Queue( ConfKeywdValue {prefix: format!("in"), kind: ConfKind::Queue, name: format!("in-queue"), sufix: format!("")} )),
            ("out queue", ConfKeywd::Queue( ConfKeywdValue {prefix: format!("out"), kind: ConfKind::Queue, name: format!(""), sufix: format!("")} )),
            ("out link", ConfKeywd::Link( ConfKeywdValue {prefix: format!("out"), kind: ConfKind::Link, name: format!(""), sufix: format!("")} )),
        ];
        for (value, target) in test_data {
            let result = ConfKeywd::from_str(value).unwrap();
            debug!("value: {:?}   |   ConfKind: {:?}   |   target: {:?}", value, result, target);
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
    }

    // #[test]
    // fn test_create_invalid() {
    //     DebugSession::init(LogLevel::Info, Backtrace::Short);
    //     init_once();
    //     init_each();
    //     info!("test_create_invalid");
    //     // let (initial, switches) = init_each();
    //     let test_data: Vec<(&str, Result<&str, ()>)> = vec![
    //         ("fn:name", Err(())),
    //         ("fn\nname", Err(())),
    //         ("fn: name", Err(())),
    //         ("fn :name", Err(())),
    //         ("fn : name", Err(())),
    //         ("Fn name", Err(())),
    //         ("FN name", Err(())),
    //         ("fnName", Err(())),
    //         ("fn_name", Err(())),
    //         ("let:name", Err(())),
    //         ("Let name", Err(())),
    //         ("LET name", Err(())),
    //         ("letName", Err(())),
    //         ("let_name", Err(())),
    //         ("const:name", Err(())),
    //         ("Const name", Err(())),
    //         ("CONST name", Err(())),
    //         ("constName", Err(())),
    //         ("const_name", Err(())),
    //         ("point:name", Err(())),
    //         ("Point name", Err(())),
    //         ("POINT name", Err(())),
    //         ("pointName", Err(())),
    //         ("point_name", Err(())),
    //     ];
    //     for (value, target) in test_data {
    //         let fnConfigType = ConfKeywd::from_str(value);
    //         debug!("value: {:?}   |   fnConfigType: {:?}   |   target: {:?}", value, fnConfigType, target);
    //         assert_eq!(fnConfigType.is_err(), true);
    //     }
    // }
}
