use bytes::BytesMut;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::{fmt, io, str};

pub trait RequestWithParams {
    fn set_params(&mut self, _: Option<HashMap<String, String>>);
}

///
/// The request object is the default request object provied by Thruster. If a different
/// server is used, such as Hyper, then you'll need to reference that server's "request"
/// documentation instead.
///
#[derive(Default)]
pub struct Request {
    body: Slice,
    method: Slice,
    path: Slice,
    version: u8,
    pub headers: SmallVec<[(Slice, Slice); 8]>,
    data: BytesMut,
    pub params: Option<HashMap<String, String>>,
}

type Slice = (usize, usize);

impl Request {
    ///
    /// Create a new, blank, request.
    ///
    pub fn new() -> Self {
        Request {
            body: (0, 0),
            method: (0, 0),
            path: (0, 0),
            version: 0,
            headers: SmallVec::new(),
            data: BytesMut::new(),
            params: None,
        }
    }

    ///
    /// Get the raw pointer to the byte array of the request
    ///
    pub fn raw_body(&self) -> &[u8] {
        self.slice(&self.body)
    }

    ///
    /// Get the body as a utf8 encoded string
    ///
    pub fn body(&self) -> &str {
        str::from_utf8(self.slice(&self.body)).unwrap()
    }

    ///
    /// Get the method as a string
    ///
    pub fn method(&self) -> &str {
        str::from_utf8(self.slice(&self.method)).unwrap()
    }

    ///
    /// Get the path as a string ("/some/path")
    ///
    pub fn path(&self) -> &str {
        str::from_utf8(self.slice(&self.path)).unwrap()
    }

    ///
    /// Get the HTTP version
    ///
    pub fn version(&self) -> u8 {
        self.version
    }

    ///
    /// Get an HashMap of the provided headers. The HashMap is lazily computed, so
    /// avoid this method unless you need to access headers.
    ///
    pub fn headers(&self) -> HashMap<String, String> {
        let mut header_map = HashMap::new();

        for slice_pair in self.headers.iter() {
            header_map.insert(
                str::from_utf8(self.slice(&slice_pair.0))
                    .unwrap()
                    .to_owned()
                    .to_lowercase(),
                str::from_utf8(self.slice(&slice_pair.1))
                    .unwrap()
                    .to_owned(),
            );
        }

        header_map
    }

    ///
    /// Automatically apply a serde deserialization to the body
    ///
    pub fn body_as<'a, T>(&self, body: &'a str) -> serde_json::Result<T>
    where
        T: serde::de::Deserialize<'a>,
    {
        serde_json::from_str(body)
    }

    ///
    /// Fetch a piece of the raw body
    ///
    fn slice(&self, slice: &Slice) -> &[u8] {
        &self.data[slice.0..slice.1]
    }

    ///
    /// Fetch the params from the route, e.g. The route "/some/:key" when applied to an incoming
    /// request for "/some/value" will populate `params` with `{ key: "value" }`
    ///
    pub fn params(&self) -> &Option<HashMap<String, String>> {
        &self.params
    }
}

impl RequestWithParams for Request {
    fn set_params(&mut self, params: Option<HashMap<String, String>>) {
        self.params = params;
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
        let mut headers = [httparse::EMPTY_HEADER; 32];
        let mut r = httparse::Request::new(&mut headers);
        let mut body_len: usize = 0;
        let mut header_vec = SmallVec::new();
        let status = r.parse(buf).map_err(|e| {
            let msg = format!("failed to parse http request: {:?}", e);
            eprintln!("msg: {}", msg);
            eprintln!("dump: {:#?}", buf);
            io::Error::new(io::ErrorKind::Other, msg)
        })?;
        let amt = match status {
            httparse::Status::Complete(amt) => amt,
            httparse::Status::Partial => return Ok(None),
        };
        let toslice = |a: &[u8]| {
            let start = a.as_ptr() as usize - buf.as_ptr() as usize;
            assert!(start < buf.len());
            (start, start + a.len())
        };
        for header in r.headers.iter() {
            let header_name = httplib::header::CONTENT_LENGTH;

            if header.name == header_name {
                let value = str::from_utf8(header.value).unwrap_or("0");
                body_len = value.parse::<usize>().unwrap_or(0);
            }

            header_vec.push((toslice(header.name.as_bytes()), toslice(header.value)));
        }

        (
            toslice(r.method.unwrap().as_bytes()),
            toslice(r.path.unwrap().as_bytes()),
            r.version.unwrap(),
            header_vec,
            amt,
            body_len,
        )
    };

    if amt + body_len != buf.len() {
        Ok(None)
    } else {
        Ok(Request {
            method,
            path,
            version,
            headers,
            data: buf.split_to(amt + body_len),
            body: (amt, amt + body_len),
            params: None,
        }
        .into())
    }
}
