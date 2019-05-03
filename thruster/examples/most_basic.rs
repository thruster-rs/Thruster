#[macro_use] extern crate thruster;
extern crate futures_legacy;

use std::boxed::Box;
use futures_legacy::future;
use futures_legacy::Future;

use thruster::{App, BasicContext as Ctx, MiddlewareChain, MiddlewareReturnValue, Request};
use thruster::server::Server;
use thruster::ThrusterServer;

fn profiling(context: Ctx, next: impl Fn(Ctx) -> MiddlewareReturnValue<Ctx>  + Send + Sync) -> MiddlewareReturnValue<Ctx> {
  println!("[start] {} -- {}",
    context.request.method(),
    context.request.path());

  let ctx_future = next(context)
      .and_then(move |ctx| {
        println!("[end] {} -- {}",
          ctx.request.method(),
          ctx.request.path());

        future::ok(ctx)
      });

  Box::new(ctx_future)
}

fn plaintext(mut context: Ctx, _next: impl Fn(Ctx) -> MiddlewareReturnValue<Ctx>  + Send + Sync) -> MiddlewareReturnValue<Ctx> {
  let val = "Hello, World!";
  context.body(val);

  Box::new(future::ok(context))
}

fn main() {
  println!("Starting server...");

  let mut app = App::<Request, Ctx>::new_basic();

  app.use_middleware("/", middleware![Ctx => profiling]);
  app.get("/plaintext", middleware![Ctx => plaintext]);

  let server = Server::new(app);
  server.start_small_load_optimized("0.0.0.0", 4321);
}
