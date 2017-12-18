use response::Response;
use request::Request;
use serde;

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

pub struct TypedContext<T> {
  pub request_body: T,
  pub body: String
}

impl<'a, T> TypedContext<T>
  where T: serde::de::Deserialize<'a> {
  pub fn new(request: &'a Request) -> TypedContext<T> {
    match request.body_as::<T>(request.raw_body()) {
      Ok(val) => TypedContext {
        body: "".to_owned(),
        request_body: val
      },
      Err(err) => panic!("Could not create context: {}", err)
    }
  }
}

impl<'a, T> Context for TypedContext<T> {
  fn get_response(&self) -> Response {
    let mut response = Response::new();
    response.body(&self.body);

    response
  }

  fn set_body(&mut self, body: String) {
    self.body = body;
  }
}
