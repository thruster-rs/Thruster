extern crate fanta;
extern crate futures;
extern crate serde;
extern crate serde_json;
extern crate tokio_proto;
extern crate tokio_service;

#[macro_use] extern crate lazy_static;

use std::boxed::Box;
use futures::{future, Future};
use std::io;

use fanta::{App, BasicContext as Ctx, MiddlewareChain, Request};

lazy_static! {
  static ref APP: App<Ctx> = {
    let mut _app = App::<Ctx>::create(generate_context);

    _app.get("/plaintext", vec![plaintext]);

    _app
  };
}

fn generate_context(request: &Request) -> Ctx {
  Ctx {
    body: "".to_owned(),
    params: request.params().clone()
  }
}

fn plaintext(mut context: Ctx, _chain: &MiddlewareChain<Ctx>) -> Box<Future<Item=Ctx, Error=io::Error>> {
  let val = "Hello, World!".to_owned();
  context.body = val;

  Box::new(future::ok(context))
}

fn main() {
  println!("Starting server...");

  App::start(&APP, "0.0.0.0".to_string(), "4321".to_string());
}
