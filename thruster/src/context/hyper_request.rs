use http::request::Parts;
use hyper::{Body, Request};
use std::collections::HashMap;
use std::net::IpAddr;

// use crate::core::request::RequestWithParams;
// use crate::parser::tree::Params;

pub struct HyperRequest {
    pub request: Request<Body>,
    pub parts: Option<Parts>,
    pub body: Option<Body>,
    pub params: Option<HashMap<String, String>>,
    pub ip: Option<IpAddr>,
}

impl HyperRequest {
    pub fn new(request: Request<Body>) -> HyperRequest {
        HyperRequest {
            request,
            parts: None,
            body: None,
            params: None,
            ip: None,
        }
    }
}

impl Default for HyperRequest {
    fn default() -> Self {
        HyperRequest::new(Request::default())
    }
}
