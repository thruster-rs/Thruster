use response::Response;

pub trait Context {
  fn get_body(&self) -> String;
  fn get_response(&self) -> Response;
}

pub struct BasicContext {
  pub body: String,
  pub _identifier: u32
}

impl BasicContext {
  pub fn new() -> BasicContext {
    BasicContext {
      body: "".to_owned(),
      _identifier: 0
    }
  }
}

impl Context for BasicContext {
  fn get_body(&self) -> String {
    self.body.clone()
  }

  fn get_response(&self) -> Response {
    let mut response = Response::new();
    response.body(&self.body);

    response
  }
}
