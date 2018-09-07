extern crate thruster;
extern crate futures;
extern crate serde;
extern crate serde_json;
extern crate smallvec;
extern crate tokio;

#[macro_use] extern crate serde_derive;

mod context;

use futures::future;

use thruster::{App, MiddlewareChain, MiddlewareReturnValue};
use context::{generate_context, Ctx};

fn not_found_404(context: Ctx, _chain: &MiddlewareChain<Ctx>) -> MiddlewareReturnValue<Ctx> {
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
fn json(mut context: Ctx, _chain: &MiddlewareChain<Ctx>) -> MiddlewareReturnValue<Ctx> {
  let json = JsonStruct {
    message: "Hello, World!"
  };

  let val = serde_json::to_string(&json).unwrap();

  context.body = val;
  context.set_header("Content-Type", "application/json");

  Box::new(future::ok(context))
}

fn plaintext(mut context: Ctx, _chain: &MiddlewareChain<Ctx>) -> MiddlewareReturnValue<Ctx> {
  let val = "Hello, World!".to_owned();

  context.body = val;
  context.set_header("Content-Type", "text/plain");

  Box::new(future::ok(context))
}

fn main() {
  println!("Starting server...");

  let mut app = App::create(generate_context);

  app.get("/json", vec![json]);
  app.get("/plaintext", vec![plaintext]);
  app.post("/post-plaintext", vec![plaintext]);

  app.get("/*", vec![not_found_404]);

  App::start(app, "0.0.0.0", 4321);
}
