use bytes::{BytesMut, BufMut};
use tokio_io::codec::{Encoder, Decoder};

use httplib::Response;
use request::{self, Request};
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
    type Item = Response<String>;
    type Error = io::Error;

    fn encode(&mut self, msg: Response<String>, buf: &mut BytesMut) -> io::Result<()> {
        encode(msg, buf);
        Ok(())
    }
}


pub fn encode(msg: Response<String>, buf: &mut BytesMut) {
    let length = msg.body().len();
    let now = ::date::now();

    templatify_buffer! { buf, "\
        HTTP/1.1 "; format!("{}", msg.status()) ;"\r\n\
        Server: Fanta\r\n\
        Content-Length: "; format!("{}", length) ;"\r\n\
        Date: "; format!("{}", now) ;"\r\n\
    " };

    for (ref k, ref v) in msg.headers() {
        let key: &str = k.as_ref();
        let val: &[u8] = v.as_bytes();
        templatify_buffer! { buf, ""; key ;": "; val ;"\r\n" };
    }

    templatify_buffer! { buf, "\r\n"; msg.body() ;"" };
}
