# Thruster
## An opinionated framework for web development in rust.

[![Build Status](https://travis-ci.org/trezm/thruster.svg?branch=master)](https://travis-ci.org/trezm/thruster)

thruster is a web framework that aims for developers to be productive and consistent across projects and teams. Its goals are to be:
- Opinionated
- Fast
- Intuitive

## Opinionated

thruster and thruster-cli strive to give a good way to do domain driven design. It's also designed to set you on the right path, but not obfuscate certain hard parts behind libraries. Made with science 🔭, not magic 🧙‍♂️.

## Fast

Using the following command, we get roughly 96% of the speed of pure `tokio-minihttp` running in release mode.

```bash
wrk -t12 -c400 -d30s http://127.0.0.1:4321/plaintext
```

thruster results:
```
Running 30s test @ http://127.0.0.1:4321/plaintext
  12 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     6.45ms    1.09ms  18.11ms   75.96%
    Req/Sec     5.03k   502.49     7.75k    82.97%
  1802773 requests in 30.05s, 244.13MB read
  Socket errors: connect 0, read 238, write 0, timeout 0
Requests/sec:  60000.61
Transfer/sec:      8.13MB
```

tokio-minihttp
```
Running 30s test @ http://127.0.0.1:4321/plaintext
  12 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     6.30ms    2.14ms  52.22ms   88.68%
    Req/Sec     5.19k     1.03k   10.91k    77.92%
  1861702 requests in 30.05s, 229.03MB read
  Socket errors: connect 0, read 243, write 0, timeout 0
Requests/sec:  61949.84
Transfer/sec:      7.62MB
```

## Intuitive

Based on frameworks like Koa, and Express, thruster aims to be a pleasure to develop with.

## Getting Started

### The most basic example

```rust
extern crate thruster;
extern crate futures;
extern crate serde;
extern crate serde_json;
extern crate tokio;

#[macro_use] extern crate lazy_static;

use std::boxed::Box;
use futures::future;
use tokio::reactor::Handle;

use thruster::{App, BasicContext as Ctx, MiddlewareChain, MiddlewareReturnValue, Request};

lazy_static! {
  static ref APP: App<Ctx> = {
    let mut _app = App::<Ctx>::create(generate_context);

    _app.get("/plaintext", vec![plaintext]);

    _app
  };
}

fn generate_context(request: &Request, _handle: &Handle) -> Ctx {
  Ctx {
    body: "".to_owned(),
    params: request.params().clone()
  }
}

fn plaintext(mut context: Ctx, _chain: &MiddlewareChain<Ctx>) -> MiddlewareReturnValue<Ctx> {
  let val = "Hello, World!".to_owned();
  context.body = val;

  Box::new(future::ok(context))
}

fn main() {
  println!("Starting server...");

  App::start(&APP, "0.0.0.0".to_string(), "4321".to_string());
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

### Changelog

#### 0.2.0
* Breaking Changes
  * Migrated to use Futures for all middleware callbacks
* Highlights
  * Dropped support for tree-based lookup of routes to remove potential branching
  * Removed many regexes involved in lookup
