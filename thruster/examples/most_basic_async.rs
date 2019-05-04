#![feature(await_macro, async_await, futures_api, proc_macro_hygiene)]
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

  context = await!(next(context));

  let elapsed_time = start_time.elapsed();
  println!("[{}μs] {} -- {}",
    elapsed_time.as_micros(),
    context.request.method(),
    context.request.path());

  context
}

#[middleware_fn]
async fn add_one(context: Ctx, next: MiddlewareNext<Ctx>) -> Ctx {
  let mut ctx: Ctx = await!(next(context));

  ctx.body(&format!("{} + 1", ctx.get_body()));
  ctx
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

  app.use_middleware("/", async_middleware!(Ctx, [profiling]));
  // for (route, middleware) in app._route_parser.route_tree.root_node.enumerate() {
  //   println!("a{}: {}", route, middleware.chain.nodes.len());
  // }

  app.get("/plaintext", async_middleware!(Ctx, [add_one, plaintext]));

  // app.set404(async_middleware!(Ctx, [four_oh_four]));

  let server = Server::new(app);
  server.start("0.0.0.0", 4321);
}
