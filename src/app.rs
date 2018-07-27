use std::io;
use std::net::ToSocketAddrs;
use smallvec::SmallVec;

use futures::future;
use futures::Future;
use tokio;
use tokio::net::{TcpStream, TcpListener};
use tokio::prelude::*;
use tokio_codec::Framed;
use tokio_proto::TcpServer;
use tokio_service::Service;
use num_cpus;

use context::{BasicContext, Context};
use http::{Http, HttpProto};
use response::Response;
use request::Request;
use route_parser::{MatchedRoute, RouteParser};
use middleware::{Middleware, MiddlewareChain, MiddlewareReturnValue};
use std::sync::Arc;

enum Method {
  DELETE,
  GET,
  POST,
  PUT,
  UPDATE
}

fn _add_method_to_route(method: Method, path: &str) -> String {
  let prefix = match method {
    Method::DELETE => "__DELETE__",
    Method::GET => "__GET__",
    Method::POST => "__POST__",
    Method::PUT => "__PUT__",
    Method::UPDATE => "__UPDATE__"
  };

  format!("{}{}", prefix, path)
}

#[inline]
fn _add_method_to_route_from_str(method: &str, path: &str) -> String {
  templatify!("__" ; method ; "__" ; path ; "")
}

struct AppService<T: 'static + Context + Send> {
  app: Arc<App<T>>
}

impl<T: 'static + Context + Send> Service for AppService<T> {
    type Request = Request;
    type Response = Response;
    type Error = io::Error;
    type Future = Box<Future<Item=Response, Error=io::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        Box::new(self.app.resolve(req))
    }
}

///
/// App, the main component of Thruster. The App is the entry point for your application
/// and handles all incomming requests. Apps are also composeable, that is, via the `subapp`
/// method, you can use all of the methods and middlewares contained within an app as a subset
/// of your app's routes.
///
/// There are three main parts to creating a thruster app:
/// 1. Use `App.create` to create a new app with a custom context generator
/// 2. Add routes and middleware via `.get`, `.post`, etc.
/// 3. Start the app with `App.start`
///
/// # Examples
/// Subapp
///
/// ```rust, ignore
/// let mut app1 = App::<BasicContext>::new();
///
/// fn test_fn_1(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> MiddlewareReturnValue<BasicContext> {
///   Box::new(future::ok(BasicContext {
///     body: context.params.get("id").unwrap().to_owned(),
///     params: context.params,
///     query_params: context.query_params
///   }))
/// };
///
/// app1.get("/:id", vec![test_fn_1]);
///
/// let mut app2 = App::<BasicContext>::new();
/// app2.use_sub_app("/test", &app1);
/// ```
///
/// In the above example, the route `/test/some-id` will return `some-id` in the body of the response.
///
pub struct App<T: 'static + Context + Send> {
  pub _route_parser: RouteParser<T>,
  ///
  /// Generate context is common to all `App`s. It's the function that's called upon receiving a request
  /// that translates an acutal `Request` struct to your custom Context type. It should be noted that
  /// the context_generator should be as fast as possible as this is called with every request, including
  /// 404s.
  pub context_generator: fn(Request) -> T,
  not_found: SmallVec<[Middleware<T>; 8]>
}

fn generate_context(request: Request) -> BasicContext {
  BasicContext {
    body: "".to_owned(),
    params: request.params().clone(),
    query_params: request.query_params().clone()
  }
}

impl<T: Context + Send> App<T> {
  pub fn start(mut app: App<T>, host: &str, port: u16) {
    let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();

    app._route_parser.optimize();
    let arc_app = Arc::new(app);

    let mut srv = TcpServer::new(HttpProto, addr);
    srv.threads(num_cpus::get());
    srv.serve(move || {
        Ok(AppService {
            app: arc_app.clone()
        })
    })

    // let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();

    // app._route_parser.optimize();

    // let listener = TcpListener::bind(&addr).unwrap();
    // let arc_app = Arc::new(app);

    // fn process<T: Context + Send>(app: Arc<App<T>>, socket: TcpStream) {
    //   let framed = Framed::new(socket, Http);
    //   let (tx, rx) = framed.split();

    //   let task = tx.send_all(rx.and_then(move |request: Request| {
    //         app.resolve(request)
    //       }))
    //       .then(|_| future::ok(()));

    //   // Spawn the task that handles the connection.
    //   tokio::spawn(task);
    // }

    // let server = listener.incoming()
    //     .map_err(|e| println!("error = {:?}", e))
    //     .for_each(move |socket| {
    //         process(arc_app.clone(), socket);
    //         Ok(())
    //     });

    // tokio::run(server);
  }

  /// Creates a new instance of app with the library supplied `BasicContext`. Useful for trivial
  /// examples, likely not a good solution for real code bases. The advantage is that the
  /// context_generator is already supplied for the developer.
  pub fn new() -> App<BasicContext> {
    App {
      _route_parser: RouteParser::new(),
      context_generator: generate_context,
      not_found: SmallVec::new()
    }
  }

  /// Create a new app with the given context generator. The app does not begin listening until start
  /// is called.
  pub fn create(generate_context: fn(Request) -> T) -> App<T> {
    App {
      _route_parser: RouteParser::new(),
      context_generator: generate_context,
      not_found: SmallVec::new()
    }
  }

  /// Add method-agnostic middleware for a route. This is useful for applying headers, logging, and
  /// anything else that might not be sensitive to the HTTP method for the endpoint.
  pub fn use_middleware(&mut self, path: &'static str, middleware: Middleware<T>) -> &mut App<T> {
    self._route_parser.add_method_agnostic_middleware(path, middleware);

    self
  }

  /// Add an app as a predetermined set of routes and middleware. Will prefix whatever string is passed
  /// in to all of the routes. This is a main feature of Thruster, as it allows projects to be extermely
  /// modular and composeable in nature.
  pub fn use_sub_app(&mut self, prefix: &'static str, app: App<T>) -> &mut App<T> {
    self._route_parser.route_tree
      .add_route_tree(prefix, app._route_parser.route_tree);

    self
  }

  /// Return the route parser for a given app
  pub fn get_route_parser(&self) -> &RouteParser<T> {
    &self._route_parser
  }

  /// Add a route that responds to `GET`s to a given path
  pub fn get(&mut self, path: &'static str, middlewares: Vec<Middleware<T>>) -> &mut App<T> {
    self._route_parser.add_route(
      &_add_method_to_route(Method::GET, path), SmallVec::from_vec(middlewares));

    self
  }

  /// Add a route that responds to `POST`s to a given path
  pub fn post(&mut self, path: &'static str, middlewares: Vec<Middleware<T>>) -> &mut App<T> {
    self._route_parser.add_route(
      &_add_method_to_route(Method::POST, path), SmallVec::from_vec(middlewares));

    self
  }

  /// Add a route that responds to `PUT`s to a given path
  pub fn put(&mut self, path: &'static str, middlewares: Vec<Middleware<T>>) -> &mut App<T> {
    self._route_parser.add_route(
      &_add_method_to_route(Method::PUT, path), SmallVec::from_vec(middlewares));

    self
  }

  /// Add a route that responds to `DELETE`s to a given path
  pub fn delete(&mut self, path: &'static str, middlewares: Vec<Middleware<T>>) -> &mut App<T> {
    self._route_parser.add_route(
      &_add_method_to_route(Method::DELETE, path), SmallVec::from_vec(middlewares));

    self
  }

  /// Add a route that responds to `UPDATE`s to a given path
  pub fn update(&mut self, path: &'static str, middlewares: Vec<Middleware<T>>) -> &mut App<T> {
    self._route_parser.add_route(
      &_add_method_to_route(Method::UPDATE, path), SmallVec::from_vec(middlewares));

    self
  }

  /// Sets the middleware if no route is successfully matched.
  pub fn set404(&mut self, middlewares: Vec<Middleware<T>>) -> &mut App<T> {
    self.not_found = SmallVec::from_vec(middlewares);

    self
  }


  fn _req_to_matched_route(&self, request: &Request) -> MatchedRoute<T> {
    let path = request.path();

    self._route_parser.match_route(
      &_add_method_to_route_from_str(request.method(), path))
  }

  /// Resolves a request, returning a future that is processable into a Response

  pub fn resolve(&self, mut request: Request) -> impl Future<Item=Response, Error=io::Error> + Send {
    let matched_route = self._req_to_matched_route(&request);
    request.set_params(matched_route.params);
    request.set_query_params(matched_route.query_params);

    let context = (self.context_generator)(request);
    let middleware = matched_route.middleware;
    let middleware_chain = MiddlewareChain::new(middleware, &self.not_found);

    let context_future = middleware_chain.next(context);

    context_future
        .and_then(|context| {
          future::ok(context.get_response())
        })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashMap;
  use bytes::{BytesMut, BufMut};
  use context::{BasicContext, Context};
  use request::{decode, Request};
  use middleware::{MiddlewareChain, MiddlewareReturnValue};
  use response::Response;
  use serde;
  use futures::{future, Future};
  use std::boxed::Box;
  use std::io;
  use std::marker::Send;

  struct TypedContext<T> {
    pub request_body: T,
    pub body: String
  }

  impl<T> TypedContext<T> {
    pub fn new<'a>(request: &'a Request) -> TypedContext<T>
      where T: serde::de::Deserialize<'a> {
      match request.body_as::<T>(request.raw_body()) {
        Ok(val) => TypedContext {
          body: "".to_owned(),
          request_body: val
        },
        Err(err) => panic!("Could not create context: {}", err)
      }
    }
  }

  impl<T> Context for TypedContext<T> {
    fn get_response(self) -> Response {
      let mut response = Response::new();
      response.body(&self.body);

      response
    }

    fn set_body(&mut self, body: String) {
      self.body = body;
    }
  }

  #[test]
  fn it_should_execute_all_middlware_with_a_given_request() {
    let mut app = App::<BasicContext>::new();

    fn test_fn_1(_context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      Box::new(future::ok(BasicContext {
        body: "1".to_string(),
        params: HashMap::new(),
        query_params: HashMap::new()
      }))
    };

    app.get("/test", vec![test_fn_1]);

    let mut bytes = BytesMut::with_capacity(41);
    bytes.put(&b"GET /test HTTP/1.1\nHost: localhost:8080\n\n"[..]);


    let request = decode(&mut bytes).unwrap().unwrap();
    let response = app.resolve(request).wait().unwrap();

    assert!(String::from_utf8(response.response).unwrap() == "1");
  }


  #[test]
  fn it_should_handle_query_parameters() {
    let mut app = App::<BasicContext>::new();

    fn test_fn_1(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      Box::new(future::ok(BasicContext {
        body: context.query_params.get("hello").unwrap().to_owned(),
        params: HashMap::new(),
        query_params: context.query_params
      }))
    };

    app.get("/test", vec![test_fn_1]);

    let mut bytes = BytesMut::with_capacity(53);
    bytes.put(&b"GET /test?hello=world HTTP/1.1\nHost: localhost:8080\n\n"[..]);


    let request = decode(&mut bytes).unwrap().unwrap();
    let response = app.resolve(request).wait().unwrap();

    assert!(String::from_utf8(response.response).unwrap() == "world");
  }

  #[test]
  fn it_should_execute_all_middlware_with_a_given_request_with_params() {
    let mut app = App::<BasicContext>::new();

    fn test_fn_1(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      Box::new(future::ok(BasicContext {
        body: context.params.get("id").unwrap().to_owned(),
        params: context.params,
        query_params: context.query_params
      }))
    };

    app.get("/test/:id", vec![test_fn_1]);

    let mut bytes = BytesMut::with_capacity(45);
    bytes.put(&b"GET /test/123 HTTP/1.1\nHost: localhost:8080\n\n"[..]);


    let request = decode(&mut bytes).unwrap().unwrap();
    let response = app.resolve(request).wait().unwrap();

    assert!(String::from_utf8(response.response).unwrap() == "123");
  }

  #[test]
  fn it_should_execute_all_middlware_with_a_given_request_with_params_in_a_subapp() {
    let mut app1 = App::<BasicContext>::new();

    fn test_fn_1(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      Box::new(future::ok(BasicContext {
        body: context.params.get("id").unwrap().to_owned(),
        params: context.params,
        query_params: context.query_params
      }))
    };

    app1.get("/:id", vec![test_fn_1]);

    let mut app2 = App::<BasicContext>::new();
    app2.use_sub_app("/test", app1);

    let mut bytes = BytesMut::with_capacity(45);
    bytes.put(&b"GET /test/123 HTTP/1.1\nHost: localhost:8080\n\n"[..]);


    let request = decode(&mut bytes).unwrap().unwrap();
    let response = app2.resolve(request).wait().unwrap();

    assert!(String::from_utf8(response.response).unwrap() == "123");
  }

  #[test]
  fn it_should_correctly_parse_params_in_subapps() {
    let mut app1 = App::<BasicContext>::new();

    fn test_fn_1(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      Box::new(future::ok(BasicContext {
        body: context.params.get("id").unwrap().to_owned(),
        params: context.params,
        query_params: context.query_params
      }))
    };

    app1.get("/:id", vec![test_fn_1]);

    let mut app2 = App::<BasicContext>::new();
    app2.use_sub_app("/test", app1);

    let mut bytes = BytesMut::with_capacity(45);
    bytes.put(&b"GET /test/123 HTTP/1.1\nHost: localhost:8080\n\n"[..]);


    let request = decode(&mut bytes).unwrap().unwrap();
    let response = app2.resolve(request).wait().unwrap();

    assert!(String::from_utf8(response.response).unwrap() == "123");
  }

  #[test]
  fn it_should_match_as_far_as_possible_in_a_subapp() {
    let mut app1 = App::<BasicContext>::new();

    fn test_fn_1(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> MiddlewareReturnValue<BasicContext> {
      Box::new(future::ok(BasicContext {
        body: context.params.get("id").unwrap().to_owned(),
        params: context.params,
        query_params: context.query_params
      }))
    };

    fn test_fn_2(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> MiddlewareReturnValue<BasicContext> {
      Box::new(future::ok(BasicContext {
        body: "-1".to_owned(),
        params: context.params,
        query_params: context.query_params
      }))
    }

    app1.get("/", vec![test_fn_2]);
    app1.get("/:id", vec![test_fn_1]);

    let mut app2 = App::<BasicContext>::new();
    app2.use_sub_app("/test", app1);

    let mut bytes = BytesMut::with_capacity(45);
    bytes.put(&b"GET /test/123 HTTP/1.1\nHost: localhost:8080\n\n"[..]);

    let request = decode(&mut bytes).unwrap().unwrap();
    let response = app2.resolve(request).wait().unwrap();

    assert!(String::from_utf8(response.response).unwrap() == "123");

    let mut bytes = BytesMut::with_capacity(41);
    bytes.put(&b"GET /test HTTP/1.1\nHost: localhost:8080\n\n"[..]);

    let request = decode(&mut bytes).unwrap().unwrap();
    let response = app2.resolve(request).wait().unwrap();

    assert!(String::from_utf8(response.response).unwrap() == "-1");
  }

  #[test]
  fn it_should_trim_trailing_slashes() {
    let mut app1 = App::<BasicContext>::new();

    fn test_fn_1(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> MiddlewareReturnValue<BasicContext> {
      Box::new(future::ok(BasicContext {
        body: context.params.get("id").unwrap().to_owned(),
        params: context.params,
        query_params: context.query_params
      }))
    };

    fn test_fn_2(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> MiddlewareReturnValue<BasicContext> {
      Box::new(future::ok(BasicContext {
        body: "-1".to_owned(),
        params: context.params,
        query_params: context.query_params
      }))
    }

    app1.get("/:id", vec![test_fn_1]);

    let mut app2 = App::<BasicContext>::new();
    app2.use_sub_app("/test", app1);
    app2.set404(vec![test_fn_2]);

    let mut bytes = BytesMut::with_capacity(42);
    bytes.put(&b"GET /test/ HTTP/1.1\nHost: localhost:8080\n\n"[..]);

    let request = decode(&mut bytes).unwrap().unwrap();
    let response = app2.resolve(request).wait().unwrap();

    assert!(String::from_utf8(response.response).unwrap() == "-1");
  }

  #[test]
  fn it_should_be_able_to_parse_an_incoming_body() {
    fn generate_context_with_body(request: Request) -> TypedContext<TestStruct> {
      TypedContext::<TestStruct>::new(&request)
    }

    let mut app = App::create(generate_context_with_body);

    #[derive(Deserialize, Serialize)]
    struct TestStruct {
      key: String
    };

    fn test_fn_1(mut context: TypedContext<TestStruct>, _chain: &MiddlewareChain<TypedContext<TestStruct>>) -> Box<Future<Item=TypedContext<TestStruct>, Error=io::Error> + Send> {
      let value = context.request_body.key.clone();

      context.set_body(value);

      Box::new(future::ok(context))
    };

    app.post("/test", vec![test_fn_1]);

    let mut bytes = BytesMut::with_capacity(76);
    bytes.put(&b"POST /test HTTP/1.1\nHost: localhost:8080\nContent-Length: 15\n\n{\"key\":\"value\"}"[..]);


    let request = decode(&mut bytes).unwrap().unwrap();
    let response = app.resolve(request).wait().unwrap();

    assert!(String::from_utf8(response.response).unwrap() == "value");
  }

  #[test]
  fn it_should_execute_all_middlware_with_a_given_request_based_on_method() {
    let mut app = App::<BasicContext>::new();

    fn test_fn_1(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      Box::new(future::ok(BasicContext {
        body: format!("{}{}", context.body, "1"),
        params: HashMap::new(),
        query_params: HashMap::new()
      }))
    };

    fn test_fn_2(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      Box::new(future::ok(BasicContext {
        body: format!("{}{}", context.body, "2"),
        params: HashMap::new(),
        query_params: HashMap::new()
      }))
    };

    app.get("/test", vec![test_fn_1]);
    app.post("/test", vec![test_fn_2]);

    let mut bytes = BytesMut::with_capacity(41);
    bytes.put(&b"GET /test HTTP/1.1\nHost: localhost:8080\n\n"[..]);


    let request = decode(&mut bytes).unwrap().unwrap();
    let response = app.resolve(request).wait().unwrap();

    assert!(String::from_utf8(response.response).unwrap() == "1");
  }

  #[test]
  fn it_should_execute_all_middlware_with_a_given_request_up_and_down() {
    let mut app = App::<BasicContext>::new();

    fn test_fn_1(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      Box::new(future::ok(BasicContext {
        body: format!("{}{}", context.body, "1"),
        params: HashMap::new(),
        query_params: HashMap::new()
      }))
    };

    fn test_fn_2(context: BasicContext, chain: &MiddlewareChain<BasicContext>) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      let mut _context = BasicContext {
        body: format!("{}{}", context.body, "2"),
        params: HashMap::new(),
        query_params: HashMap::new()
      };

      let context_with_body = chain.next(_context)
        .and_then(|mut _context| {
          _context.body = format!("{}{}", _context.body, "2");
          future::ok(_context)
        });

      Box::new(context_with_body)
    };

    app.get("/test", vec![test_fn_2, test_fn_1]);

    let mut bytes = BytesMut::with_capacity(41);
    bytes.put(&b"GET /test HTTP/1.1\nHost: localhost:8080\n\n"[..]);


    let request = decode(&mut bytes).unwrap().unwrap();
    let response = app.resolve(request).wait().unwrap();

    assert!(String::from_utf8(response.response).unwrap() == "212");
  }

  #[test]
  fn it_should_return_whatever_was_set_as_the_body_of_the_context() {
    let mut app = App::<BasicContext>::new();

    fn test_fn_1(_context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      Box::new(future::ok(BasicContext {
        body: "Hello world".to_string(),
        params: HashMap::new(),
        query_params: HashMap::new()
      }))
    };

    app.get("/test", vec![test_fn_1]);

    let mut bytes = BytesMut::with_capacity(41);
    bytes.put(&b"GET /test HTTP/1.1\nHost: localhost:8080\n\n"[..]);


    let request = decode(&mut bytes).unwrap().unwrap();
    let response = app.resolve(request).wait().unwrap();

    assert!(String::from_utf8(response.response).unwrap() == "Hello world");
  }

  #[test]
  fn it_should_first_run_use_then_methods() {
    let mut app = App::<BasicContext>::new();

    fn method_agnostic(_context: BasicContext, chain: &MiddlewareChain<BasicContext>) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      let updated_context = chain.next(BasicContext {
        body: "agnostic".to_owned(),
        params: HashMap::new(),
        query_params: HashMap::new()
      });

      let body_with_copied_context = updated_context
          .and_then(|context| {
            future::ok(BasicContext {
              body: context.body,
              params: HashMap::new(),
              query_params: HashMap::new()
            })
          });

      Box::new(body_with_copied_context)
    }

    fn test_fn_1(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      Box::new(future::ok(BasicContext {
        body: format!("{}-1", context.body),
        params: HashMap::new(),
        query_params: HashMap::new()
      }))
    };

    app.use_middleware("/", method_agnostic);
    app.get("/test", vec![test_fn_1]);

    let mut bytes = BytesMut::with_capacity(41);
    bytes.put(&b"GET /test HTTP/1.1\nHost: localhost:8080\n\n"[..]);


    let request = decode(&mut bytes).unwrap().unwrap();
    let response = app.resolve(request).wait().unwrap();

    assert!(String::from_utf8(response.response).unwrap() == "agnostic-1");
  }

  #[test]
  fn it_should_be_able_to_correctly_route_sub_apps() {
    let mut app1 = App::<BasicContext>::new();

    fn test_fn_1(_context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      Box::new(future::ok(BasicContext {
        body: "1".to_string(),
        params: HashMap::new(),
        query_params: HashMap::new()
      }))
    };

    app1.get("/test", vec![test_fn_1]);

    let mut app2 = App::<BasicContext>::new();
    app2.use_sub_app("/", app1);

    let mut bytes = BytesMut::with_capacity(41);
    bytes.put(&b"GET /test HTTP/1.1\nHost: localhost:8080\n\n"[..]);


    let request = decode(&mut bytes).unwrap().unwrap();
    let response = app2.resolve(request).wait().unwrap();

    assert!(String::from_utf8(response.response).unwrap() == "1");
  }

  #[test]
  fn it_should_be_able_to_correctly_prefix_route_sub_apps() {
    let mut app1 = App::<BasicContext>::new();

    fn test_fn_1(_context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      Box::new(future::ok(BasicContext {
        body: "1".to_string(),
        params: HashMap::new(),
        query_params: HashMap::new()
      }))
    };

    app1.get("/test", vec![test_fn_1]);

    let mut app2 = App::<BasicContext>::new();
    app2.use_sub_app("/sub", app1);

    let mut bytes = BytesMut::with_capacity(45);
    bytes.put(&b"GET /sub/test HTTP/1.1\nHost: localhost:8080\n\n"[..]);


    let request = decode(&mut bytes).unwrap().unwrap();
    let response = app2.resolve(request).wait().unwrap();

    assert!(String::from_utf8(response.response).unwrap() == "1");
  }

  #[test]
  fn it_should_be_able_to_correctly_prefix_the_root_of_sub_apps() {
    let mut app1 = App::<BasicContext>::new();

    fn test_fn_1(_context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      Box::new(future::ok(BasicContext {
        body: "1".to_string(),
        params: HashMap::new(),
        query_params: HashMap::new()
      }))
    };

    app1.get("/", vec![test_fn_1]);

    let mut app2 = App::<BasicContext>::new();
    app2.use_sub_app("/sub", app1);

    let mut bytes = BytesMut::with_capacity(45);
    bytes.put(&b"GET /sub HTTP/1.1\nHost: localhost:8080\n\n"[..]);

    let request = decode(&mut bytes).unwrap().unwrap();
    let response = app2.resolve(request).wait().unwrap();

    assert!(String::from_utf8(response.response).unwrap() == "1");
  }

  #[test]
  fn it_should_be_able_to_correctly_handle_not_found_routes() {
    let mut app = App::<BasicContext>::new();

    fn test_fn_1(_context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      Box::new(future::ok(BasicContext {
        body: "1".to_string(),
        params: HashMap::new(),
        query_params: HashMap::new()
      }))
    };

    fn test_404(_context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      Box::new(future::ok(BasicContext {
        body: "not found".to_string(),
        params: HashMap::new(),
        query_params: HashMap::new()
      }))
    };

    app.get("/", vec![test_fn_1]);
    app.set404(vec![test_404]);

    let mut bytes = BytesMut::with_capacity(51);
    bytes.put(&b"GET /not_found HTTP/1.1\nHost: localhost:8080\n\n"[..]);


    let request = decode(&mut bytes).unwrap().unwrap();
    let response = app.resolve(request).wait().unwrap();

    assert!(String::from_utf8(response.response).unwrap() == "not found");
  }
}
