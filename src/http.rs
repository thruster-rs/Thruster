use bytes::{BytesMut, BufMut};
use tokio_io::codec::{Encoder, Decoder};

// use httplib::Response;
use response::{self, Response};
use request::{self, Request};
use std::io;
use std::time::{Duration, Instant};
use std::fmt::Write;

pub struct Http;

impl Decoder for Http {
    type Item = Request;
    type Error = io::Error;


    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Request>> {
        //#bench*/ let instant = Instant::now();
        let res = request::decode(buf);
        //#bench*/ let new_now = Instant::now();
        //#bench*/ println!("decode {:?}", new_now.duration_since(instant));

        res
    }
}

impl Encoder for Http {
    type Item = Response;
    type Error = io::Error;


    fn encode(&mut self, msg: Response, buf: &mut BytesMut) -> io::Result<()> {
        //#bench*/ let instant = Instant::now();
        response::encode(msg, buf);
        //#bench*/ let new_now = Instant::now();
        //#bench*/ println!("encode {:?}", new_now.duration_since(instant));

        Ok(())
    }
}


// pub fn encode(msg: Response, buf: &mut BytesMut) {
//     let length = msg.body().len();
//     let now = ::date::now();
//     let mut now_str = String::new();
//     let parts = msg.into_parts();
//     let status = parts.0.status.to_string();
//     write!(now_str, "{}", now);

//     templatify_buffer! { buf, "\
//         HTTP/1.1 "; status ;"\r\n\
//         Server: thruster\r\n\
//         Content-Length: "; length.to_string() ;"\r\n\
//         Date: "; now_str ;"\r\n\
//     " };


//     // templatify_buffer! { buf, "\
//     //     HTTP/1.1 "; msg.status().to_string() ;"\r\n\
//     //     Server: thruster\r\n\
//     //     Content-Length: "; length.to_string() ;"\r\n\
//     //     Date: "; now.to_string() ;"\r\n\
//     // " };

//     // for (k, v) in parts.0.headers.iter() {
//     //     let key: &str = k.as_str();
//     //     let val: &[u8] = v.as_bytes();
//     //     templatify_buffer! { buf, ""; key ;": "; val ;"\r\n" };
//     // }

//     templatify_buffer! { buf, "\r\n"; parts.1 ;"" };
// }
