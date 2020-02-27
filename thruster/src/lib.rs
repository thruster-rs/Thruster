#[macro_use]
extern crate templatify;

mod app;
mod async_await;
mod core;
mod server;

pub mod context;
pub mod middleware;

pub use app::app::App;
pub use app::testing_async as testing;
pub use crate::core::context::Context;
pub use crate::core::http::Http;
pub use crate::core::middleware::MiddlewareResult;
pub use crate::core::request::{decode, Request, RequestWithParams};
pub use crate::core::response::{encode, Response};
pub use crate::core::{errors, map_try};
pub use crate::core::{
    Chain, Middleware, MiddlewareChain, MiddlewareFn, MiddlewareNext, MiddlewareReturnValue,
};

#[cfg(feature = "hyper_server")]
pub use server::hyper_server;

#[cfg(all(feature = "hyper_server", feature = "tls"))]
pub use server::ssl_hyper_server;

#[cfg(feature = "tls")]
pub use server::ssl_server;

pub use context::basic_context::BasicContext;
pub use server::ThrusterServer;

// This backwards compats since the async_middleware used to live in
// proc.
pub mod thruster_proc {
    pub use core_async_await::async_middleware;
    pub use thruster_proc::*;
}
pub use core_async_await::async_middleware;

#[cfg(feature = "hyper_server")]
pub use context::basic_hyper_context::{BasicHyperContext, HyperRequest};
