#[macro_use] extern crate thruster;
extern crate futures_legacy;

use std::boxed::Box;
use futures_legacy::future;

#[macro_use(middleware)]
use thruster::{App, BasicContext as Ctx, MiddlewareChain, MiddlewareReturnValue, Request};
use thruster::server::Server;
use thruster::ThrusterServer;

fn plaintext(mut context: Ctx, _next: impl Fn(Ctx) -> MiddlewareReturnValue<Ctx>  + Send + Sync) -> MiddlewareReturnValue<Ctx> {
  let val = "Hello, World!";
  context.body(val);

  Box::new(future::ok(context))
}

fn main() {
  println!("Starting server...");

  let mut app = App::<Request, Ctx>::new_basic();

  app.get("/plaintext", middleware![Ctx => plaintext]);

  let server = Server::new(app);
  server.start_small_load_optimized("0.0.0.0", 4321);
}
