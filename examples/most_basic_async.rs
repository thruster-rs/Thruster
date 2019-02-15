// enable the await! macro, async support, and the new std::Futures api.
#![feature(await_macro, async_await, futures_api, proc_macro_hygiene)]
#[macro_use] extern crate thruster;
extern crate futures;
extern crate middleware_proc;

use std::boxed::Box;

use thruster::{App, BasicContext as Ctx, MiddlewareChain, MiddlewareReturnValue, Request, async_middleware};
use thruster::builtins::server::Server;
use thruster::server::ThrusterServer;

#[async_middleware]
async fn plaintext(mut context: Ctx, _next: impl Fn(Ctx) -> MiddlewareReturnValue<Ctx> + Send + Sync) -> Result<Ctx, io::Error> {
  let val = "Hello, World!";
  context.body(val);
  Ok(context)
}

fn main() {
  println!("Starting server...");

  let mut app = App::<Request, Ctx>::new_basic();

  app.get("/plaintext", middleware![Ctx => plaintext]);

  let server = Server::new(app);
  server.start("0.0.0.0", 4321);
}
