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
use middleware::Middleware;

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

    {
      let matched_route = self._route_parser.match_route(path.to_owned());


      let middleware = matched_route.middleware;
      let len = middleware.len();
      for i in 0..(len * 2) {
        if i >= len {
          context = middleware.get(len - (i % len)).unwrap()(context);
        } else {
          context = middleware.get(i).unwrap()(context);
        }
      }
    }

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
  use middleware::Middleware;
  use futures::future::{self, FutureResult};

  #[test]
  fn it_execute_all_middlware_with_a_given_request() {
    let mut app = App::new();
    // let mut test_fn_1_was_called: bool = false;

    fn test_fn_1(context: Context) -> Context {
      // test_fn_1_was_called = true;
      Context {
        response: context.response,
        _identifier: 1
      }
    };

    app.get("test", vec![test_fn_1]);

    let mut bytes = BytesMut::with_capacity(40);
    bytes.put(&b"GET /ping HTTP/1.1\nHost: localhost:8080\n"[..]);

    // let request = decode(&mut bytes).unwrap().unwrap();
    // let ctx = app.resolve(request);

    // assert!(ctx._identifier == 1);
  }
}
