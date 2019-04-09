#[macro_use] extern crate thruster;
extern crate futures_legacy;

use futures_legacy::future;

use thruster::{App, BasicContext, Context, MiddlewareChain, MiddlewareReturnValue, Response};
use thruster::server::Server;
use thruster::thruster_context::basic_context::generate_context;
use thruster::ThrusterServer;

use std::str;


pub struct Ctx {
  pub body: String
}

impl Ctx {
  pub fn new(context: Ctx) -> Ctx {
    Ctx {
      body: context.body
    }
  }
}

impl Context for Ctx {
  type Response = Response;

  fn get_response(self) -> Response {
    let mut response = Response::new();

    response.status_code(200, "OK");
    response.body(&self.body);
    response
  }

  fn set_body(&mut self, body: Vec<u8>) {
    self.body = str::from_utf8(&body).unwrap_or("").to_owned();
  }
}

impl From<BasicContext> for Ctx {
  fn from(basic: BasicContext) -> Ctx {
    Ctx {
      body: basic.get_body()
    }
  }
}

impl Into<BasicContext> for Ctx {
  fn into(self: Self) -> BasicContext {
    let mut b = BasicContext::new();

    b.body(&self.body);

    b
  }
}

fn log(context: BasicContext, next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> MiddlewareReturnValue<BasicContext> {
  println!("[{}] {}", context.request.method(), context.request.path());

  next(context)
}

fn goodbye(mut context: Ctx, _next: impl Fn(Ctx) -> MiddlewareReturnValue<Ctx>  + Send + Sync) -> MiddlewareReturnValue<Ctx> {
  context.body = "Goodbye, world!".to_owned();

  Box::new(future::ok(context))
}

fn main() {
  println!("Starting server...");

  let mut app = App::create(generate_context);

  app.get("/hello", middleware![BasicContext => log, Ctx => goodbye]);

  let server = Server::new(app);
  server.start("0.0.0.0", 4321);
}
