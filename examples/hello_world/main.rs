#[macro_use] extern crate thruster;
extern crate futures;
extern crate serde;
extern crate serde_json;
extern crate smallvec;
extern crate tokio;

#[macro_use] extern crate serde_derive;

mod context;

use futures::future;

use thruster::{App, MiddlewareChain, MiddlewareReturnValue};
use thruster::builtins::server::Server;
use thruster::server::ThrusterServer;
use crate::context::{generate_context, Ctx};

fn not_found_404(context: Ctx, _next: impl Fn(Ctx) -> MiddlewareReturnValue<Ctx>  + Send + Sync) -> MiddlewareReturnValue<Ctx> {
  let mut context = Ctx::new(context);

  context.body = "<html>
  ( ͡° ͜ʖ ͡°) What're you looking for here?
</html>".to_owned();
  context.set_header("Content-Type", "text/html");
  context.status_code = 404;

  Box::new(future::ok(context))
}

#[derive(Serialize)]
struct JsonStruct<'a> {
  message: &'a str
}
fn json(mut context: Ctx, _next: impl Fn(Ctx) -> MiddlewareReturnValue<Ctx>  + Send + Sync) -> MiddlewareReturnValue<Ctx> {
  let json = JsonStruct {
    message: "Hello, World!"
  };

  let val = serde_json::to_string(&json).unwrap();

  context.body = val;
  context.set_header("Content-Type", "application/json");

  Box::new(future::ok(context))
}

fn plaintext(mut context: Ctx, _next: impl Fn(Ctx) -> MiddlewareReturnValue<Ctx>  + Send + Sync) -> MiddlewareReturnValue<Ctx> {
  let val = "Hello, World!".to_owned();

  context.body = val;
  context.set_header("Content-Type", "text/plain");

  Box::new(future::ok(context))
}

fn main() {
  println!("Starting server...");

  let mut app = App::create(generate_context);

  app.get("/json", middleware![Ctx => json]);
  app.get("/plaintext", middleware![Ctx => plaintext]);
  app.post("/post-plaintext", middleware![Ctx => plaintext]);

  app.get("/*", middleware![Ctx => not_found_404]);

  let server = Server::new(app);
  server.start("0.0.0.0", 4321);
}
