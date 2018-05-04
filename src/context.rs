use std::collections::HashMap;
use httplib::Response;

pub trait Context {
  fn get_response(&self) -> Response<String>;
  fn set_body(&mut self, String);
}

pub struct BasicContext {
  pub body: String,
  pub params: HashMap<String, String>,
  pub query_params: HashMap<String, String>
}

impl BasicContext {
  pub fn new() -> BasicContext {
    BasicContext {
      body: "".to_owned(),
      params: HashMap::new(),
      query_params: HashMap::new()
    }
  }
}

impl Context for BasicContext {
  fn get_response(&self) -> Response<String> {
    let mut response = Response::builder();

    response.body(self.body.clone()).unwrap()
  }

  fn set_body(&mut self, body: String) {
    self.body = body;
  }
}
