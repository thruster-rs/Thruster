#[macro_use] extern crate thruster;
extern crate futures_legacy;
extern crate serde_json;

use std::boxed::Box;
use futures_legacy::future;
use thruster::{App, BasicContext as Ctx, MiddlewareChain, MiddlewareReturnValue, Request};
use thruster::builtins::server::Server;
use thruster::builtins::query_params::query_params;
use thruster::server::ThrusterServer;

fn plaintext(mut context: Ctx, _next: impl Fn(Ctx) -> MiddlewareReturnValue<Ctx>  + Send + Sync) -> MiddlewareReturnValue<Ctx> {
  let val = serde_json::to_string(&context.query_params).unwrap();
  context.body(&val);

  Box::new(future::ok(context))
}

fn main() {
  println!("Starting server...");

  let mut app = App::<Request, Ctx>::new_basic();

  app.use_middleware("/", middleware![Ctx => query_params]);
  app.get("/plaintext", middleware![Ctx => plaintext]);

  let server = Server::new(app);
  server.start("0.0.0.0", 4321);
}
