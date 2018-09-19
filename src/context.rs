use std::collections::HashMap;
use hyper::{Body, Response};

/// A `Context` is what will be passed between functions in the middleware for
/// the defined routes of Thruster. Since a new context is made for each
/// incomming request, it's important to keep this struct lean and quick, as
/// well as the `context_generator` associated with it.
pub trait Context {
  fn get_response(self) -> Response<Body>;
  fn set_body(&mut self, String);
  fn set_params(&mut self, HashMap<String, String>);
}
