#[macro_use] extern crate thruster;
extern crate futures;

use std::boxed::Box;
use futures::future;

use thruster::{App, BasicContext as Ctx, MiddlewareChain, MiddlewareReturnValue, Request};
use thruster::thruster_middleware::cookies::CookieOptions;
use thruster::server::Server;
use thruster::ThrusterServer;

fn plaintext(mut context: Ctx, _next: impl Fn(Ctx) -> MiddlewareReturnValue<Ctx>  + Send + Sync) -> MiddlewareReturnValue<Ctx> {
  let val = "Hello, World!";
  context.body(val);
  context.cookie("SomeCookie", "Some Value!", &CookieOptions::default());

  Box::new(future::ok(context))
}

fn redirect(mut context: Ctx, _next: impl Fn(Ctx) -> MiddlewareReturnValue<Ctx>  + Send + Sync) -> MiddlewareReturnValue<Ctx> {
  context.redirect("/plaintext");

  Box::new(future::ok(context))
}

fn main() {
  println!("Starting server...");

  let mut app = App::<Request, Ctx>::new_basic();

  app.get("/plaintext", middleware![Ctx => plaintext]);
  app.get("*", middleware![Ctx => redirect]);

  let server = Server::new(app);
  server.start("0.0.0.0", 4321);
}
