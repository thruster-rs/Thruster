use http::request::Parts;
use hyper::{Body, Request};
use std::net::IpAddr;

use crate::parser::tree::Params;
use crate::RequestWithParams;

pub struct HyperRequest {
    pub request: Request<Body>,
    pub parts: Option<Parts>,
    pub body: Option<Body>,
    pub params: Params,
    pub ip: Option<IpAddr>,
}

impl HyperRequest {
    pub fn new(request: Request<Body>) -> HyperRequest {
        HyperRequest {
            request,
            parts: None,
            body: None,
            params: Params::default(),
            ip: None,
        }
    }
}

impl Default for HyperRequest {
    fn default() -> Self {
        HyperRequest::new(Request::default())
    }
}

impl RequestWithParams for HyperRequest {
    fn set_params(&mut self, params: Params) {
        self.params = params;
    }

    fn get_params<'a>(&'a self) -> &'a Params {
        &self.params
    }
}
