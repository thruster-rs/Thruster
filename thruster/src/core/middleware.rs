use crate::core::errors::ThrusterError;
use crate::ReusableBoxFuture;
use std::boxed::Box;



pub type MiddlewareResult<C> = Result<C, ThrusterError<C>>;
pub type MiddlewareReturnValue<C> = ReusableBoxFuture<MiddlewareResult<C>>;
pub type MiddlewareNext<C> = Box<dyn FnOnce(C) -> ReusableBoxFuture<MiddlewareResult<C>> + Send>;
pub type MiddlewareFn<C> = fn(C, MiddlewareNext<C>) -> ReusableBoxFuture<MiddlewareResult<C>>;
