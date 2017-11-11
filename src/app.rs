use std::io;
use std::vec::Vec;

use futures::future;
use tokio_service::Service;

use context::Context;
use request::Request;
use response::Response;
use route_parser::RouteParser;
use middleware::{Middleware, MiddlewareChain};

type ServerFuture = future::Ok<Response, io::Error>;
enum Method {
  DELETE,
  GET,
  POST,
  PUT,
  UPDATE
}

fn _add_method_to_route(method: Method, path: String) -> String {
  let prefix = match method {
    Method::DELETE => "__DELETE__",
    Method::GET => "__GET__",
    Method::POST => "__POST__",
    Method::PUT => "__PUT__",
    Method::UPDATE => "__UPDATE__"
  };

  format!("/{}{}", prefix, path)
  // format!("{}", path)
}

pub struct App {
  _route_parser: RouteParser
}

impl App {
  pub fn new() -> App {
    App {
      _route_parser: RouteParser::new()
    }
  }

  pub fn get(&mut self, path: &'static str, middlewares: Vec<Middleware>) -> &mut App {
    self._route_parser.add_route(
      _add_method_to_route(Method::GET, format!("/{}", path.to_owned())), middlewares);

    self
  }

  pub fn post(&mut self, path: &'static str, middlewares: Vec<Middleware>) -> &mut App {
    self._route_parser.add_route(
      _add_method_to_route(Method::POST, format!("/{}", path.to_owned())), middlewares);

    self
  }

  fn resolve(&self, request: Request) -> Context {
    let path = request.path();
    let method = match request.method() {
      "DELETE" => Method::DELETE,
      "GET" => Method::GET,
      "POST" => Method::POST,
      "PUT" => Method::PUT,
      "UPDATE" => Method::UPDATE,
      _ => Method::GET
    };
    let mut context = Context::new(Response::new());

    let matched_route = self._route_parser.match_route(
      _add_method_to_route(method, path.to_owned()));
    let middleware = matched_route.middleware;
    let middleware_chain = MiddlewareChain::new(middleware);

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
    // println!("{}", _request.path());

    let result = self.resolve(_request);
    let mut response = result.response;
    response.body(&result.body);

    future::ok(response)
  }
}


#[cfg(test)]
mod tests {
  use super::App;
  use bytes::{BytesMut, BufMut};
  use context::Context;
  use request::decode;
  use middleware::MiddlewareChain;

  #[test]
  fn it_should_execute_all_middlware_with_a_given_request() {
    let mut app = App::new();

    fn test_fn_1(context: Context, _chain: &MiddlewareChain) -> Context {
      Context {
        body: "".to_string(),
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
  fn it_should_execute_all_middlware_with_a_given_request_based_on_method() {
    let mut app = App::new();

    fn test_fn_1(context: Context, _chain: &MiddlewareChain) -> Context {
      Context {
        body: "".to_string(),
        response: context.response,
        _identifier: 1
      }
    };

    fn test_fn_2(context: Context, _chain: &MiddlewareChain) -> Context {
      Context {
        body: "".to_string(),
        response: context.response,
        _identifier: 2
      }
    };

    app.get("test", vec![test_fn_1]);
    app.post("test", vec![test_fn_2]);

    let mut bytes = BytesMut::with_capacity(41);
    bytes.put(&b"GET /test HTTP/1.1\nHost: localhost:8080\n\n"[..]);

    let request = decode(&mut bytes).unwrap().unwrap();
    let ctx = app.resolve(request);

    assert!(ctx._identifier == 1);
  }

  #[test]
  fn it_should_execute_all_middlware_with_a_given_request_up_and_down() {
    let mut app = App::new();

    fn test_fn_1(context: Context, _chain: &MiddlewareChain) -> Context {
      Context {
        body: "".to_string(),
        response: context.response,
        _identifier: context._identifier + 20
      }
    };

    fn test_fn_2(context: Context, chain: &MiddlewareChain) -> Context {
      let mut _context = Context {
        body: "".to_string(),
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

  #[test]
  fn it_should_return_whatever_was_set_as_the_body_of_the_context() {
    let mut app = App::new();

    fn test_fn_1(context: Context, _chain: &MiddlewareChain) -> Context {
      Context {
        body: "Hello world".to_string(),
        response: context.response,
        _identifier: context._identifier + 20
      }
    };

    app.get("test", vec![test_fn_1]);

    let mut bytes = BytesMut::with_capacity(41);
    bytes.put(&b"GET /test HTTP/1.1\nHost: localhost:8080\n\n"[..]);

    let request = decode(&mut bytes).unwrap().unwrap();
    let ctx = app.resolve(request);

    assert!(ctx.body == "Hello world");
  }
}
