use std::collections::HashMap;
use response::Response;

/// A `Context` is what will be passed between functions in the middleware for
/// the defined routes of Thruster. Since a new context is made for each
/// incomming request, it's important to keep this struct lean and quick, as
/// well as the `context_generator` associated with it.
pub trait Context {
  fn get_response(&self) -> Response;
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
  fn get_response(&self) -> Response {
    let mut response = Response::new();

    response.body(&self.body);

    response
  }

  fn set_body(&mut self, body: String) {
    self.body = body;
  }
}
