extern crate env_logger;
extern crate futures;
extern crate fanta;
extern crate tokio_proto;
extern crate tokio_service;
extern crate time;

#[macro_use]
extern crate lazy_static;

use fanta::{App, Context, MiddlewareChain, Request, Response};
use time::Duration;

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
  fn get_response(&self) -> Response {
    let mut response = Response::new();
    response.body(&self.body);

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

    _app.use_middleware("/", profiling);

    _app.get("/index.html", vec![index]);

    _app.get("/test/route", vec![index]);
    _app
  };
}

fn profiling(context: CustomContext, chain: &MiddlewareChain<CustomContext>) -> CustomContext {
  let start_time = time::now();

  let context = chain.next(context);

  let elapsed_time: Duration = time::now() - start_time;
  println!("[{}ms] {} -- {}",
    elapsed_time.num_microseconds().unwrap(),
    context.method.clone(),
    context.path.clone());

  context
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
