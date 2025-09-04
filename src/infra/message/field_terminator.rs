use sal_core::error::Error;
use crate::infra::message::{Bytes, MessageParse};

///
/// Used terminate the sequence of Message fields,
/// just returns passed bytes without changes
pub struct FieldTerminator {}
//
//
impl FieldTerminator {
    pub fn new() -> Self {
        Self {}
    }
}
impl<'a> MessageParse<(), (), Bytes> for FieldTerminator {
    ///
    /// Resets passed `bytes`
    fn parse(&mut self, bytes: Bytes) -> Result<((), (), Bytes), Error> {
        Ok(((), (), bytes))
    }
}
