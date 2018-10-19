extern crate thruster;
extern crate futures;

use futures::future;

use thruster::{App, BasicContext, Context, MiddlewareChain, MiddlewareReturnValue};
use thruster::builtins::server::Server;
use thruster::builtins::basic_context::generate_context;
use thruster::server::ThrusterServer;

trait Ctx1 {
  fn set_hello_world(&mut self);
}

trait Ctx2 {
  fn set_goodbye_world(&mut self);
}

impl Ctx1 for BasicContext {
  fn set_hello_world(&mut self) {
    self.body = "Hello, World!".to_owned();
  }
}

impl Ctx2 for BasicContext {
  fn set_goodbye_world(&mut self) {
    self.body = "Goodbye, cruel world!".to_owned();
  }
}

fn hello<Ctx: 'static + Ctx1 + Context + Send>(mut context: Ctx, _chain: &MiddlewareChain<Ctx>) -> MiddlewareReturnValue<Ctx> {
  context.set_hello_world();

  Box::new(future::ok(context))
}

fn goodbye<Ctx: 'static + Ctx2 + Context + Send>(mut context: Ctx, _chain: &MiddlewareChain<Ctx>) -> MiddlewareReturnValue<Ctx> {
  context.set_goodbye_world();

  Box::new(future::ok(context))
}

fn main() {
  println!("Starting server...");

  let mut app = App::create(generate_context);
  let mut sub_app = App::create(generate_context);

  sub_app.get("/", vec![goodbye]);

  app.get("/hello", vec![hello]);
  app.use_sub_app("/goodbye", sub_app);

  let server = Server::new(app);
  server.start("0.0.0.0", 4321);
}
