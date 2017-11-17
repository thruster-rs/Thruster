extern crate env_logger;
extern crate futures;
extern crate fanta;
extern crate tokio_proto;
extern crate tokio_service;
extern crate time;

#[macro_use]
extern crate lazy_static;

use fanta::{App, Context, MiddlewareChain, Request, Response};

struct CustomContext {
  pub body: String,
  pub method: String,
  pub path: String
}

impl CustomContext {
  fn new(context: CustomContext) -> CustomContext {
    CustomContext {
      body: context.body,
      method: context.method,
      path: context.path
    }
  }
}

impl Context for CustomContext {
  fn get_body(&self) -> String {
    self.body.clone()
  }

  fn get_response(&self) -> Response {
    let mut response = Response::new();
    response.body(&self.body);
    response.header("Content-Type", "text/plain");

    response
  }
}

fn generate_context(request: &Request) -> CustomContext {
  CustomContext {
    body: "".to_owned(),
    method: request.method().to_owned(),
    path: request.path().to_owned()
  }
}

lazy_static! {
  static ref APP: App<CustomContext> = {
    let mut _app = App::<CustomContext>::create(generate_context);

    _app.get("/index.html", vec![index]);

    _app.get("/test/route", vec![index]);
    _app
  };
}

fn index(context: CustomContext, _chain: &MiddlewareChain<CustomContext>) -> CustomContext {
  let mut context = CustomContext::new(context);

  context.body = "Hello world".to_owned();

  context
}

fn main() {
  println!("Starting server...");

  drop(env_logger::init());
  App::start(&APP, "8080".to_string());
}
