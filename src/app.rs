use std::io;
use std::vec::Vec;

use futures::future;
use tokio_service::Service;
use tokio_proto::TcpServer;
use regex::Regex;

use context::{BasicContext, Context};
use request::Request;
use response::Response;
use route_parser::RouteParser;
use middleware::{Middleware, MiddlewareChain};
use http::Http;
use util::{strip_leading_slash};

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
}

fn _insert_prefix_to_methodized_path(path: String, prefix: String) -> String {
  let regex = Regex::new(r"^(__\w+__)/(.*)").unwrap();

  let path_clone_just_in_case = path.clone();
  match regex.captures(&path) {
    Some(cap) => {
      let mut stripped_prefix = strip_leading_slash(prefix);

      if stripped_prefix.len() > 0 &&
        (&cap[2]).len() > 0 {
        stripped_prefix = format!("{}/", stripped_prefix);
      }

      format!("{}/{}{}", &cap[1], stripped_prefix, &cap[2])
    },
    None => path_clone_just_in_case
  }
}

pub struct App<T: Context> {
  _route_parser: RouteParser<T>,
  pub context_generator: fn(&Request) -> T
}

fn generate_context(_request: &Request) -> BasicContext {
  BasicContext::new()
}

impl<T: Context> App<T> {
  pub fn start(app: &'static App<T>, port: String) {
    let addr = format!("0.0.0.0:{}", port).parse().unwrap();

    TcpServer::new(Http, addr)
      .serve(move || {
        let service = AppService::new(app);

        Ok(service)
      });
  }

  pub fn new() -> App<BasicContext> {
    App {
      _route_parser: RouteParser::new(),
      context_generator: generate_context
    }
  }

  pub fn create(generate_context: fn(&Request) -> T) -> App<T> {
    App {
      _route_parser: RouteParser::new(),
      context_generator: generate_context
    }
  }

  pub fn use_middleware(&mut self, path: &'static str, middleware: Middleware<T>) -> &mut App<T> {
    self._route_parser.add_method_agnostic_middleware(path.to_owned(), middleware);

    self
  }

  pub fn use_sub_app(&mut self, prefix: &'static str, app: &App<T>) -> &mut App<T> {
    let sub_app_middleware = &app.get_route_parser().middleware;

    // This is incorrect right now, because we need to prefix the path.
    for (path, middleware) in sub_app_middleware {
      self._route_parser.add_route(
        _insert_prefix_to_methodized_path(path.to_owned(), prefix.to_owned()),
        middleware.clone());
    }

    self
  }

  pub fn get_route_parser(&self) -> &RouteParser<T> {
    &self._route_parser
  }

  pub fn get(&mut self, path: &'static str, middlewares: Vec<Middleware<T>>) -> &mut App<T> {
    self._route_parser.add_route(
      _add_method_to_route(Method::GET, path.to_owned()), middlewares);

    self
  }

  pub fn post(&mut self, path: &'static str, middlewares: Vec<Middleware<T>>) -> &mut App<T> {
    self._route_parser.add_route(
      _add_method_to_route(Method::POST, path.to_owned()), middlewares);

    self
  }

  fn resolve(&self, request: Request) -> Response {
    let path = request.path();
    let method = match request.method() {
      "DELETE" => Method::DELETE,
      "GET" => Method::GET,
      "POST" => Method::POST,
      "PUT" => Method::PUT,
      "UPDATE" => Method::UPDATE,
      _ => Method::GET
    };

    let matched_route = self._route_parser.match_route(
      _add_method_to_route(method, path.to_owned()));
    match matched_route.sub_app {
      Some(sub_app) => {
        let mut context = (sub_app.context_generator)(&request);

        let middleware = matched_route.middleware;
        let middleware_chain = MiddlewareChain::new(middleware);

        context = middleware_chain.next(context);

        context.get_response()
      },
      None => {
        let mut context = (self.context_generator)(&request);

        let middleware = matched_route.middleware;
        let middleware_chain = MiddlewareChain::new(middleware);

        context = middleware_chain.next(context);

        context.get_response()
      }
    }
  }
}

pub struct AppService<'a, T: Context + 'a> {
  _app: &'a App<T>
}

impl<'a, T: Context> AppService<'a, T> {
  pub fn new(app: &'a App<T>) -> AppService<'a, T> {
    AppService {
      _app: app
    }
  }
}

impl<'a, T: Context> Service for AppService<'a, T> {
  type Request = Request;
  type Response = Response;
  type Error = io::Error;
  type Future = ServerFuture;

  fn call(&self, _request: Request) -> ServerFuture {
    // println!("{}", _request.path());

    let result = self._app.resolve(_request);

    future::ok(result)
  }
}


#[cfg(test)]
mod tests {
  use super::App;
  use bytes::{BytesMut, BufMut};
  use context::BasicContext;
  use request::decode;
  use middleware::MiddlewareChain;

  #[test]
  fn it_should_execute_all_middlware_with_a_given_request() {
    let mut app = App::<BasicContext>::new();

    fn test_fn_1(_context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> BasicContext {
      BasicContext {
        body: "1".to_string()
      }
    };

    app.get("/test", vec![test_fn_1]);

    let mut bytes = BytesMut::with_capacity(41);
    bytes.put(&b"GET /test HTTP/1.1\nHost: localhost:8080\n\n"[..]);

    let request = decode(&mut bytes).unwrap().unwrap();
    let response = app.resolve(request);

    assert!(response.get_response() == "1");
  }

  #[test]
  fn it_should_execute_all_middlware_with_a_given_request_based_on_method() {
    let mut app = App::<BasicContext>::new();

    fn test_fn_1(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> BasicContext {
      BasicContext {
        body: format!("{}{}", context.body, "1")
      }
    };

    fn test_fn_2(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> BasicContext {
      BasicContext {
        body: format!("{}{}", context.body, "2")
      }
    };

    app.get("/test", vec![test_fn_1]);
    app.post("/test", vec![test_fn_2]);

    let mut bytes = BytesMut::with_capacity(41);
    bytes.put(&b"GET /test HTTP/1.1\nHost: localhost:8080\n\n"[..]);

    let request = decode(&mut bytes).unwrap().unwrap();
    let response = app.resolve(request);

    assert!(response.get_response() == "1");
  }

  #[test]
  fn it_should_execute_all_middlware_with_a_given_request_up_and_down() {
    let mut app = App::<BasicContext>::new();

    fn test_fn_1(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> BasicContext {
      BasicContext {
        body: format!("{}{}", context.body, "1")
      }
    };

    fn test_fn_2(context: BasicContext, chain: &MiddlewareChain<BasicContext>) -> BasicContext {
      let mut _context = BasicContext {
        body: format!("{}{}", context.body, "2")
      };

      _context = chain.next(_context);

      _context.body = format!("{}{}", _context.body, "2");

      _context
    };

    app.get("/test", vec![test_fn_2, test_fn_1]);

    let mut bytes = BytesMut::with_capacity(41);
    bytes.put(&b"GET /test HTTP/1.1\nHost: localhost:8080\n\n"[..]);

    let request = decode(&mut bytes).unwrap().unwrap();
    let response = app.resolve(request);

    assert!(response.get_response() == "212");
  }

  #[test]
  fn it_should_return_whatever_was_set_as_the_body_of_the_context() {
    let mut app = App::<BasicContext>::new();

    fn test_fn_1(_context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> BasicContext {
      BasicContext {
        body: "Hello world".to_string()
      }
    };

    app.get("/test", vec![test_fn_1]);

    let mut bytes = BytesMut::with_capacity(41);
    bytes.put(&b"GET /test HTTP/1.1\nHost: localhost:8080\n\n"[..]);

    let request = decode(&mut bytes).unwrap().unwrap();
    let response = app.resolve(request);

    assert!(response.get_response() == "Hello world");
  }

  #[test]
  fn it_should_first_run_use_then_methods() {
    let mut app = App::<BasicContext>::new();

    fn method_agnostic(_context: BasicContext, chain: &MiddlewareChain<BasicContext>) -> BasicContext {
      let updated_context = chain.next(BasicContext {
        body: "agnostic".to_owned(),
      });

      BasicContext {
        body: updated_context.body
      }
    }

    fn test_fn_1(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> BasicContext {
      BasicContext {
        body: format!("{}-1", context.body)
      }
    };

    app.use_middleware("/", method_agnostic);
    app.get("/test", vec![test_fn_1]);

    let mut bytes = BytesMut::with_capacity(41);
    bytes.put(&b"GET /test HTTP/1.1\nHost: localhost:8080\n\n"[..]);

    let request = decode(&mut bytes).unwrap().unwrap();
    let response = app.resolve(request);

    assert!(response.get_response() == "agnostic-1");
  }

  #[test]
  fn it_should_be_able_to_correctly_route_sub_apps() {
    let mut app1 = App::<BasicContext>::new();

    fn test_fn_1(_context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> BasicContext {
      BasicContext {
        body: "1".to_string()
      }
    };

    app1.get("/test", vec![test_fn_1]);

    let mut app2 = App::<BasicContext>::new();
    app2.use_sub_app("/", &app1);

    let mut bytes = BytesMut::with_capacity(41);
    bytes.put(&b"GET /test HTTP/1.1\nHost: localhost:8080\n\n"[..]);

    let request = decode(&mut bytes).unwrap().unwrap();
    let response = app2.resolve(request);

    assert!(response.get_response() == "1");
  }

  #[test]
  fn it_should_be_able_to_correctly_prefix_route_sub_apps() {
    let mut app1 = App::<BasicContext>::new();

    fn test_fn_1(_context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> BasicContext {
      BasicContext {
        body: "1".to_string()
      }
    };

    app1.get("/test", vec![test_fn_1]);

    let mut app2 = App::<BasicContext>::new();
    app2.use_sub_app("/sub", &app1);

    let mut bytes = BytesMut::with_capacity(45);
    bytes.put(&b"GET /sub/test HTTP/1.1\nHost: localhost:8080\n\n"[..]);

    let request = decode(&mut bytes).unwrap().unwrap();
    let response = app2.resolve(request);

    assert!(response.get_response() == "1");
  }

  #[test]
  fn it_should_be_able_to_correctly_prefix_the_root_of_sub_apps() {
    let mut app1 = App::<BasicContext>::new();

    fn test_fn_1(_context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> BasicContext {
      BasicContext {
        body: "1".to_string()
      }
    };

    app1.get("/", vec![test_fn_1]);

    let mut app2 = App::<BasicContext>::new();
    app2.use_sub_app("/sub", &app1);

    let mut bytes = BytesMut::with_capacity(45);
    bytes.put(&b"GET /sub HTTP/1.1\nHost: localhost:8080\n\n"[..]);

    let request = decode(&mut bytes).unwrap().unwrap();
    let response = app2.resolve(request);

    assert!(response.get_response() == "1");
  }
}
