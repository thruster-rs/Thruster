extern crate env_logger;
extern crate futures;
extern crate fanta;
extern crate tokio_proto;
extern crate tokio_service;

use fanta::{App, Context, MiddlewareChain, Http};
use tokio_proto::TcpServer;

/// `HelloWorld` is the *service* that we're going to be implementing to service
/// the HTTP requests we receive.
///
/// The tokio-minihttp crate, and much of Tokio itself, are centered around the
/// concept of a service for interoperability between crates. Our service here
/// carries no data with it.
///
/// Note that a new instance of `HelloWorld` is created for each TCP connection
/// we service, created below in the closure passed to `serve`.
fn index(context: Context, _chain: &MiddlewareChain) -> Context {
  Context {
    body: "Hello world".to_string(),
    response: context.response,
    _identifier: 0
  }
}

fn main() {
  println!("Starting server...");

  drop(env_logger::init());
  let addr = "0.0.0.0:8080".parse().unwrap();

  TcpServer::new(Http, addr)
      .serve(|| {
        let mut app = App::new();

        app.get("/index.html", vec![index]);

        Ok(app)
      });
}
