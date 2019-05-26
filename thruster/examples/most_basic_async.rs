#![feature(async_await, proc_macro_hygiene)]
extern crate thruster;

use std::time::Instant;
use thruster::{MiddlewareNext, MiddlewareReturnValue, MiddlewareResult};
use thruster::{App, BasicContext as Ctx, Request, map_try};
use thruster::server::Server;
use thruster::errors::ThrusterError as Error;
use thruster::ThrusterServer;
use thruster::thruster_proc::{async_middleware, middleware_fn};

trait ErrorSet {
  fn parsing_error(context: Ctx) -> Error<Ctx>;
}

impl ErrorSet for Error<Ctx> {
  fn parsing_error(context: Ctx) -> Error<Ctx> {
    Error {
      context,
      message: "Parsing failure!".to_string(),
      status: 400,
      cause: None
    }
  }
}

#[middleware_fn]
async fn profiling(mut context: Ctx, next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
  let start_time = Instant::now();

  context = next(context).await?;

  let elapsed_time = start_time.elapsed();
  println!("[{}μs] {} -- {}",
    elapsed_time.as_micros(),
    context.request.method(),
    context.request.path());

  Ok(context)
}

#[middleware_fn]
async fn add_one(context: Ctx, next: MiddlewareNext<Ctx>) ->  MiddlewareResult<Ctx> {
  let mut ctx: Ctx = next(context).await?;

  ctx.body(&format!("{} + 1", ctx.get_body()));
  Ok(ctx)
}

#[middleware_fn]
async fn plaintext(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
  let val = "Hello, World!";
  context.body(val);
  Ok(context)
}

#[middleware_fn]
async fn error(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
  let res = "Hello, world".parse::<u32>();
  let non_existent_param = map_try!(res, Err(_) => Error::parsing_error(context));

  context.body(&format!("{}", non_existent_param));

  Ok(context)
}

#[middleware_fn]
async fn json_error_handler(context: Ctx, next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
  let res = next(context).await;

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

#[middleware_fn]
async fn four_oh_four(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
  context.status(404);
  context.body("Whoops! That route doesn't exist!");
  Ok(context)
}

fn main() {
  println!("Starting server...");

  let mut app = App::<Request, Ctx>::new_basic();

  app.use_middleware("/", async_middleware!(Ctx, [profiling, json_error_handler]));

  app.get("/plaintext", async_middleware!(Ctx, [add_one, plaintext]));
  app.get("/error", async_middleware!(Ctx, [add_one, error]));
  app.set404(async_middleware!(Ctx, [four_oh_four]));

  let server = Server::new(app);
  server.start("0.0.0.0", 4321);
}
