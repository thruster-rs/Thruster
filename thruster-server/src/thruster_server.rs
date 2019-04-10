use thruster_app::app::App;
use thruster_core::context::Context;
use thruster_core::request::RequestWithParams;

pub trait ThrusterServer {
  type Context: Context + Send;
  type Response: Send;
  type Request: RequestWithParams + Send;

  fn new(_: App<Self::Request, Self::Context>) -> Self;
  fn start(self, host: &str, port: u16);
}
