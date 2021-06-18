use thruster_proc::middleware_fn;

use crate::core::context::Context;
use crate::core::{MiddlewareNext, MiddlewareResult};

///
/// Middleware to allow CORS.
///
#[middleware_fn(_internal)]
pub async fn cors<T: 'static + Context + Send>(
    mut context: T,
    next: MiddlewareNext<T>,
) -> MiddlewareResult<T> {
    context.set("Access-Control-Allow-Origin", "*");
    context.set("Access-Control-Allow-Headers", "*");
    context.set(
        "Access-Control-Allow-Methods",
        "GET, POST, PUT, DELETE, OPTIONS",
    );

    next(context).await
}
