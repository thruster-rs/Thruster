extern crate httparse;
extern crate http as httplib;

#[macro_use] extern crate templatify;

#[cfg(not(feature = "thruster_async_await"))]
#[macro_use] pub mod middleware;
#[cfg(feature = "thruster_error_handling")]
#[macro_use] pub mod macros;

pub mod date;
pub mod http;
pub mod request;
pub mod response;
pub mod context;
pub mod route_parser;
pub mod route_tree;
#[cfg(feature = "thruster_error_handling")]
pub mod errors;

#[cfg(not(feature = "thruster_async_await"))]
pub use crate::middleware::*;
#[cfg(feature = "thruster_async_await")]
pub use thruster_core_async_await::{Chain, Middleware, MiddlewareChain, MiddlewareNext, MiddlewareReturnValue};
#[cfg(feature = "thruster_async_await")]
pub use thruster_core_async_await::middleware;
#[cfg(feature = "thruster_error_handling")]
pub use thruster_core_async_await::{MiddlewareResult};
