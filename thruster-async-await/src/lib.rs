#![feature(proc_macro_hygiene)]

use std::io;
use std::future::Future;

use thruster_core::context::Context;
use thruster_core::request::{RequestWithParams};
use thruster_core::route_parser::{MatchedRoute};
#[cfg(feature = "thruster_error_handling")]
use thruster_core::errors::Error;

pub fn resolve<R: RequestWithParams, T: 'static + Context + Send>(context_generator: fn(R) -> T, mut request: R, matched_route: MatchedRoute<T>) -> impl Future<Output=Result<T::Response, io::Error>> + Send {
  request.set_params(matched_route.params);

  let context = (context_generator)(request);

  let copy = matched_route.middleware.clone();
  let context_future = async move {
    let ctx = copy.run(context).await;

    #[cfg(feature = "thruster_error_handling")]
    let ctx = match ctx {
      Ok(val) => val,
      Err(e) => {
        e.build_context()
      }
    };

    Ok(ctx.get_response())
  };

  context_future
}
