use response::Response;

pub trait Context {
  fn get_response(&self) -> Response;
  fn set_body(&mut self, String);
}

pub struct BasicContext {
  pub body: String
}

impl BasicContext {
  pub fn new() -> BasicContext {
    BasicContext {
      body: "".to_owned()
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
