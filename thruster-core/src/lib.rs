extern crate httparse;
extern crate http as httplib;

#[macro_use] extern crate templatify;

#[cfg(not(feature = "thruster_async_await"))]
#[macro_use] pub mod middleware;
#[cfg(feature = "thruster_async_await")]
pub mod async_middleware;

pub mod date;
pub mod http;
pub mod request;
pub mod response;
pub mod context;
pub mod route_parser;
pub mod route_tree;

#[cfg(not(feature = "thruster_async_await"))]
pub use crate::middleware::{Middleware, MiddlewareChain, MiddlewareReturnValue};

#[cfg(feature = "thruster_async_await")]
pub use crate::async_middleware::{Chain, Middleware, MiddlewareChain, MiddlewareReturnValue};
