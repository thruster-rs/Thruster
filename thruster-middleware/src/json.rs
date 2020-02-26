use thruster_core::context::Context;
use thruster_proc::middleware_fn;
use thruster_core::{MiddlewareNext, MiddlewareReturnValue, MiddlewareResult};

///
/// Middleware to set the request to respond with JSON type. Requires
/// `thruster_error_handling` to be turned on.
///
#[middleware_fn]
pub async fn json<T: 'static + Context + Send>(mut context: T, next: MiddlewareNext<T>) -> MiddlewareResult<T> {
  context.set("Content-Type", "application/json");

  context = next(context).await?;

  Ok(context)
}
