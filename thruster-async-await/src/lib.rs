#![feature(async_await, proc_macro_hygiene)]

use std::io;
use futures_legacy::{Future as FutureLegacy};

use thruster_core::context::Context;
use thruster_core::request::{RequestWithParams};
use thruster_core::route_parser::{MatchedRoute};

pub fn resolve<R: RequestWithParams, T: 'static + Context + Send>(context_generator: fn(R) -> T, mut request: R, matched_route: MatchedRoute<T>) -> impl FutureLegacy<Item=T::Response, Error=io::Error> + Send {
  use tokio_futures::compat::into_01;

  request.set_params(matched_route.params);

  let context = (context_generator)(request);

  let copy = matched_route.middleware.clone();
  let context_future = async move {
    let ctx = copy.run(context).await;
    Ok(ctx.get_response())
  };

  into_01(context_future)
}
