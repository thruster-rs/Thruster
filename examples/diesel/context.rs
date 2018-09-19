use std::collections::{HashMap};
use smallvec::SmallVec;

use hyper::{Body, Response, Request, StatusCode};
use http::response::Builder;
use thruster::{Context};

pub struct Ctx {
  pub body: String,
  pub method: String,
  pub path: String,
  pub request_body: Body,
  pub params: HashMap<String, String>,
  pub headers: SmallVec<[(String, String); 8]>,
  pub status_code: u16,
  pub response: Builder
}

impl Ctx {
  pub fn set_header(&mut self, key: &str, val: &str) {
    self.response.header(key, val);
  }
}

impl Context for Ctx {
  fn get_response(self) -> Response<Body> {
    let mut response_builder = Response::builder();
    response_builder.status(StatusCode::from_u16(self.status_code).unwrap());
    response_builder.body(Body::from(self.body)).unwrap()
  }

  fn set_body(&mut self, body: String) {
    self.body = body;
  }

  fn set_params(&mut self, params: HashMap<String, String>) {
    self.params = params;
  }
}

pub fn generate_context(request: Request<Body>) -> Ctx {
  let method = request.method().to_string();
  let path = request.uri().to_string();

  Ctx {
    body: "".to_owned(),
    method: method,
    path: path,
    params: HashMap::new(),
    request_body: request.into_body(),
    headers: SmallVec::new(),
    status_code: 200,
    response: Response::builder()
  }
}
