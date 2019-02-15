use crate::app::App;
use crate::context::Context;
use crate::request::RequestWithParams;

pub trait ThrusterServer {
  type Context: Context + Send;
  type Response: Send;
  type Request: RequestWithParams + Send;

  fn new(_: App<Self::Request, Self::Context>) -> Self;
  fn start(self, host: &str, port: u16);
}
