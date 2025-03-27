#[cfg(test)]
mod retain_buffer {
    use std::sync::Once;
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use crate::core_::retain_buffer::retain_buffer::RetainBuffer;
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
    fn basic() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!("test RetainBuffer");
        let mut buffer = RetainBuffer::new("test", "", Some(3));
        buffer.push(11);
        buffer.push(12);
        buffer.push(13);
        let result = buffer.pop_first().unwrap();
        let target = 11;
        assert!(
            result == target,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target,
        );
        let result = buffer.pop_first().unwrap();
        let target = 12;
        assert!(
            result == target,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target,
        );
        let result = buffer.pop_first().unwrap();
        let target = 13;
        assert!(
            result == target,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target,
        );
        let result = buffer.pop_first();
        let target = None;
        assert!(
            result == target,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target,
        );
    }
    ///
    ///
    #[test]
    fn capacity() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!("test RetainBuffer capacity");
        let mut buffer = RetainBuffer::new("test", "", Some(3));
        buffer.push(11);
        let result = buffer.len();
        let target = 1;
        assert!(
            result == target,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target,
        );
        buffer.push(12);
        let result = buffer.len();
        let target = 2;
        assert!(
            result == target,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target,
        );
        buffer.push(13);
        let result = buffer.len();
        let target = 3;
        assert!(
            result == target,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target
        );
        let result = buffer.pop_first().unwrap();
        let target = 11;
        assert!(
            result == target,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target,
        );
        let result = buffer.pop_first().unwrap();
        let target = 12;
        assert!(
            result == target,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target,
        );
        let result = buffer.pop_first().unwrap();
        let target = 13;
        assert!(
            result == target,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target,
        );
        let result = buffer.pop_first();
        let target = None;
        assert!(
            result == target,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target,
        );
        let result = buffer.len();
        let target = 0;
        assert!(
            result == target,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target,
        );
        buffer.push(14);
        buffer.push(15);
        buffer.push(16);
        buffer.push(17);
        let result = buffer.pop_first().unwrap();
        let target = 15;
        assert!(
            result == target,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target,
        );
        let result = buffer.pop_first().unwrap();
        let target = 16;
        assert!(
            result == target,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target,
        );
        let result = buffer.pop_first().unwrap();
        let target = 17;
        assert!(
            result == target,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target,
        );
        let result = buffer.pop_first();
        let target = None;
        assert!(
            result == target,
            "\nresult: {:?}\ntarget: {:?}",
            result,
            target,
        );
    }
}
