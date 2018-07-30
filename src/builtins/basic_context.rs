use std::collections::HashMap;
use smallvec::SmallVec;
use bytes::BytesMut;

use context::Context;
use response::Response;
use request::Request;
use builtins::retains_request::RetainsRequest;
use builtins::query_params::HasQueryParams;

pub fn generate_context(request: Request) -> BasicContext {
  BasicContext {
    body: "".to_owned(),
    params: request.params().clone(),
    query_params: request.query_params().clone(),
    request: request
  }
}

pub struct BasicContext {
  pub body: String,
  pub params: HashMap<String, String>,
  pub query_params: HashMap<String, String>,
  pub request: Request
}

impl BasicContext {
  pub fn new() -> BasicContext {
    BasicContext {
      body: "".to_owned(),
      params: HashMap::new(),
      query_params: HashMap::new(),
      request: Request::new()
    }
  }
}

impl Context for BasicContext {
  fn get_response(self) -> Response {
    let mut response = Response::new();

    response.body(&self.body);

    response
  }

  fn set_body(&mut self, body: String) {
    self.body = body;
  }
}

impl RetainsRequest for BasicContext {
  fn get_request<'a>(&'a self) -> &'a Request {
    &self.request
  }
}

impl HasQueryParams for BasicContext {
  fn set_query_params(&mut self, query_params: HashMap<String, String>) {
    self.query_params = query_params;
  }
}
