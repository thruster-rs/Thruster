#![feature(test)]

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
extern crate tokio_io;

#[macro_use] extern crate templatify;
// For tests
#[allow(unused_imports)]
#[macro_use] extern crate serde_derive;
#[allow(unused_imports)]
#[macro_use] extern crate serde_json;
#[allow(unused_imports)]
extern crate tokio_core;
#[allow(unused_imports)]
extern crate test;

mod app;
mod builtins;
mod context;
mod date;
mod http;
mod middleware;
mod request;
mod route_parser;
mod util;

pub use app::App;
pub use builtins::send::file;
pub use context::{BasicContext, Context};
pub use middleware::{Middleware, MiddlewareChain, MiddlewareReturnValue};
pub use request::Request;
pub use httplib::Response;
pub use http::Http;
