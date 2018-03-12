extern crate fanta;
extern crate futures;
extern crate serde;
extern crate serde_json;

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate lazy_static;

mod context;

use futures::future;

use fanta::{App, MiddlewareChain, MiddlewareReturnValue};
use context::{generate_context, Ctx};

lazy_static! {
  static ref APP: App<Ctx> = {
    let mut _app = App::<Ctx>::create(generate_context);

    _app.get("/json", vec![json]);
    _app.get("/plaintext", vec![plaintext]);

    _app.set404(vec![not_found_404]);

    _app
  };
}

fn not_found_404(context: Ctx, _chain: &MiddlewareChain<Ctx>) -> MiddlewareReturnValue<Ctx> {
  let mut context = Ctx::new(context);

  context.body = "<html>
  ( ͡° ͜ʖ ͡°) What're you looking for here?
</html>".to_owned();
  context.set_header("Content-Type".to_owned(), "text/html".to_owned());
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
  context.set_header("Server".to_owned(), "fanta".to_owned());
  context.set_header("Content-Type".to_owned(), "application/json".to_owned());

  Box::new(future::ok(context))
}

fn plaintext(mut context: Ctx, _chain: &MiddlewareChain<Ctx>) -> MiddlewareReturnValue<Ctx> {
  let val = "Hello, World!".to_owned();

  context.body = val;
  context.set_header("Server".to_owned(), "fanta".to_owned());
  context.set_header("Content-Type".to_owned(), "text/plain".to_owned());

  Box::new(future::ok(context))
}

fn main() {
  println!("Starting server...");

  App::start(&APP, "0.0.0.0".to_string(), "4321".to_string());
}
