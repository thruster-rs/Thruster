extern crate env_logger;
extern crate futures;
extern crate fanta;
extern crate tokio_proto;
extern crate tokio_service;

use std::io;
use std::collections::HashMap;

use futures::future;
use fanta::{Request, Response, Http};
use tokio_proto::TcpServer;
use tokio_service::Service;

/// `HelloWorld` is the *service* that we're going to be implementing to service
/// the HTTP requests we receive.
///
/// The tokio-minihttp crate, and much of Tokio itself, are centered around the
/// concept of a service for interoperability between crates. Our service here
/// carries no data with it.
///
/// Note that a new instance of `HelloWorld` is created for each TCP connection
/// we service, created below in the closure passed to `serve`.
type Future = future::Ok<Response, io::Error>;

struct App {
  _routes: HashMap<&'static str, fn(Request) -> Future>
}

impl App {
  fn new() -> App {
    App {
      _routes: HashMap::new()
    }
  }

  fn get(&mut self, path: &'static str, executor: fn(Request) -> Future) -> &mut App {
    self._routes.insert(path, executor);

    self
  }

  fn resolve(&self, request: Request) -> Future {
    let route_executor;

    {
        let path = request.path();

        match self._routes.get(path) {
          Some(route) => route_executor = route,
          None => panic!("Unhandled route: {}", path)
        }
    }

    route_executor(request)
  }
}

impl Service for App {
  type Request = Request;
  type Response = Response;
  type Error = io::Error;
  type Future = Future;

  fn call(&self, _request: Request) -> Future {
    println!("{}", _request.path());

    self.resolve(_request)
  }
}

fn pong(request: Request) -> Future {
  let mut resp = Response::new();

  resp.body("pong");
  future::ok(resp)
}

fn destiny(request: Request) -> Future {
  let mut resp = Response::new();

  resp.body("When no one is around you, say baby I love you.");
  future::ok(resp)
}

fn main() {
  println!("Starting server...");

  drop(env_logger::init());
  let addr = "0.0.0.0:8080".parse().unwrap();

  TcpServer::new(Http, addr)
      .serve(|| {
        let mut app = App::new();

        app.get("/ping", pong)
          .get("/saymyname", destiny);

        Ok(app)
      });
}
