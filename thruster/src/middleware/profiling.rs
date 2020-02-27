use std::time::Instant;
use core::context::Context;
use proc::middleware_fn;
use core::{MiddlewareNext, MiddlewareReturnValue, MiddlewareResult};
use log::{info};

///
/// Middleware to set the request to respond with JSON type.
///
#[middleware_fn]
pub async fn json<T: 'static + Context + Send>(mut context: T, next: MiddlewareNext<T>) -> MiddlewareResult<T> {
  let start_time = Instant::now();

  context = next(context).await?;

  let elapsed_time = start_time.elapsed();
  info!("{}μs\t\t{}",
    elapsed_time.as_micros(),
    context.route());

  Ok(context)
}
