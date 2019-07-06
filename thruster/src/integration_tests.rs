#![feature(async_await, proc_macro_hygiene)]

use thruster::{MiddlewareNext, MiddlewareReturnValue, Request};
use thruster::{App, BasicContext};
use thruster::thruster_proc::{async_middleware, middleware_fn};
use thruster::testing;

#[middleware_fn]
async fn test_fn_1(mut context: BasicContext, _next: MiddlewareNext<BasicContext>) -> BasicContext {
  let body = &context.params.get("id").unwrap().clone();

  context.body(body);
  context
}

#[middleware_fn]
async fn test_fn_404(mut context: BasicContext, _next: MiddlewareNext<BasicContext>) -> BasicContext {
  context.body("404");
  context
}

#[test]
fn it_should_correctly_404_if_no_param_is_given() {
  let mut app = App::<Request, BasicContext>::new_basic();

  app.get("/test/:id", async_middleware!(BasicContext, [test_fn_1]));
  app.get("/a", async_middleware!(BasicContext, [test_fn_1]));
  app.get("/a/b/c", async_middleware!(BasicContext, [test_fn_1]));
  app.set404(async_middleware!(BasicContext, [test_fn_404]));
  app._route_parser.optimize();

  let response = testing::get(&app, "/test");

  assert!(response.body == "404");
}
