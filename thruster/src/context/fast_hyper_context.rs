use bytes::Bytes;
use http::header::{HeaderMap, HeaderName, HeaderValue, SERVER};
use hyper::{Body, Response, StatusCode};
use std::str;

pub use crate::context::hyper_request::HyperRequest;
use crate::core::context::Context;

pub fn generate_context<S>(request: HyperRequest, _state: &S, _path: &str) -> FastHyperContext {
    FastHyperContext::new(request)
}

#[derive(Default)]
pub struct FastHyperContext {
    pub body: Body,
    pub status: u16,
    pub hyper_request: Option<HyperRequest>,
    pub http_version: hyper::Version,
    headers: HeaderMap,
}

const SERVER_HEADER_NAME: HeaderName = SERVER;
impl FastHyperContext {
    pub fn new(req: HyperRequest) -> FastHyperContext {
        let mut headers = HeaderMap::new();
        headers.insert(SERVER_HEADER_NAME, HeaderValue::from_static("thruster"));

        FastHyperContext {
            body: Body::empty(),
            status: 200,
            hyper_request: Some(req),
            http_version: hyper::Version::HTTP_11,
            headers,
        }
    }
}

impl Context for FastHyperContext {
    type Response = Response<Body>;

    fn get_response(self) -> Self::Response {
        let mut response = Response::new(self.body);

        *response.status_mut() = StatusCode::from_u16(self.status).unwrap();
        *response.headers_mut() = self.headers;
        *response.version_mut() = self.http_version;

        response
    }

    fn set_body(&mut self, body: Vec<u8>) {
        self.body = Body::from(body);
    }

    fn set_body_bytes(&mut self, bytes: Bytes) {
        self.body = Body::from(bytes);
    }

    fn route(&self) -> &str {
        let uri = self.hyper_request.as_ref().unwrap().request.uri();

        match uri.path_and_query() {
            Some(val) => val.as_str(),
            None => uri.path(),
        }
    }

    fn set(&mut self, key: &str, value: &str) {
        self.headers.insert(
            HeaderName::from_bytes(key.as_bytes()).unwrap(),
            HeaderValue::from_str(value).unwrap(),
        );
    }

    fn remove(&mut self, key: &str) {
        self.headers.remove(key);
    }
}
