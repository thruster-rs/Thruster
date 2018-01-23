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
    response.header("Content-Type", "text/plain");

    response
  }

  fn set_body(&mut self, body: String) {
    self.body = body;
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
    _app.get("/", vec![index]);

    let mut subroute = App::<CustomContext>::create(generate_context);
    subroute.get("/", vec![sub_route]);
    subroute.get("/hello", vec![sub_route]);

    _app.use_sub_app("/sub", &subroute);

    _app
  };
}

fn index(context: CustomContext, _chain: &MiddlewareChain<CustomContext>) -> CustomContext {
  let mut context = CustomContext::new(context);

  context.body = "Hello world".to_owned();

  context
}

fn sub_route(context: CustomContext, _chain: &MiddlewareChain<CustomContext>) -> CustomContext {
  let mut context = CustomContext::new(context);

  context.body = "subroute".to_owned();

  context
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

fn main() {
  println!("Starting server...");

  drop(env_logger::init());
  App::start(&APP, "0.0.0.0".to_string(), "8080".to_string());
}
