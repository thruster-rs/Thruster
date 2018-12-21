#[macro_use] extern crate thruster;
extern crate futures;

use std::boxed::Box;
use futures::future;

use thruster::{testing, App, BasicContext as Ctx, MiddlewareChain, MiddlewareReturnValue, Request};

fn plaintext(mut context: Ctx, _next: impl Fn(Ctx) -> MiddlewareReturnValue<Ctx>  + Send + Sync) -> MiddlewareReturnValue<Ctx> {
  let val = "Hello, World!".to_owned();
  context.body = val;

  Box::new(future::ok(context))
}

fn main() {
  let mut app = App::<Request, Ctx>::new_basic();

  app.get("/plaintext", middleware![Ctx => plaintext]);

  let result = testing::get(&app, "/plaintext");

  assert!(result.body == "Hello, World!");
}
