use std::io;
use std::vec::Vec;

use futures::prelude::*;
use futures::Future;
use futures::future::{self, FutureResult};
use tokio_service::Service;

use context::Context;
use fanta_error::FantaError;
use request::Request;
use response::Response;
use route_parser::RouteParser;
use middleware::{Middleware, MiddlewareChain};

type ServerFuture = future::Ok<Response, io::Error>;

pub struct App {
  _route_parser: RouteParser
}

impl App {
  fn new() -> App {
    App {
      _route_parser: RouteParser::new()
    }
  }

  fn get(&mut self, path: &'static str, middlewares: Vec<Middleware>) -> &mut App {
    self._route_parser.add_route(path.to_owned(), middlewares);

    self
  }

  fn resolve(&self, request: Request) -> Context {
    let path = request.path();
    let mut context = Context::new(Response::new());

    let matched_route = self._route_parser.match_route(path.to_owned());
    let middleware = matched_route.middleware;
    let mut middleware_chain = MiddlewareChain::new(middleware);

    context = middleware_chain.next(context);

    context
  }
}

impl Service for App {
  type Request = Request;
  type Response = Response;
  type Error = io::Error;
  type Future = ServerFuture;

  fn call(&self, _request: Request) -> ServerFuture {
    println!("{}", _request.path());

    future::ok(self.resolve(_request).response)
  }
}


#[cfg(test)]
mod tests {
  use super::App;
  use bytes::{BytesMut, BufMut};
  use context::Context;
  use request::{decode, Request};
  use fanta_error::FantaError;
  use middleware::{Middleware, MiddlewareChain};
  use futures::future::{self, FutureResult};

  #[test]
  fn it_execute_all_middlware_with_a_given_request() {
    let mut app = App::new();

    fn test_fn_1(context: Context, chain: &MiddlewareChain) -> Context {
      Context {
        response: context.response,
        _identifier: 1
      }
    };

    app.get("test", vec![test_fn_1]);

    let mut bytes = BytesMut::with_capacity(41);
    bytes.put(&b"GET /test HTTP/1.1\nHost: localhost:8080\n\n"[..]);

    let request = decode(&mut bytes).unwrap().unwrap();
    let ctx = app.resolve(request);

    assert!(ctx._identifier == 1);
  }

  #[test]
  fn it_execute_all_middlware_with_a_given_request_up_and_down() {
    let mut app = App::new();

    fn test_fn_1(context: Context, chain: &MiddlewareChain) -> Context {
      Context {
        response: context.response,
        _identifier: context._identifier + 20
      }
    };

    fn test_fn_2(context: Context, chain: &MiddlewareChain) -> Context {
      let mut _context = Context {
        response: context.response,
        _identifier: context._identifier + 1
      };

      _context = chain.next(_context);

      _context._identifier = _context._identifier * 2;

      _context
    };

    app.get("test", vec![test_fn_2, test_fn_1]);

    let mut bytes = BytesMut::with_capacity(41);
    bytes.put(&b"GET /test HTTP/1.1\nHost: localhost:8080\n\n"[..]);

    let request = decode(&mut bytes).unwrap().unwrap();
    let ctx = app.resolve(request);

    assert!(ctx._identifier == 42);
  }
}
