#![feature(await_macro, async_await, futures_api, proc_macro_hygiene)]

use std::io;
use futures_legacy::{Future as FutureLegacy};

use thruster_core::context::Context;
use thruster_core::request::{RequestWithParams};
use thruster_core::route_parser::{MatchedRoute};

pub fn resolve<R: RequestWithParams, T: 'static + Context + Send>(context_generator: fn(R) -> T, mut request: R, matched_route: MatchedRoute<T>) -> impl FutureLegacy<Item=T::Response, Error=io::Error> + Send {
  use tokio_async_await::compat::backward;

  request.set_params(matched_route.params);

  let context = (context_generator)(request);
  let middleware = matched_route.middleware;

  let copy = matched_route.middleware.clone();
  let context_future = async move {
    let ctx = await!(copy.run(context));
    Ok(ctx.get_response())
  };

  backward::Compat::new(context_future)
}
