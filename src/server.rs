use app::App;
use context::Context;
use request::RequestWithParams;

pub trait ThrusterServer {
  type Context: Context + Send;
  type Response: Send;
  type Request: RequestWithParams + Send;

  fn new(App<Self::Request, Self::Context>) -> Self;
  fn start(self, host: &str, port: u16);
}
