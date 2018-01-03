# Fanta
## An opinionated framework for web development in rust.

[![Build Status](https://travis-ci.org/trezm/Fanta.svg?branch=master)](https://travis-ci.org/trezm/Fanta)

Fanta is a web framework that aims for developers to be productive and consistent across projects and teams. Its goals are to be:
- Opinionated
- Fast
- Intuitive

Based heavily off of the work here: https://github.com/tokio-rs/tokio-minihttp

## WARNING -- FANTA IS NOT READY, AND PROBABLY SHOULDN'T EVEN BE ON GITHUB YET

## Opinionated

This is a work in progress

## Fast

Initial tests of plain-text requests yeilds the following resuts:

```
Framework: Cowboy
Requests/sec:  10982.93
Transfer/sec:      1.36MB
Framework: Node (Koa)
Requests/sec:   9592.82
Transfer/sec:      1.39MB
Framework: Node (Express)
Requests/sec:   5926.87
Transfer/sec:      1.22MB
Framework: Phoenix/Elixir (prod mode)
Requests/sec:    538.02
Transfer/sec:    132.93KB
Framework: Rocket (dev mode)
Requests/sec:   5537.87
Transfer/sec:    789.58KB
Framework: Rocket (prod mode)
Requests/sec:  18555.74
Transfer/sec:      2.58MB
Framework: Fanta (dev mode)
Requests/sec:   7362.82
Transfer/sec:    726.22KB
Framework: Fanta (prod mode)
Requests/sec:  39865.62
Transfer/sec:      3.84MB
```

## Intuitive

Left to the developer to decide at this point!

## Getting Started

To get started, check out the example app:

``` Rust
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

  fn set_body(&mut self, body: String) {
    self.body = body;
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
```
