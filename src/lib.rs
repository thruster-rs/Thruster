extern crate bytes;
extern crate futures;
extern crate httparse;
extern crate http as httplib;
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

#[macro_use] mod middleware;
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

pub use app::App;
pub use context::Context;
pub use builtins::basic_context::BasicContext;
pub use builtins::cookies::{Cookie, CookieOptions, SameSite};
pub use middleware::{Middleware, MiddlewareChain, MiddlewareReturnValue};
pub use response::{encode, Response};
pub use request::{decode, Request};
pub use http::Http;
pub mod testing;

#[cfg(feature="hyper_server")]
pub use builtins::basic_hyper_context::BasicHyperContext;
