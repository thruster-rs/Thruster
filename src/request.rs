use std::{io, str, fmt};
use std::collections::HashMap;
use serde;
use bytes::BytesMut;
use serde_json;
use smallvec::SmallVec;

use httparse;
use httplib;

pub struct Request {
    body: Slice,
    method: Slice,
    path: Slice,
    version: u8,
    // TODO: use a small vec to avoid this unconditional allocation
    pub headers: SmallVec<[(Slice, Slice); 8]>,
    data: BytesMut,
    pub params: HashMap<String, String>,
    pub query_params: HashMap<String, String>
}

type Slice = (usize, usize);

impl Request {
    pub fn raw_body(&self) -> &str {
        str::from_utf8(self.slice(&self.body)).unwrap()
    }

    pub fn method(&self) -> &str {
        str::from_utf8(self.slice(&self.method)).unwrap()
    }

    pub fn path(&self) -> &str {
        str::from_utf8(self.slice(&self.path)).unwrap()
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn headers(&self) -> HashMap<String, String> {
        let mut header_map = HashMap::new();

        for slice_pair in self.headers.iter() {
            header_map.insert(
                str::from_utf8(self.slice(&slice_pair.0)).unwrap().to_owned(),
                str::from_utf8(self.slice(&slice_pair.1)).unwrap().to_owned()
            );
        }

        header_map
    }

    pub fn body_as<'a, T>(&self, body: &'a str) -> serde_json::Result<T>
        where T: serde::de::Deserialize<'a>
    {
        serde_json::from_str(body)
    }

    fn slice(&self, slice: &Slice) -> &[u8] {
        &self.data[slice.0..slice.1]
    }

    pub fn params(&self) -> &HashMap<String, String> {
        &self.params
    }

    pub fn query_params(&self) -> &HashMap<String, String> {
        &self.query_params
    }

    pub fn set_params(&mut self, params: HashMap<String, String>) {
        self.params = params;
    }

    pub fn set_query_params(&mut self, query_params: HashMap<String, String>) {
        self.query_params = query_params;
    }
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<HTTP Request {} {}>", self.method(), self.path())
    }
}


pub fn decode(buf: &mut BytesMut) -> io::Result<Option<Request>> {
    // TODO: we should grow this headers array if parsing fails and asks
    //       for more headers
    let (method, path, version, headers, amt, body_len) = {
        let mut headers = [httparse::EMPTY_HEADER; 8];
        let mut r = httparse::Request::new(&mut headers);
        let mut body_len: usize = 0;
        let mut header_vec = SmallVec::new();
        let status = try!(r.parse(buf).map_err(|e| {
            let msg = format!("failed to parse http request: {:?}", e);
            io::Error::new(io::ErrorKind::Other, msg)
        }));

        let amt = match status {
            httparse::Status::Complete(amt) => amt,
            httparse::Status::Partial => {
                return Ok(None)
            },
        };

        let toslice = |a: &[u8]| {
            let start = a.as_ptr() as usize - buf.as_ptr() as usize;
            assert!(start < buf.len());
            (start, start + a.len())
        };

        for header in r.headers.iter() {
            if header.name == httplib::header::CONTENT_LENGTH {
                let value = str::from_utf8(header.value).unwrap_or("0");
                body_len = value.parse::<usize>().unwrap_or(0);
            }

            header_vec.push((toslice(header.name.as_bytes()), toslice(header.value)));
        }

        println!("amt: {}", amt);
        println!("body_len: {}", body_len);

        (toslice(r.method.unwrap().as_bytes()),
         toslice(r.path.unwrap().as_bytes()),
         r.version.unwrap(),
         header_vec,
         amt,
         body_len)
    };

    Ok(Request {
        method: method,
        path: path,
        version: version,
        headers: headers,
        data: buf.split_to(amt + body_len),
        body: (amt, amt + body_len),
        params: HashMap::new(),
        query_params: HashMap::new()
    }.into())
}
