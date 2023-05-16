# Thruster [![Latest Version]][crates.io] [![Downloads]][crates.io] [![Online]][discord]

[Latest Version]: https://img.shields.io/crates/v/thruster.svg
[Downloads]: https://img.shields.io/crates/d/thruster.svg
[Online]: https://img.shields.io/discord/658730946211610643
[crates.io]: https://crates.io/crates/thruster
[discord]: https://discord.gg/m9JrPRds

[Get started with examples and walkthroughs on our website!](https://mertz.gitbook.io/thruster/)

## A fast and intuitive rust web framework

Don't have time to read the docs? Check out

- [Why you should use Thruster](#why-you-should-use-thruster)
- [Why you shouldn't use Thruster (definitely read this one)](#why-you-shouldnt-use-thruster)

✅ Runs in stable
✅ Runs fast
✅ Doesn't use unsafe

[Documentation](https://docs.rs/thruster)

## Features

- [built with async/await in mind](https://github.com/thruster-rs/Thruster/blob/master/thruster/examples/profiling.rs#L11)
- [hyper compatible](https://github.com/thruster-rs/Thruster/blob/master/thruster/examples/hyper_most_basic.rs)
- [ssl ready](https://github.com/thruster-rs/Thruster/tree/master/thruster/examples/hyper_most_basic_ssl)
- [testable](#testing)
- [static file serving](https://github.com/thruster-rs/Thruster/tree/master/thruster/examples/static_file)
- [socketio](https://github.com/thruster-rs/thruster-socketio)
- [gRPC](https://github.com/thruster-rs/Thruster/tree/master/thruster/examples/grpc), and more experimental [non-tonic based gRPC](https://github.com/thruster-rs/thruster-grpc)
- [dependency injection](https://github.com/thruster-rs/thruster-jab)

## Motivation

Thruster is a web framework that aims for developers to be productive and consistent across projects and teams. Its goals are to be:
- Performant
- Simple
- Intuitive

Thruster also
- Does not use `unsafe`
- Works in stable rust

## Fast

Thruster can be run with different server backends and represents a nicely packaged layer over them. This means that it can keep up with the latest and greatest changes from the likes of Hyper, Actix, or even ThrusterServer, a home-grown http engine.

## Intuitive

Based on frameworks like Koa, and Express, Thruster aims to be a pleasure to develop with.

## Example

To run the example `cargo run --example <example-name>`.
For example, `cargo run --example hello_world` and open [http://localhost:4321/](http://localhost:4321/)

### Middleware Based

The core parts that make the new async await code work is designating middleware functions with the `#[middleware_fn]` attribute (which marks the middleware so that it's compatible with the stable futures version that Thruster is built on,) and then the `m!` macro in the actual routes.

A simple example for using async await is:

```rust
use std::boxed::Box;
use std::future::Future;
use std::pin::Pin;
use std::time::Instant;

use thruster::{App, BasicContext as Ctx, Request};
use thruster::{m, middleware_fn, MiddlewareNext, MiddlewareResult, Server, ThrusterServer};

#[middleware_fn]
async fn profile(context: Ctx, next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let start_time = Instant::now();

    context = next(context).await;

    let elapsed_time = start_time.elapsed();
    println!(
        "[{}μs] {} -- {}",
        elapsed_time.as_micros(),
        context.request.method(),
        context.request.path()
    );

    Ok(context)
}

#[middleware_fn]
async fn plaintext(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let val = "Hello, World!";
    context.body(val);
    Ok(context)
}

#[middleware_fn]
async fn four_oh_four(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    context.status(404);
    context.body("Whoops! That route doesn't exist!");
    Ok(context)
}

#[tokio::main]
fn main() {
    println!("Starting server...");

    let mut app = App::<Request, Ctx, ()>::new_basic();

    app.get("/plaintext", m![profile, plaintext]);
    app.set404(m![four_oh_four]);

    let server = Server::new(app);
    server.build("0.0.0.0", 4321).await;
}
```

### Error handling

Here's a nice example

```rust
use thruster::errors::ThrusterError as Error;
use thruster::proc::{m, middleware_fn};
use thruster::{map_try, App, BasicContext as Ctx, Request};
use thruster::{MiddlewareNext, MiddlewareResult, MiddlewareReturnValue, Server, ThrusterServer};

#[middleware_fn]
async fn plaintext(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let val = "Hello, World!";
    context.body(val);
    Ok(context)
}

#[middleware_fn]
async fn error(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let res = "Hello, world".parse::<u32>()
        .map_err(|_| {
            let mut context = Ctx::default();
            
            context.status(400);

            ThrusterError {
                context,
                message: "Custom error message".to_string(),
                cause: None,
            }
        }?;

    context.body(&format!("{}", non_existent_param));

    Ok(context)
}

#[tokio::main]
fn main() {
    println!("Starting server...");

    let app = App::<Request, Ctx, ()>::new_basic()
        .get("/plaintext", m![plaintext])
        .get("/error", m![error]);

    let server = Server::new(app);
    server.build("0.0.0.0", 4321).await;
}
```

## Testing
Thruster provides an easy test suite to test your endpoints, simply include the `testing` module as below:

```rust
let mut app = App::<Request, Ctx, ()>::new_basic();

...

app.get("/plaintext", m![plaintext]);

...

let result = testing::get(app, "/plaintext");

assert!(result.body == "Hello, World!");
```

## Make your own middleware modules
Middleware is super easy to make! Simply create a function and export it at a module level. Below, you'll see a piece of middleware that allows profiling of requests:

```rust
#[middleware_fn]
async fn profiling<C: 'static + Context + Send>(
    mut context: C,
    next: MiddlewareNext<C>,
) -> MiddlewareResult<C> {
    let start_time = Instant::now();

    context = next(context).await?;

    let elapsed_time = start_time.elapsed();
    info!("[{}μs] {}", elapsed_time.as_micros(), context.route());

    Ok(context)
}
```

You might find that you want to allow for more specific data stored on the context, for example, perhaps you want to be able to hydrate query parameters into a hashmap for later use by other middlewares. In order to do this, you can create an additional trait for the context that middlewares downstream must adhere to. Check out the provided [query_params middleware](https://github.com/thruster-rs/Thruster/blob/master/thruster/src/middleware/query_params.rs) for an example.

## Other, or Custom Backends

Thruster is capable of just providing the routing layer on top of a server of some sort, for example, in the Hyper snippet above. This can be applied broadly to any backend, as long as the server implements `ThrusterServer`.

```rust
use async_trait::async_trait;

#[async_trait]
pub trait ThrusterServer {
    type Context: Context + Send;
    type Response: Send;
    type Request: RequestWithParams + Send;

    fn new(App<Self::Request, Self::Context>) -> Self;
    async fn build(self, host: &str, port: u16);
}
```

There needs to be:
- An easy way to create a server.
- A function to build the server into a future that could be loaded into an async runtime.

Within the `build` function, the server implementation should:
- Start up some sort of listener for connections
- Call `let matched = app.resolve_from_method_and_path(<some method>, <some path>);` (This is providing the actual routing.)
- Call `app.resolve(<incoming request>, matched)` (This runs the chained middleware.)

## Why you should use Thruster
- Change your backends at will. Out of the box, Thruster now can be used over: [actix-web](https://github.com/thruster-rs/Thruster/blob/master/thruster/examples/actix_most_basic.rs), [hyper](https://github.com/thruster-rs/Thruster/blob/master/thruster/examples/hyper_most_basic.rs), or [a custom backend](https://github.com/thruster-rs/Thruster/blob/master/thruster/examples/hello_world.rs)
- Thruster supports [testing](#testing) from the framework level
- @trezm gets lonely when no one makes PRs or opens issues.
- Thruster is more succinct for more middleware-centric concepts -- like a route guard. Take this example in actix to restrict IPs:
```rust
fn ip_guard(head: &RequestHead) -> bool {
    // Check for the cloudflare IP header
    let ip = if let Some(val) = head.headers().get(CF_IP_HEADER) {
        val.to_str().unwrap_or("").to_owned()
    } else if let Some(val) = head.peer_addr {
        val.to_string()
    } else {
        return false;
    };

    "1.2.3.4".contains(&ip)
}

#[actix_web::post("/ping")]
async fn ping() -> Result<HttpResponse, UserPersonalError> {
    Ok(HttpResponse::Ok().body("pong"))
}

...
        web::scope("/*")
            // This is confusing, but we catch all routes that _aren't_
            // ip guarded and return an error.
            .guard(guard::Not(ip_guard))
            .route("/*", web::to(HttpResponse::Forbidden)),
    )
    .service(ping);
...
```

Here is Thruster:

```rust

#[middleware_fn]
async fn ip_guard(mut context: Ctx, next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    if "1.2.3.4".contains(&context.headers().get("Auth-Token").unwrap_or("")) {
        context = next(context).await?;

        Ok(context)
    } else {
        Err(Error::unauthorized_error(context))
    }

}

#[middleware_fn]
async fn ping(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    context.body("pong");
    Ok(context)
}

...
    app.get("/ping", m![ip_guard, plaintext]);
...
```
A bit more direct is nice!

## Why you shouldn't use Thruster
- It's got few maintainers (pretty much just one.)
- There are other projects that have been _far_ more battle tested. Thruster is in use in production, but nowhere that you'd know or that matters.
- It hasn't been optimized by wicked smarties. @trezm tries his best, but keeps getting distracted by his dog(s).
- Serously, this framework ~could be~ _is_ great, but it definitely hasn't been poked and proded like others. Your help could go a long way to making it more secure and robust, but we might not be there just yet.

If you got this far, thanks for reading! Always feel free to reach out.
