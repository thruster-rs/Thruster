# Thruster
## An opinionated framework for web development in rust.

[![Build Status](https://travis-ci.org/trezm/Thruster.svg?branch=master)](https://travis-ci.org/trezm/Thruster)
[![Gitter chat](https://img.shields.io/gitter/room/thruster-rs/thruster.svg)](https://gitter.im/thruster-rs/Lobby)

[Documentation](https://docs.rs/thruster/0.4.5/thruster/)

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

## Getting Started

If you have `cargo generate` installed, you can simply run the [cargo generator](https://github.com/ami44/thruster-basic-template)

```
cargo generate --git https://github.com/ami44/thruster-basic-template.git --name myproject
```

### The most basic example

```rust
extern crate thruster;
extern crate futures;

use std::boxed::Box;
use futures::future;

use thruster::{App, BasicContext as Ctx, MiddlewareChain, MiddlewareReturnValue, Request};
use thruster::builtins::server::Server;
use thruster::server::ThrusterServer;

fn plaintext(mut context: Ctx, next: impl Fn(Ctx) -> MiddlewareReturnValue<Ctx>  + Send + Sync) -> MiddlewareReturnValue<Ctx> {
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

fn plaintext(mut context: Ctx, next: impl Fn(Ctx) -> MiddlewareReturnValue<Ctx>  + Send + Sync) -> MiddlewareReturnValue<Ctx> {
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
