extern crate env_logger;
extern crate futures;
extern crate fanta;
extern crate tokio_proto;
extern crate tokio_service;
extern crate time;

#[macro_use]
extern crate lazy_static;

use fanta::{App, Context, MiddlewareChain, Request, Response};

/**
 * Here we make a custom context. Contexts are the way that
 * Fanta passes data between middleware functions. Typically
 * this is where you'd add fields such as the currently
 * signed in user, or tracking sessions, etc.
 */

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

/**
 * The only important part about Contexts is that they
 * implement the Context trait. The Context trait only
 * requires a single method, get_response.
 *
 * get_response should create a Response object from
 * the current context. This is where you would hydrate
 * headers, body, and anything else custom that belongs
 * on the response.
 */

impl Context for CustomContext {
  fn get_response(&self) -> Response {
    let mut response = Response::new();
    response.body(&self.body);
    response.header("Content-Type", "text/plain");

    response
  }
}

/**
 * The generate_context function is pretty self explanatory.
 * It's the function that's passed in to the app that gets
 * called every time a new request into the app is made. It
 * takes the request and transforms it into Context.
 */

fn generate_context(request: &Request) -> CustomContext {
  CustomContext {
    body: "".to_owned(),
    method: request.method().to_owned(),
    path: request.path().to_owned()
  }
}

/**
 * The app object _must_ have a static lifetime. This is
 * just a way to generate static lifetime apps.
 */

lazy_static! {
  static ref APP: App<CustomContext> = {
    let mut _app = App::<CustomContext>::create(generate_context);

    /**
     * We have a single route with the root (/) path. It
     * then takes a vector of middleware to be run whenever
     * a request is made to that path.
     */
    _app.get("/", vec![index]);

    _app
  };
}

/**
 * From above, this is the single function that's executed
 * when the `/` path is hit. Middleware functions take a
 * a Context, a MiddlewareChain (explained below,) and return
 * a Context.
 *
 * A MiddlewareChain is Fanta's way of representing the call
 * stack of middleware functions. It has a single method,
 * `next` which calls the next function in the middleawre
 * chain, which returns a new context.
 *
 * If chain.next() is not called, the chain simply does not
 * continue any further and the call stack returns back up.
 */

fn index(context: CustomContext, _chain: &MiddlewareChain<CustomContext>) -> CustomContext {
  let mut context = CustomContext::new(context);

  context.body = "Hello world".to_owned();

  context
}

/**
 * Start the app!
 */

fn main() {
  println!("Starting server...");

  drop(env_logger::init());
  App::start(&APP, "8080".to_string());
}
