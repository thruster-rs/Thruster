use thruster_proc::middleware_fn;

use crate::core::context::Context;
use crate::core::{MiddlewareNext, MiddlewareReturnValue, MiddlewareResult};

///
/// Middleware to set the request to respond with JSON type.
///
#[middleware_fn]
pub async fn json<T: 'static + Context + Send>(mut context: T, next: MiddlewareNext<T>) -> MiddlewareResult<T> {
  context.set("Content-Type", "application/json");

  context = next(context).await?;

  Ok(context)
}
