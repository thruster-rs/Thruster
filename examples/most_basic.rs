extern crate thruster;
extern crate futures;
extern crate serde;
extern crate serde_json;
extern crate tokio;

use std::boxed::Box;
use futures::future;

use thruster::{App, BasicContext as Ctx, MiddlewareChain, MiddlewareReturnValue, Request};

fn generate_context(request: &Request) -> Ctx {
  Ctx {
    body: "".to_owned(),
    params: request.params().clone(),
    query_params: request.query_params().clone()
  }
}

fn plaintext(mut context: Ctx, _chain: &MiddlewareChain<Ctx>) -> MiddlewareReturnValue<Ctx> {
  let val = "Hello, World!".to_owned();
  context.body = val;

  Box::new(future::ok(context))
}

fn main() {
  println!("Starting server...");

  let mut app = App::<Ctx>::create(generate_context);

  app.get("/plaintext", vec![plaintext]);

  App::start(app, "0.0.0.0", 4321);
}
