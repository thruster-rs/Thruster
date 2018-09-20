extern crate bytes;
extern crate futures;
extern crate hyper;
extern crate httparse;
extern crate http;
extern crate net2;
extern crate num_cpus;
extern crate regex;
extern crate serde;
extern crate time;
extern crate tokio;
extern crate tokio_codec;
extern crate tokio_io;

#[macro_use] extern crate smallvec;
#[macro_use] extern crate templatify;
// For tests
#[allow(unused_imports)]
#[macro_use] extern crate serde_derive;
#[allow(unused_imports)]
#[macro_use] extern crate serde_json;
#[allow(unused_imports)]
extern crate tokio_core;

mod app;
pub mod builtins;
mod context;
mod middleware;
mod route_parser;
mod route_tree;

pub mod date;
pub use app::App;
pub use context::Context;
pub use builtins::basic_context::BasicContext;
pub use builtins::basic_context::CookieOptions;
pub use middleware::{Middleware, MiddlewareChain, MiddlewareReturnValue};
pub use route_parser::MatchedRoute;
pub mod testing;
