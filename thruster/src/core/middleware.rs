use crate::core::errors::ThrusterError;
use std::boxed::Box;
use std::future::Future;
use std::pin::Pin;

pub type MiddlewareResult<C> = Result<C, ThrusterError<C>>;
pub type MiddlewareReturnValue<C> =
    Pin<Box<dyn Future<Output = MiddlewareResult<C>> + 'static + Send + Sync>>;
pub type MiddlewareNext<C> = Box<
    dyn FnOnce(C) -> Pin<Box<dyn Future<Output = MiddlewareResult<C>> + 'static + Send + Sync>>
        + 'static
        + Send
        + Sync,
>;
pub type MiddlewareFn<C> =
    fn(
        C,
        MiddlewareNext<C>,
    ) -> Pin<Box<dyn Future<Output = MiddlewareResult<C>> + 'static + Send + Sync>>;
