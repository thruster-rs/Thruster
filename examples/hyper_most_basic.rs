#[macro_use] extern crate thruster;
extern crate futures;
extern crate hyper;

use std::boxed::Box;
use futures::future;

use hyper::{Body, Request};
use thruster::{App, MiddlewareChain, MiddlewareReturnValue};
use thruster::builtins::hyper_server::Server;
use thruster::builtins::basic_hyper_context::{generate_context, BasicHyperContext as Ctx};
use thruster::server::ThrusterServer;

fn plaintext(mut context: Ctx, _next: impl Fn(Ctx) -> MiddlewareReturnValue<Ctx>  + Send + Sync) -> MiddlewareReturnValue<Ctx> {
  let val = "Hello, World!".to_owned();
  context.body = val;

  Box::new(future::ok(context))
}

fn main() {
  println!("Starting server...");

  let mut app = App::<Request<Body>, Ctx>::create(generate_context);

  app.get("/plaintext", middleware![Ctx => plaintext]);

  let server = Server::new(app);
  server.start("0.0.0.0", 4321);
}
