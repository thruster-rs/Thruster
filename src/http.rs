use bytes::BytesMut;
use tokio_io::codec::{Encoder, Decoder};

use response::{self, Response};
use request::{self, Request};
use std::io;

pub struct Http;

impl Decoder for Http {
    type Item = Request;
    type Error = io::Error;


    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Request>> {
        let res = request::decode(buf);

        res
    }
}

impl Encoder for Http {
    type Item = Response;
    type Error = io::Error;


    fn encode(&mut self, msg: Response, buf: &mut BytesMut) -> io::Result<()> {
        response::encode(msg, buf);

        Ok(())
    }
}
