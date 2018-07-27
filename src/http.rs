use bytes::BytesMut;
use tokio_io::codec::{Encoder, Decoder, Framed};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_proto::pipeline::ServerProto;

use response::{self, Response};
use request::{self, Request};
use std::io;

pub struct HttpProto;

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for HttpProto {
    type Request = Request;
    type Response = Response;
    type Transport = Framed<T, Http>;
    type BindTransport = io::Result<Framed<T, Http>>;

    fn bind_transport(&self, io: T) -> io::Result<Framed<T, Http>> {
        Ok(io.framed(Http))
    }
}

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
