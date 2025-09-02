use std::fmt::Debug;
use sal_core::{dbg::Dbg, error::Error};
use super::message::{Bytes, MessageParse};
///
/// Extracting `Data` field from the input bytes fixed length
pub struct FindField<'a, FieldIn, FieldOut, Out> {
    dbg: Dbg,
    size: usize,
    field: Box<dyn MessageParse<'a, FieldIn, FieldOut, Bytes>>,
    field_data: Option<(FieldIn, FieldOut)>,
    from_bytes: Box<dyn Fn(&[u8]) -> Result<Option<Out>, Error>>,
    remainder: Bytes,
}
//
//
impl<'a, FieldIn, FieldOut, Out> FindField<'a, FieldIn, FieldOut, Out> {
    ///
    /// Returns [FindField] new instance
    pub fn new(parent: impl Into<String>, size: usize, from_bytes: impl Fn(&[u8]) -> Result<Option<Out>, Error> + 'static, field: impl MessageParse<'a, FieldIn, FieldOut, Bytes> + 'static) -> Self {
        let dbg = Dbg::new(parent, format!("FindField(size {size})"));
        if size == 0 {
            panic!("{dbg}.new | Size should be >= 1");
        }
        Self {
            size,
            from_bytes: Box::new(from_bytes),
            field: Box::new(field),
            field_data: None,
            remainder: Vec::with_capacity(size - 1),
            dbg,
        }
    }
    ///
    /// Returns T of specified bytes length
    fn convert(&mut self, remainder: Vec<u8>) -> Result<(Out, Bytes), Error> {
        let mut e = Error::new(&self.dbg, "");
        if remainder.len() >= self.size {
            match remainder
                .windows(self.size)
                .find_map(|bytes| {
                    match (self.from_bytes)(bytes) {
                        Ok(data) => match data {
                                Some(data) => if remainder.len() >= self.size + 1 {
                                Some((data, remainder[(self.size + 1)..].to_vec()))
                            } else {
                                Some((data, vec![]))
                            },
                            None => None,
                        }
                        Err(err) => {
                            e = err;
                            None
                        },
                    }
                }) {
                    Some(val) => Ok(val),
                    None => {
                        self.remainder = remainder[(remainder.len() - self.size + 1)..].to_vec();
                        Err(Error::new(&self.dbg, "parse").pass_with("FromBytes error", e))
                    }
                }
        } else {
            self.remainder = remainder[(remainder.len() - self.size + 1)..].to_vec();
            Err(Error::new(&self.dbg, "parse").err("Take error"))
        }
    }
    ///
    /// Resets state to the initial
    fn reset(&mut self) {
        self.field_data = None;
        self.remainder = Vec::with_capacity(self.size - 1);
    }
}
//
//
impl<'a, FieldIn: Copy + Debug, FieldOut: Copy + Debug, Out: Debug> MessageParse<'a, (FieldIn, FieldOut), Out, Bytes> for FindField<'a, FieldIn, FieldOut, Out> {
    ///
    /// Extracting `Data` field from the input bytes
    /// - returns `Id`, `Kind`, `Size` & `Bytes` following by the `Size`
    /// - call this method multiple times, until the end of message
    fn parse(&mut self, bytes: Bytes) -> Result<((FieldIn, FieldOut), Out, Bytes), Error> {
        let error = Error::new(&self.dbg, "parse");
        let dbg = self.dbg.clone();
        let remainder = [std::mem::take(&mut self.remainder), bytes].concat();
        log::debug!("{dbg}.parse | remainder: {:?}", remainder);
        match self.field_data {
            Some((din, dout)) => match self.convert(remainder) {
                Ok((data, remainder)) => {
                    log::debug!("{}.parse | Field exist | din: {:?}, dout: {:?}, data: {:?}, remainder: {:?}", dbg, din, dout, data, remainder);
                    Ok(((din, dout), data, remainder))
                }
                Err(err) => Err(error.pass(err)),
            }
            None => {
                match self.field.parse(remainder) {
                    Ok((din, dout, remainder)) => {
                        match self.convert(remainder) {
                            Ok((data, remainder)) => {
                                log::debug!("{}.parse | Field parsed | din: {:?}, dout: {:?}, data: {:?}, remainder: {:?}", dbg, din, dout, data, remainder);
                                Ok(((din, dout), data, remainder))
                            }
                            Err(err) => {
                                self.field_data = Some((din, dout));
                                Err(error.pass(err))
                            }
                        }
                    }
                    Err(err) => Err(error.pass(err)),
                }
            }
        }
    }
}
