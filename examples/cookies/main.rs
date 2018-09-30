extern crate thruster;
extern crate futures;

use std::boxed::Box;
use futures::future;

use thruster::{App, BasicContext as Ctx, CookieOptions, MiddlewareChain, MiddlewareReturnValue, Request};
use thruster::builtins::server::Server;
use thruster::server::ThrusterServer;

fn plaintext(mut context: Ctx, _chain: &MiddlewareChain<Ctx>) -> MiddlewareReturnValue<Ctx> {
  let val = "Hello, World!".to_owned();
  context.body = val;
  context.cookie("SomeCookie", "Some Value!", &CookieOptions::default());

  Box::new(future::ok(context))
}

fn redirect(mut context: Ctx, _chain: &MiddlewareChain<Ctx>) -> MiddlewareReturnValue<Ctx> {
  context.redirect("/plaintext");

  Box::new(future::ok(context))
}

fn main() {
  println!("Starting server...");

  let mut app = App::<Request, Ctx>::new();

  app.get("/plaintext", vec![plaintext]);
  app.get("*", vec![redirect]);

  let server = Server::new(app);
  server.start("0.0.0.0", 4321);
}
