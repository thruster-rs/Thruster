#![feature(proc_macro_hygiene)]

use std::boxed::Box;

use thruster::{App, BasicContext as Ctx, MiddlewareNext, MiddlewareReturnValue, Request};
use thruster::thruster_proc::{async_middleware, middleware_fn};
use thruster::testing;
use tokio::runtime::Runtime;

#[middleware_fn]
async fn test_fn_1(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> Ctx {
  let body = &context.params.get("id").unwrap().clone();

  context.body(body);

  context
}

#[middleware_fn]
async fn test_fn_404(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> Ctx {
  context.body("404");

  context
}


#[test]
fn it_should_correctly_404_if_no_param_is_given() {
  let mut app = App::<Request, Ctx>::new_basic();

  app.get("/test/:id", async_middleware!(Ctx, [test_fn_1]));
  app.set404(async_middleware!(Ctx, [test_fn_404]));

  let _ = Runtime::new().unwrap().block_on(async {
    let response = testing::get(&app, "/test").await;

    println!("response.body: '{}'", response.body);
    assert!(response.body == "404");
  });
}
