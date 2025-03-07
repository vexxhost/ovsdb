use bytes::{BufMut, BytesMut};
use serde_json::Value;
use std::io;
use tokio_util::codec::{Decoder, Encoder};

pub struct JsonCodec;

impl Encoder<BytesMut> for JsonCodec {
    type Error = io::Error;

    fn encode(&mut self, data: BytesMut, buf: &mut BytesMut) -> Result<(), io::Error> {
        buf.reserve(data.len());
        buf.put(data);
        Ok(())
    }
}

impl Decoder for JsonCodec {
    type Item = Value;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Value>, io::Error> {
        if src.is_empty() {
            return Ok(None);
        }

        match serde_json::from_slice::<Value>(src) {
            Ok(val) => {
                src.clear();

                Ok(Some(val))
            }
            Err(ref e) if e.is_eof() => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
