/// A `Context` is what will be passed between functions in the middleware for
/// the defined routes of Thruster. Since a new context is made for each
/// incomming request, it's important to keep this struct lean and quick, as
/// well as the `context_generator` associated with it.
pub trait Context {
  type Response: Send;

  fn get_response(self) -> Self::Response;
  fn set_body(&mut self, String);
}
