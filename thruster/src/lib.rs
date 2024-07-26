#![allow(clippy::needless_lifetimes)]
#![allow(clippy::type_complexity)]

#[macro_use]
extern crate templatify;

#[macro_use]
extern crate log;

mod app;
mod core;
// mod parser;
mod server;

pub mod context;
pub mod middleware;
pub mod parser;

pub use crate::core::context::Context;
pub use crate::core::context_state::ContextState;
pub use crate::core::errors;
pub use crate::core::http::Http;
pub use crate::core::middleware::MiddlewareResult;
pub use crate::core::request::{decode, Request, RequestWithParams, ThrusterRequest};
pub use crate::core::response::{encode, Response};
pub use crate::core::{MiddlewareFn, MiddlewareNext, MiddlewareReturnValue};
pub use app::testing_async as testing;
pub use app::App;
pub use app::Testable;

// Reexport tokio_util::sync::ReusableBoxFuture;
pub use tokio_util::sync::ReusableBoxFuture;

pub use server::*;

#[cfg(feature = "hyper_server")]
pub use server::hyper_server;

#[cfg(feature = "unix_hyper_server")]
pub use server::unix_hyper_server;

#[cfg(all(feature = "hyper_server", feature = "tls"))]
pub use server::ssl_hyper_server;

pub use context::basic_context::BasicContext;
pub use server::ThrusterServer;
pub use thruster_proc::*;

#[cfg(feature = "hyper_server")]
pub use context::basic_hyper_context::BasicHyperContext;
#[cfg(feature = "hyper_server")]
pub use context::hyper_request::HyperRequest;
