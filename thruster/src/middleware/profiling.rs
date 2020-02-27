use core::context::Context;
use core::{MiddlewareNext, MiddlewareResult, MiddlewareReturnValue};
use log::info;
use proc::middleware_fn;
use std::time::Instant;

///
/// Middleware to set the request to respond with JSON type.
///
#[middleware_fn]
pub async fn json<T: 'static + Context + Send>(
    mut context: T,
    next: MiddlewareNext<T>,
) -> MiddlewareResult<T> {
    let start_time = Instant::now();

    context = next(context).await?;

    let elapsed_time = start_time.elapsed();
    info!("{}μs\t\t{}", elapsed_time.as_micros(), context.route());

    Ok(context)
}
