#![feature(await_macro, async_await, futures_api)]
#![feature(plugin_registrar, rustc_private)]

extern crate bytes;
extern crate futures;
extern crate futures_legacy;
extern crate httparse;
extern crate http as httplib;
extern crate middleware_proc;
extern crate net2;
extern crate num_cpus;
extern crate regex;
extern crate serde;
extern crate time;
extern crate tokio;
extern crate tokio_codec;
extern crate tokio_io;
extern crate tokio_core;

#[cfg(feature="hyper_server")]
extern crate hyper;

extern crate smallvec;
#[macro_use] extern crate templatify;
// For tests
#[allow(unused_imports)]
#[macro_use] extern crate serde_derive;
#[allow(unused_imports)]
#[macro_use] extern crate serde_json;

#[cfg(not(feature = "async_await"))]
#[macro_use] mod middleware;
#[cfg(feature = "async_await")]
mod async_middleware;

mod app;
pub mod builtins;
pub mod server;
mod context;
mod date;
mod http;
mod response;
mod request;
mod route_parser;
mod route_tree;

#[cfg(not(feature = "async_await"))]
pub use crate::middleware::{Middleware, MiddlewareChain, MiddlewareReturnValue};
#[cfg(feature = "async_await")]
pub use crate::async_middleware::{Chain, Middleware, MiddlewareChain, MiddlewareReturnValue};

pub use crate::app::App;
pub use crate::context::Context;
pub use crate::builtins::basic_context::BasicContext;
pub use crate::builtins::cookies::{Cookie, CookieOptions, SameSite};
pub use crate::response::{encode, Response};
pub use crate::request::{decode, Request};
pub use crate::http::Http;
pub mod testing;
pub use middleware_proc::{async_middleware, middleware_fn};

#[cfg(feature="hyper_server")]
pub use crate::builtins::basic_hyper_context::BasicHyperContext;
