#[macro_use] extern crate thruster;
extern crate futures_legacy;
extern crate hyper;

use std::boxed::Box;
use futures_legacy::future;

use thruster::{App, Context, MiddlewareChain, MiddlewareReturnValue};
use thruster::server::{HyperServer, ThrusterServer};
use thruster::thruster_context::basic_hyper_context::{generate_context, BasicHyperContext as Ctx, HyperRequest};

fn plaintext(mut context: Ctx, _next: impl Fn(Ctx) -> MiddlewareReturnValue<Ctx>  + Send + Sync) -> MiddlewareReturnValue<Ctx> {
  let val = "Hello, World!".as_bytes().to_vec();
  context.set_body(val);

  Box::new(future::ok(context))
}

fn main() {
  println!("Starting server...");

  let mut app = App::<HyperRequest, Ctx>::create(generate_context);

  app.get("/plaintext", middleware![Ctx => plaintext]);

  let server = Server::new(app);
  server.start("0.0.0.0", 4321);
}
