use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

use crate::core::request::{decode, Request};
use crate::core::response::{encode, Response};
use std::io;

pub struct Http;

impl Decoder for Http {
    type Item = Request;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Request>> {
        decode(buf)
    }
}

impl Encoder<Response> for Http {
    type Error = io::Error;

    fn encode(&mut self, msg: Response, buf: &mut BytesMut) -> io::Result<()> {
        encode(&msg, buf);

        Ok(())
    }
}
