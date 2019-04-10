extern crate httparse;
extern crate http as httplib;

#[macro_use] extern crate templatify;

#[cfg(not(feature = "thruster_async_await"))]
#[macro_use] pub mod middleware;

pub mod date;
pub mod http;
pub mod request;
pub mod response;
pub mod context;
pub mod route_parser;
pub mod route_tree;

#[cfg(not(feature = "thruster_async_await"))]
pub use crate::middleware::*;

#[cfg(feature = "thruster_async_await")]
pub use thruster_core_async_await::{Chain, Middleware, MiddlewareChain, MiddlewareReturnValue};
#[cfg(feature = "thruster_async_await")]
pub use thruster_core_async_await::middleware;
