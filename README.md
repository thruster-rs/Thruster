# Thruster [![Build Status](https://travis-ci.com/trezm/Thruster.svg?branch=master)](https://travis-ci.org/trezm/Thruster) ![Crates.io](https://img.shields.io/crates/v/thruster.svg) ![Crates.io](https://img.shields.io/crates/d/thruster.svg)  [![Gitter chat](https://img.shields.io/gitter/room/thruster-rs/thruster.svg)](https://gitter.im/thruster-rs/Lobby)
## An opinionated framework for web development in rust.



[Documentation](https://docs.rs/thruster)

## Motivation

Thruster is a web framework that aims for developers to be productive and consistent across projects and teams. Its goals are to be:
- Opinionated
- Fast
- Intuitive

Thruster also
- Does not use `unsafe`
- Works in stable rust

## Opinionated

thruster and thruster-cli strive to give a good way to do domain driven design. It's also designed to set you on the right path, but not obfuscate certain hard parts behind libraries. Made with science 🔭, not magic 🧙‍♂️.

## Fast

Using the following wrk command, here are the results in `hello_world` examples for various frameworks

```bash
wrk -t12 -c400 -d30s http://127.0.0.1:4321/plaintext
```

```
>>> Framework: Cowboy
Requests/sec:  14066.80
Transfer/sec:      1.75MB
>>> Framework: Phoenix/Elixir (prod mode)
Requests/sec:    531.22
Transfer/sec:    131.25KB
>>> Framework: Actix (prod mode)
Requests/sec:  48661.48
Transfer/sec:      6.03MB
>>> Framework: Hyper (prod mode)
Requests/sec:  52909.67
Transfer/sec:      4.44MB
>>> Framework: Thruster (prod mode)
Requests/sec:  53612.10
Transfer/sec:      7.57MB
```

## Intuitive

Based on frameworks like Koa, and Express, thruster aims to be a pleasure to develop with.

## Example

To run the example `cargo run --example <example-name>`.
For example, `cargo run --example hello_world` and open [http://localhost:4321/](http://localhost:4321/)

### The most basic example

```rust
extern crate thruster;
extern crate futures;

use std::boxed::Box;
use futures::future;

use thruster::{App, BasicContext as Ctx, MiddlewareChain, MiddlewareReturnValue, Request};
use thruster::builtins::server::Server;
use thruster::server::ThrusterServer;

fn plaintext(mut context: Ctx, next: impl Fn(Ctx) -> MiddlewareReturnValue<Ctx>  + Send) -> MiddlewareReturnValue<Ctx> {
  let val = "Hello, World!".to_owned();
  context.body = val;

  Box::new(future::ok(context))
}

fn main() {
  println!("Starting server...");

  let mut app = App::<Request, Ctx>::new_basic();

  app.get("/plaintext", middleware![Ctx => plaintext]);

  let server = Server::new(app);
  server.start("0.0.0.0", 4321);
}
```

### The most basic example with Hyper
```rust
extern crate thruster;
extern crate futures;
extern crate hyper;

use std::boxed::Box;
use futures::future;

use hyper::{Body, Request};
use thruster::{App, MiddlewareChain, MiddlewareReturnValue};
use thruster::builtins::hyper_server::Server;
use thruster::builtins::basic_hyper_context::{generate_context, BasicHyperContext as Ctx};
use thruster::server::ThrusterServer;

fn plaintext(mut context: Ctx, next: impl Fn(Ctx) -> MiddlewareReturnValue<Ctx>  + Send) -> MiddlewareReturnValue<Ctx> {
  let val = "Hello, World!".to_owned();
  context.body = val;

  Box::new(future::ok(context))
}

fn main() {
  println!("Starting server...");

  let mut app = App::<Request<Body>, Ctx>::create(generate_context);

  app.get("/plaintext", middleware![Ctx => plaintext]);

  let server = Server::new(app);
  server.start("0.0.0.0", 4321);
}
```

### Async/Await

Thruster also supports async/await on nightly! Once you've installed nightly (`rustup install nightly`,) you can use async await support in thruster by enabling the `thruster_async_await` feature,

```toml
thruster = { version = "0.7", features = ["thruster_async_await"] }
```

The core parts that make the new async await code work is designating middleware functions with the `#[middleware_fn]` attribute (which marks the middleware so that it's compatible with the stable futures version that thruster is built on,) and then the `async_middleware!` macro in the actual routes.

_Note:, for the short term, the argument style of this macro has changed from `middleware!`. It now is of the form `async_middleware!(<Context Type>, [<middleware_fn>, <middleware_fn>, ...])`._

A simple example for using async await is:

```rust
#![feature(async_await, proc_macro_hygiene)]
extern crate thruster;

use std::boxed::Box;
use std::pin::Pin;
use std::future::Future;
use std::time::Instant;

use thruster::{Chain, Middleware, MiddlewareChain, MiddlewareNext};
use thruster::{App, BasicContext as Ctx, Request};
use thruster::server::Server;
use thruster::ThrusterServer;
use thruster::thruster_proc::{async_middleware, middleware_fn};

#[middleware_fn]
async fn profile(context: Ctx, next: MiddlewareNext<Ctx>) -> Ctx {
  let start_time = Instant::now();

  context = next(context).await;

  let elapsed_time = start_time.elapsed();
  println!("[{}μs] {} -- {}",
    elapsed_time.as_micros(),
    context.request.method(),
    context.request.path());

  context
}

#[middleware_fn]
async fn plaintext(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> Ctx {
  let val = "Hello, World!";
  context.body(val);
  context
}

#[middleware_fn]
async fn four_oh_four(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> Ctx {
  context.status(404);
  context.body("Whoops! That route doesn't exist!");
  context
}

fn main() {
  println!("Starting server...");

  let mut app = App::<Request, Ctx>::new_basic();

  app.get("/plaintext", async_middleware!(Ctx, [profile, plaintext]));
  app.set404(async_middleware!(Ctx, [four_oh_four]));

  let server = Server::new(app);
  server.start("0.0.0.0", 4321);
}
```

### Async/Await with error handling

To turn on experimental error handling, pass the flag `thruster_error_handling`. This will enable the useage of the `map_try!` macro from the main package. This has the same function as `try!`, but with the ability to properly map the error in a way that the compiler knows that execution ends (so there's no movement issues with `context`.)

This ends up looking like:

```rust
#![feature(async_await, futures_api, proc_macro_hygiene)]
extern crate thruster;

use thruster::{MiddlewareNext, MiddlewareReturnValue, MiddlewareResult};
use thruster::{App, BasicContext as Ctx, Request, map_try};
use thruster::server::Server;
use thruster::errors::ThrusterError as Error;
use thruster::ThrusterServer;
use thruster::thruster_proc::{async_middleware, middleware_fn};

#[middleware_fn]
async fn plaintext(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
  let val = "Hello, World!";
  context.body(val);
  Ok(context)
}

#[middleware_fn]
async fn error(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
  let res = "Hello, world".parse::<u32>();
  let non_existent_param = map_try!(res, Err(_) => {
      Error {
        context,
        message: "Parsing failure!".to_string(),
        status: 400
      }
    }
  );

  context.body(&format!("{}", non_existent_param));

  Ok(context)
}

#[middleware_fn]
async fn json_error_handler(context: Ctx, next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
  let res = await!(next(context));

  let ctx = match res {
    Ok(val) => val,
    Err(e) => {
      let mut context = e.context;
      context.body(&format!("{{\"message\": \"{}\",\"success\":false}}", e.message));
      context.status(e.status);
      context
    }
  };

  Ok(ctx)
}

fn main() {
  println!("Starting server...");

  let mut app = App::<Request, Ctx>::new_basic();

  app.use_middleware("/", async_middleware!(Ctx, [json_error_handler]));

  app.get("/plaintext", async_middleware!(Ctx, [plaintext]));
  app.get("/error", async_middleware!(Ctx, [error]));

  let server = Server::new(app);
  server.start("0.0.0.0", 4321);
}

```

### Quick setup without a DB

The easiest way to get started is to just clone the [starter kit](https://github.com/trezm/thruster-starter-kit)

```bash
> git clone git@github.com:trezm/thruster-starter-kit.git
> cd thruster-starter-kit
> cargo run
```

The example provides a simple plaintext route, a route with JSON serialization, and the preferred way to organize sub routes using sub apps.

### Quick setup with postgres

The easiest way to get started with postgres is to install thruster-cli,

```bash
> cargo install thruster-cli
```

And then to run

```bash
> thruster-cli init MyAwesomeProject
> thruster-cli component Users
> thruster-cli migrate
```

Which will generate everything you need to get started! Note that this requires a running postgres connection and assumes the following connection string is valid:

```
postgres://postgres@localhost/<Your Project Name>
```

This is all configurable and none of it is hidden from the developer. It's like seeing the magic trick and learning how it's done! Check out the docs for [thruster-cli here](https://github.com/trezm/thruster-cli).

## Testing
Thruster provides an easy test suite to test your endpoints, simply include the `testing` module as below:

```rust
let mut app = App::<Request, Ctx>::new_basic();

...

app.get("/plaintext", middleware![Ctx => plaintext]);

...

let result = testing::get(app, "/plaintext");

assert!(result.body == "Hello, World!");
```

## Other, or Custom Backends

Thruster is capable of just providing the routing layer on top of a server of some sort, for example, in the Hyper snippet above. This can be applied broadly to any backend, as long as the server implements `ThrusterServer`.

```rs
pub trait ThrusterServer {
  type Context: Context + Send;
  type Response: Send;
  type Request: RequestWithParams + Send;

  fn new(App<Self::Request, Self::Context>) -> Self;
  fn start(self, host: &str, port: u16);
}
```

There needs to be:
- An easy way to new up a server.
- A function to start the server.

Within the `start` function, the server implementation should:
- Start up some sort of listener for connections
- Call `let matched = app.resolve_from_method_and_path(<some method>, <some path>);` (This is providing the actual routing.)
- Call `app.resolve(<incoming request>, matched)` (This runs the chained middleware.)

## Using cargo generate

_Note: This hasn't yet been update for the latest version of thruster_

If you have `cargo generate` installed, you can simply run the [cargo generator](https://github.com/ami44/thruster-basic-template)

```
cargo generate --git https://github.com/ami44/thruster-basic-template.git --name myproject
```
