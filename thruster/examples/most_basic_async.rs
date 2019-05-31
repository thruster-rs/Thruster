#![feature(async_await, proc_macro_hygiene)]
extern crate thruster;

use std::time::Instant;
use thruster::{MiddlewareNext, MiddlewareReturnValue};
use thruster::{App, BasicContext as Ctx, Request};
use thruster::server::Server;
use thruster::ThrusterServer;
use thruster::thruster_proc::{async_middleware, middleware_fn};

#[middleware_fn]
async fn profiling(mut context: Ctx, next: MiddlewareNext<Ctx>) -> Ctx {
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

fn main() {
  println!("Starting server...");

  let mut app = App::<Request, Ctx>::new_basic();

  app.use_middleware("/", async_middleware!(Ctx, [profiling]));
  app.get("/plaintext", async_middleware!(Ctx, [plaintext]));

  let server = Server::new(app);
  server.start("0.0.0.0", 4321);
}
