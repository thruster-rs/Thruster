use http::request::Parts;
use hyper::{Body, Request};
use std::collections::HashMap;
use std::net::IpAddr;

use crate::core::request::RequestWithParams;

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

impl RequestWithParams for HyperRequest {
    fn set_params(&mut self, params: Option<HashMap<String, String>>) {
        self.params = params;
    }
}

impl Default for HyperRequest {
    fn default() -> Self {
        HyperRequest::new(Request::default())
    }
}
