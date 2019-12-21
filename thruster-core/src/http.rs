use bytes::BytesMut;
use tokio_util::codec::{Encoder, Decoder};

use crate::response::{self, Response};
use crate::request::{self, Request};
use std::io;

pub struct Http;

impl Decoder for Http {
    type Item = Request;
    type Error = io::Error;


    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Request>> {
        request::decode(buf)
    }
}

impl Encoder for Http {
    type Item = Response;
    type Error = io::Error;


    fn encode(&mut self, msg: Response, buf: &mut BytesMut) -> io::Result<()> {
        response::encode(&msg, buf);

        Ok(())
    }
}
