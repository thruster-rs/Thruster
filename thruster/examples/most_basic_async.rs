#![feature(await_macro, async_await, futures_api, proc_macro_hygiene)]
extern crate thruster;

use std::boxed::Box;
use std::pin::Pin;
use std::future::Future;

use thruster::{Chain, Middleware, MiddlewareChain};
use thruster::{App, BasicContext as Ctx, Request};
use thruster::server::Server;
use thruster::ThrusterServer;
use thruster::thruster_proc::{async_middleware, middleware_fn};

#[middleware_fn]
async fn add_one(mut context: Ctx, next: Box<(Fn(Ctx) -> Pin<Box<Future<Output=Ctx> + Send + Sync>>) + 'static + Send + Sync>) -> Ctx {
  let mut ctx: Ctx = await!(next(context));

  ctx.body(&format!("{} + 1", ctx.get_body()));
  ctx
}

#[middleware_fn]
async fn plaintext(mut context: Ctx, _next: Box<(Fn(Ctx) -> Pin<Box<Future<Output=Ctx> + Send + Sync>>) + 'static + Send + Sync>) -> Ctx {
  let val = "Hello, World!";
  context.body(val);
  context
}

#[middleware_fn]
async fn four_oh_four(mut context: Ctx, _next: Box<(Fn(Ctx) -> Pin<Box<Future<Output=Ctx> + Send + Sync>>) + 'static + Send + Sync>) -> Ctx {
  context.status(404);
  context.body("Whoops! That route doesn't exist!");
  context
}

fn main() {
  println!("Starting server...");

  let mut app = App::<Request, Ctx>::new_basic();

  app.get("/plaintext", async_middleware!(Ctx, [add_one, plaintext]));
  app.set404(async_middleware!(Ctx, [four_oh_four]));

  let server = Server::new(app);
  server.start("0.0.0.0", 4321);
}
