extern crate thruster;
extern crate futures;

use std::boxed::Box;
use futures::future;

use thruster::{testing, App, BasicContext as Ctx, MiddlewareChain, MiddlewareReturnValue, Request};

fn plaintext(mut context: Ctx, _chain: &MiddlewareChain<Ctx>) -> MiddlewareReturnValue<Ctx> {
  let val = "Hello, World!".to_owned();
  context.body = val;

  Box::new(future::ok(context))
}

fn main() {
  let mut app = App::<Request, Ctx>::new();

  app.get("/plaintext", vec![plaintext]);

  let result = testing::get(app, "/plaintext");

  assert!(result.body == "Hello, World!");
}
