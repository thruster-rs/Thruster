use std::boxed::Box;
use futures::future;

use thruster::{App, BasicContext as Ctx, MiddlewareChain, MiddlewareReturnValue, Request};
use thruster_core::{middleware};
use thruster::testing;


fn test_fn_1(mut context: Ctx, _next: impl Fn(Ctx) -> MiddlewareReturnValue<Ctx>  + Send + Sync) -> MiddlewareReturnValue<Ctx> {
  let body = &context.params.get("id").unwrap().clone();

  context.body(body);

  Box::new(future::ok(context))
}

fn test_fn_404(mut context: Ctx, _next: impl Fn(Ctx) -> MiddlewareReturnValue<Ctx>  + Send + Sync) -> MiddlewareReturnValue<Ctx> {
  context.body("404");

  Box::new(future::ok(context))
}


#[test]
fn it_should_correctly_404_if_no_param_is_given() {
  let mut app = App::<Request, Ctx>::new_basic();

  app.get("/test/:id", middleware![Ctx => test_fn_1]);
  app.set404(middleware![Ctx => test_fn_404]);
  app._route_parser.optimize();

  let response = testing::get(&app, "/test");

  assert!(response.body == "404");
}
