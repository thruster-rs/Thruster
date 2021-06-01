use bytes::Bytes;
use http::header::{HeaderName, HeaderValue, SERVER};
use hyper::{Body, Response, StatusCode};
use std::str;

pub use crate::context::hyper_request::HyperRequest;
use crate::core::context::Context;

pub fn generate_context<S>(request: HyperRequest, _state: &S, _path: &str) -> FastHyperContext {
    FastHyperContext::new(request)
}

pub fn fast_generate_context<S>(
    request: HyperRequest,
    _state: &S,
    _path: &str,
) -> FastHyperContext {
    FastHyperContext {
        body: None,
        status: 200,
        hyper_request: Some(request),
        http_version: hyper::Version::HTTP_11,
    }
}

#[derive(Default)]
pub struct FastHyperContext {
    pub body: Option<Body>,
    pub status: u16,
    pub hyper_request: Option<HyperRequest>,
    pub http_version: hyper::Version,
}

impl Clone for FastHyperContext {
    fn clone(&self) -> Self {
        warn!("You should not be calling this method -- it just returns a default context.");
        FastHyperContext::default()
    }
}

const SERVER_HEADER_NAME: HeaderName = SERVER;
impl FastHyperContext {
    pub fn new(req: HyperRequest) -> FastHyperContext {
        FastHyperContext {
            body: None,
            status: 200,
            hyper_request: Some(req),
            http_version: hyper::Version::HTTP_11,
        }
    }
}

impl Context for FastHyperContext {
    type Response = Response<Body>;

    fn get_response(self) -> Self::Response {
        let mut response = Response::new(self.body.unwrap());

        *response.status_mut() = StatusCode::from_u16(self.status).unwrap();
        *response.version_mut() = self.http_version;
        *response.headers_mut() = self.hyper_request.unwrap().request.into_parts().0.headers;
        response
            .headers_mut()
            .insert(SERVER_HEADER_NAME, HeaderValue::from_static("thruster"));

        response
    }

    fn set_body(&mut self, body: Vec<u8>) {
        self.body = Some(Body::from(body));
    }

    fn set_body_bytes(&mut self, bytes: Bytes) {
        self.body = Some(Body::from(bytes));
    }

    fn route(&self) -> &str {
        let uri = self.hyper_request.as_ref().unwrap().request.uri();

        match uri.path_and_query() {
            Some(val) => val.as_str(),
            None => uri.path(),
        }
    }

    fn set(&mut self, key: &str, value: &str) {
        self.hyper_request
            .as_mut()
            .unwrap()
            .request
            .headers_mut()
            .insert(
                HeaderName::from_bytes(key.as_bytes()).unwrap(),
                HeaderValue::from_str(value).unwrap(),
            );
    }

    fn remove(&mut self, key: &str) {
        self.hyper_request
            .as_mut()
            .unwrap()
            .request
            .headers_mut()
            .remove(key);
    }
}
