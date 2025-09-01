use sal_core::{dbg::Dbg, error::Error};
use crate::infra::message::FromBytes;
use super::message::{Bytes, MessageParse};
///
/// Extracting `Data` field from the input bytes
pub struct ParseData<FieldIn, FieldOut, Out> {
    dbg: Dbg,
    size: usize,
    field: Box<dyn MessageParse<FieldIn, FieldOut, Out>>,
    field_data: Option<(FieldIn, FieldOut)>,
    remainder: Bytes,
}
//
//
impl<Out: FromBytes, FieldIn, FieldOut> ParseData<FieldIn, FieldOut, Out> {
    ///
    /// Returns [ParseData] new instance
    pub fn new(parent: impl Into<String>, size: usize, field: impl MessageParse<FieldIn, FieldOut, Out> + 'static) -> Self {
        Self {
            size,
            field: Box::new(field),
            field_data: None,
            remainder: vec![],
            dbg: Dbg::new(parent, "ParseData"),
        }
    }
    ///
    /// Returns T of specified bytes length
    fn take(&mut self, mut remainder: Vec<u8>) -> Result<(Out, Bytes), Error> {
        if remainder.len() >= self.size {
            let bytes = remainder.drain(..self.size);
            // let dbg_bytes = if data_bytes.len() > 16 {format!("{:?}...", &data_bytes[..16])} else {format!("{:?}", data_bytes)};
            // log::trace!("{}.parse | data_bytes: {:?}", self.dbg, dbg_bytes);
            self.reset();
            match Out::from_bytes(&bytes.collect::<Vec<u8>>()) {
                Ok(data) => Ok((data, remainder)),
                Err(err) => Err(Error::new(&self.dbg, "parse").pass_with("FromBytes error", err)),
            }
        } else {
            self.remainder.extend(remainder);
            Err(Error::new(&self.dbg, "parse").err("Take error"))
        }
    }
    ///
    /// Resets state to the initial
    fn reset(&mut self) {
        self.field_data = None;
        self.remainder = vec![];
    }
}
//
//
impl<Out: FromBytes, FieldIn, FieldOut> MessageParse<FieldIn, FieldOut, Out> for ParseData<FieldIn, FieldOut, Out> {
    ///
    /// Extracting `Data` field from the input bytes
    /// - returns `Id`, `Kind`, `Size` & `Bytes` following by the `Size`
    /// - call this method multiple times, until the end of message
    fn parse(&mut self, bytes: Bytes) -> Result<(FieldIn, FieldOut, Out, Bytes), Error> {
        let error = Error::new(&self.dbg, "parse");
        let mut remainder = [std::mem::take(&mut self.remainder), bytes].concat();
        match self.field_data.take() {
            Some((din, dout)) => self.take(remainder).map(|(data, remainder)| (din, dout, data, remainder)),
            None => {
                match self.field.parse(remainder) {
                    Ok((din, dout, data, mut remainder)) => {
                        match self.take(remainder) {
                            Ok((data, remainder)) => Ok((din, dout, data, remainder)),
                            Err(err) => {
                                self.field_data = Some((din, dout));
                                Err(error.pass(err))
                            }
                        }
                    }
                    Err(err) => Err(error.pass(err))
                }
            }
        }
    }
}
