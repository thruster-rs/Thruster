use std::fmt::{self, Write};
use bytes::{BytesMut, BufMut};
use tokio_io::codec::{Encoder, Decoder, Framed};
use tokio_io::{AsyncRead, AsyncWrite};
use httplib::Response;
use request::{self, Request};
use std::io;

enum StatusMessage {
    Ok,
    Custom(u32, String)
}

// pub struct Http;
//
// impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for Http {
//     type Request = Request;
//     type Response = Response;
//     type Transport = Framed<T, HttpCodec>;
//     type BindTransport = io::Result<Framed<T, HttpCodec>>;

//     fn bind_transport(&self, io: T) -> io::Result<Framed<T, HttpCodec>> {
//         Ok(io.framed(HttpCodec))
//     }
// }

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

    write!(FastWrite(buf), "\
        HTTP/1.1 {}\r\n\
        Server: Example\r\n\
        Content-Length: {}\r\n\
        Date: {}\r\n\
    ", msg.status(), length, now).unwrap();

    for (ref k, ref v) in msg.headers() {
        push(buf, k.as_ref());
        push(buf, ": ".as_bytes());
        push(buf, v.as_bytes());
        push(buf, "\r\n".as_bytes());
    }

    push(buf, "\r\n".as_bytes());
    push(buf, msg.body().as_bytes());
}

fn push(buf: &mut BytesMut, data: &[u8]) {
    buf.reserve(data.len());
    unsafe {
        buf.bytes_mut()[..data.len()].copy_from_slice(data);
        buf.advance_mut(data.len());
    }
}

// TODO: impl fmt::Write for Vec<u8>
//
// Right now `write!` on `Vec<u8>` goes through io::Write and is not super
// speedy, so inline a less-crufty implementation here which doesn't go through
// io::Error.
struct FastWrite<'a>(&'a mut BytesMut);

impl<'a> fmt::Write for FastWrite<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        push(&mut *self.0, s.as_bytes());
        Ok(())
    }

    fn write_fmt(&mut self, args: fmt::Arguments) -> fmt::Result {
        fmt::write(self, args)
    }
}

impl fmt::Display for StatusMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            StatusMessage::Ok => f.pad("200 OK"),
            StatusMessage::Custom(c, ref s) => write!(f, "{} {}", c, s),
        }
    }
}
