use std::fmt::{self, Write};

use bytes::BytesMut;

pub struct Response {
    pub response: Vec<u8>,
    pub status_message: StatusMessage,
    pub header_raw: BytesMut
}

pub enum StatusMessage {
    Ok,
    Custom(u32, String)
}

impl Response {
    pub fn new() -> Response {
        Response {
            response: Vec::new(),
            status_message: StatusMessage::Ok,
            header_raw: BytesMut::new()
        }
    }

    pub fn status_code(&mut self, code: u32, message: &str) -> &mut Response {
        self.status_message = StatusMessage::Custom(code, message.to_string());
        self
    }

    pub fn header(&mut self, name: &str, val: &str) -> &mut Response {
        let header_string = templatify! { "" ; name ; ": " ; val ; "\r\n" };
        self.header_raw.extend_from_slice(header_string.as_bytes());

        self
    }

    pub fn body(&mut self, s: &str) -> &mut Response {
        self.response = s.as_bytes().to_vec();
        self
    }

    pub fn body_bytes(&mut self, b: &[u8]) -> &mut Response {
        self.response = b.to_vec();
        self
    }

    pub fn body_bytes_from_vec(&mut self, b: Vec<u8>) -> &mut Response {
        self.response = b;
        self
    }
}

pub fn encode(msg: &Response, buf: &mut BytesMut) {
    let length = msg.response.len();
    let now = crate::date::now();

    write!(FastWrite(buf), "\
HTTP/1.1 {}\r\n\
Content-Length: {}\r\n\
Date: {}\r\n\n", msg.status_message, length, now).unwrap();

    buf.extend_from_slice(&msg.header_raw);
    buf.extend_from_slice(b"\r\n");
    buf.extend_from_slice(msg.response.as_slice());
}

impl Default for Response {
    fn default() -> Response {
        Response::new()
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
        (*self.0).extend_from_slice(s.as_bytes());
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
