use std::io;

#[cfg(not(feature = "thruster_async_await"))]
use futures::future;

use futures::{Future as FutureLegacy};
use thruster_core::context::Context;
use thruster_core::request::{Request, RequestWithParams};
use thruster_core::route_parser::{MatchedRoute, RouteParser};
use thruster_core::middleware::{MiddlewareChain};

use thruster_context::basic_context::{generate_context, BasicContext};

enum Method {
  DELETE,
  GET,
  OPTIONS,
  POST,
  PUT,
  UPDATE
}

// Warning, this method is slow and shouldn't be used for route matching, only for route adding
fn _add_method_to_route(method: &Method, path: &str) -> String {
  let prefix = match method {
    Method::DELETE => "__DELETE__",
    Method::GET => "__GET__",
    Method::OPTIONS => "__OPTIONS__",
    Method::POST => "__POST__",
    Method::PUT => "__PUT__",
    Method::UPDATE => "__UPDATE__"
  };

  match &path[0..1] {
    "/" => format!("{}{}", prefix, path),
    _ => format!("{}/{}", prefix, path)
  }
}

#[inline]
fn _add_method_to_route_from_str(method: &str, path: &str) -> String {
  templatify!("__" ; method ; "__" ; path ; "")
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
/// let mut app1 = App::<Request, BasicContext>::new();
///
/// fn test_fn_1(context: BasicContext, next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> MiddlewareReturnValue<BasicContext> {
///   Box::new(future::ok(BasicContext {
///     body: context.params.get("id").unwrap().to_owned(),
///     params: context.params,
///     query_params: context.query_params
///   }))
/// };
///
/// app1.get("/:id", middleware![BasicContext => test_fn_1]);
///
/// let mut app2 = App::<Request, BasicContext>::new();
/// app2.use_sub_app("/test", &app1);
/// ```
///
/// In the above example, the route `/test/some-id` will return `some-id` in the body of the response.
///
/// The provided start methods are great places to start, but you can also simply use Thruster as a router
/// and create your own version of an HTTP server by directly calling `App.resolve` with a Request object.
/// It will then return a future with the Response object that corresponds to the request. This can be
/// useful if trying to integrate with a different type of load balancing system within the threads of the
/// application.
///
pub struct App<R: RequestWithParams, T: 'static + Context + Send> {
  pub _route_parser: RouteParser<T>,
  ///
  /// Generate context is common to all `App`s. It's the function that's called upon receiving a request
  /// that translates an acutal `Request` struct to your custom Context type. It should be noted that
  /// the context_generator should be as fast as possible as this is called with every request, including
  /// 404s.
  pub context_generator: fn(R) -> T
}

impl<R: RequestWithParams, T: Context + Send> App<R, T> {
  /// Creates a new instance of app with the library supplied `BasicContext`. Useful for trivial
  /// examples, likely not a good solution for real code bases. The advantage is that the
  /// context_generator is already supplied for the developer.
  pub fn new_basic() -> App<Request, BasicContext> {
    App::create(generate_context)
  }

  /// Create a new app with the given context generator. The app does not begin listening until start
  /// is called.
  pub fn create(generate_context: fn(R) -> T) -> App<R, T> {
    App {
      _route_parser: RouteParser::new(),
      context_generator: generate_context
    }
  }

  /// Add method-agnostic middleware for a route. This is useful for applying headers, logging, and
  /// anything else that might not be sensitive to the HTTP method for the endpoint.
  pub fn use_middleware(&mut self, path: &'static str, middleware: MiddlewareChain<T>) -> &mut App<R, T> {
    self._route_parser.add_method_agnostic_middleware(path, middleware);

    self
  }

  /// Add an app as a predetermined set of routes and middleware. Will prefix whatever string is passed
  /// in to all of the routes. This is a main feature of Thruster, as it allows projects to be extermely
  /// modular and composeable in nature.
  pub fn use_sub_app(&mut self, prefix: &'static str, app: App<R, T>) -> &mut App<R, T> {
    self._route_parser.route_tree
      .add_route_tree(prefix, app._route_parser.route_tree);

    self
  }

  /// Return the route parser for a given app
  pub fn get_route_parser(&self) -> &RouteParser<T> {
    &self._route_parser
  }

  /// Add a route that responds to `GET`s to a given path
  pub fn get(&mut self, path: &'static str, middlewares: MiddlewareChain<T>) -> &mut App<R, T> {
    self._route_parser.add_route(
      &_add_method_to_route(&Method::GET, path), middlewares);

    self
  }

  /// Add a route that responds to `OPTION`s to a given path
  pub fn options(&mut self, path: &'static str, middlewares: MiddlewareChain<T>) -> &mut App<R, T> {
    self._route_parser.add_route(
      &_add_method_to_route(&Method::OPTIONS, path), middlewares);

    self
  }

  /// Add a route that responds to `POST`s to a given path
  pub fn post(&mut self, path: &'static str, middlewares: MiddlewareChain<T>) -> &mut App<R, T> {
    self._route_parser.add_route(
      &_add_method_to_route(&Method::POST, path), middlewares);

    self
  }

  /// Add a route that responds to `PUT`s to a given path
  pub fn put(&mut self, path: &'static str, middlewares: MiddlewareChain<T>) -> &mut App<R, T> {
    self._route_parser.add_route(
      &_add_method_to_route(&Method::PUT, path), middlewares);

    self
  }

  /// Add a route that responds to `DELETE`s to a given path
  pub fn delete(&mut self, path: &'static str, middlewares: MiddlewareChain<T>) -> &mut App<R, T> {
    self._route_parser.add_route(
      &_add_method_to_route(&Method::DELETE, path), middlewares);

    self
  }

  /// Add a route that responds to `UPDATE`s to a given path
  pub fn update(&mut self, path: &'static str, middlewares: MiddlewareChain<T>) -> &mut App<R, T> {
    self._route_parser.add_route(
      &_add_method_to_route(&Method::UPDATE, path), middlewares);

    self
  }

  /// Sets the middleware if no route is successfully matched.
  pub fn set404(&mut self, middlewares: MiddlewareChain<T>) -> &mut App<R, T> {
    self._route_parser.add_route(
      &_add_method_to_route(&Method::GET, "/*"), middlewares.clone());
    self._route_parser.add_route(
      &_add_method_to_route(&Method::POST, "/*"), middlewares.clone());
    self._route_parser.add_route(
      &_add_method_to_route(&Method::PUT, "/*"), middlewares.clone());
    self._route_parser.add_route(
      &_add_method_to_route(&Method::UPDATE, "/*"), middlewares.clone());
    self._route_parser.add_route(
      &_add_method_to_route(&Method::DELETE, "/*"), middlewares);

    self
  }

  pub fn resolve_from_method_and_path(&self, method: &str, path: &str) -> MatchedRoute<T> {
    let path_with_method = &_add_method_to_route_from_str(method, path);

    self._route_parser.match_route(path_with_method)
  }

  /// Resolves a request, returning a future that is processable into a Response
  pub fn resolve(&self, request: R, matched_route: MatchedRoute<T>) -> impl FutureLegacy<Item=T::Response, Error=io::Error> + Send {
    self._resolve(request, matched_route)
  }

  #[cfg(feature = "thruster_async_await")]
  fn _resolve(&self, request: R, matched_route: MatchedRoute<T>) -> impl FutureLegacy<Item=T::Response, Error=io::Error> + Send {
    use thruster_async_await::resolve;

    resolve(self.context_generator, request, matched_route)
  }

  #[cfg(not(feature = "thruster_async_await"))]
  fn _resolve(&self, mut request: R, matched_route: MatchedRoute<T>) -> impl FutureLegacy<Item=T::Response, Error=io::Error> + Send {
    request.set_params(matched_route.params);

    let context = (self.context_generator)(request);
    let context_future = matched_route.middleware.run(context);

    #[cfg(feature = "thruster_error_handling")]
    let context_future = context_future.unwrap();

    context_future
        .and_then(|context| {
          future::ok(context.get_response())
        })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::testing;

  use futures::{future, Future};
  use std::boxed::Box;
  use std::io;
  use std::marker::Send;

  use thruster_core::request::Request;
  use thruster_core::middleware::{MiddlewareChain, MiddlewareReturnValue};
  use thruster_middleware::query_params;
  use thruster_middleware::cookies;
  use thruster_context::basic_context::BasicContext;

  #[cfg(feature = "thruster_async_await")]
  use thruster_proc::{async_middleware, middleware_fn};

  #[test]
  fn it_should_execute_all_middlware_with_a_given_request() {
    let mut app = App::<Request, BasicContext>::new_basic();

    fn test_fn_1(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      context.body("1");
      Box::new(future::ok(context))
    };

    app.use_middleware("/", middleware![BasicContext => query_params::query_params]);
    app.get("/test", middleware![BasicContext => test_fn_1]);

    let response = testing::get(&app, "/test");

    assert!(response.body == "1");
  }

  #[test]
  fn it_should_correctly_differentiate_wildcards_and_valid_routes() {
    let mut app = App::<Request, BasicContext>::new_basic();

    fn test_fn_1(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      context.body("1");
      Box::new(future::ok(context))
    };

    fn test_fn_404(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      context.body("2");
      Box::new(future::ok(context))
    };


    app.get("/", middleware![BasicContext => test_fn_1]);
    app.get("/*", middleware![BasicContext => test_fn_404]);

    let response = testing::get(&app, "/");

    assert!(response.body == "1");
  }

  #[test]
  fn it_should_handle_query_parameters() {
    let mut app = App::<Request, BasicContext>::new_basic();

    fn test_fn_1(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      let body = &context.query_params.get("hello").unwrap().clone();

      context.body(body);
      Box::new(future::ok(context))
    };

    app.use_middleware("/", middleware![BasicContext => query_params::query_params]);
    app.get("/test", middleware![BasicContext => test_fn_1]);

    let response = testing::get(&app, "/test?hello=world");

    assert!(response.body == "world");
  }

  #[test]
  fn it_should_handle_cookies() {
    let mut app = App::<Request, BasicContext>::new_basic();

    fn test_fn_1(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      let body = match context.cookies.get(0) {
        Some(cookie) => {
          assert!(cookie.options.same_site.is_some());
          assert!(cookie.options.same_site.as_ref().unwrap() == &cookies::SameSite::Strict);
          cookie.value.clone()
        },
        None => "".to_owned()
      };

      context.body(&body);
      Box::new(future::ok(context))
    };

    app.use_middleware("/", middleware![BasicContext => cookies::cookies]);
    app.get("/test", middleware![BasicContext => test_fn_1]);

    let response = testing::request(&app, "GET", "/test", &vec![("Set-Cookie", "Hello=World; SameSite=Strict")], "");

    assert!(response.body == "World");
  }

  #[test]
  fn it_should_execute_all_middlware_with_a_given_request_with_params() {
    let mut app = App::<Request, BasicContext>::new_basic();

    fn test_fn_1(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      let body = &context.params.get("id").unwrap().clone();

      context.body(body);
      Box::new(future::ok(context))
    };

    app.get("/test/:id", middleware![BasicContext => test_fn_1]);

    let response = testing::get(&app, "/test/123");

    assert!(response.body == "123");
  }

  #[test]
  fn it_should_execute_all_middlware_with_a_given_request_with_params_in_a_subapp() {
    let mut app1 = App::<Request, BasicContext>::new_basic();

    fn test_fn_1(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      let body = &context.params.get("id").unwrap().clone();

      context.body(body);
      Box::new(future::ok(context))
    };

    app1.get("/:id", middleware![BasicContext => test_fn_1]);

    let mut app2 = App::<Request, BasicContext>::new_basic();
    app2.use_sub_app("/test", app1);

    let response = testing::get(&app2, "/test/123");

    assert!(response.body == "123");
  }

  #[test]
  fn it_should_correctly_parse_params_in_subapps() {
    let mut app1 = App::<Request, BasicContext>::new_basic();

    fn test_fn_1(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      let body = &context.params.get("id").unwrap().clone();

      context.body(body);
      Box::new(future::ok(context))
    };

    app1.get("/:id", middleware![BasicContext => test_fn_1]);

    let mut app2 = App::<Request, BasicContext>::new_basic();
    app2.use_sub_app("/test", app1);

    let response = testing::get(&app2, "/test/123");

    assert!(response.body == "123");
  }

  #[test]
  fn it_should_match_as_far_as_possible_in_a_subapp() {
    let mut app1 = App::<Request, BasicContext>::new_basic();

    fn test_fn_1(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> MiddlewareReturnValue<BasicContext> {
      let body = &context.params.get("id").unwrap().clone();

      context.body(body);
      Box::new(future::ok(context))
    };

    fn test_fn_2(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> MiddlewareReturnValue<BasicContext> {
      context.body("-1");
      Box::new(future::ok(context))
    }

    app1.get("/", middleware![BasicContext => test_fn_2]);
    app1.get("/:id", middleware![BasicContext => test_fn_1]);

    let mut app2 = App::<Request, BasicContext>::new_basic();
    app2.use_sub_app("/test", app1);

    let response = testing::get(&app2, "/test/123");

    assert!(response.body == "123");
  }

  #[test]
  fn it_should_trim_trailing_slashes() {
    let mut app1 = App::<Request, BasicContext>::new_basic();

    fn test_fn_1(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> MiddlewareReturnValue<BasicContext> {
      let body = &context.params.get("id").unwrap().clone();

      context.body(body);
      Box::new(future::ok(context))
    };

    fn test_fn_2(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> MiddlewareReturnValue<BasicContext> {
      context.body("-1");
      Box::new(future::ok(context))
    }

    app1.get("/:id", middleware![BasicContext => test_fn_1]);

    let mut app2 = App::<Request, BasicContext>::new_basic();
    app2.use_sub_app("/test", app1);
    app2.set404(middleware![BasicContext => test_fn_2]);

    let response = testing::get(&app2, "/test/");

    assert!(response.body == "-1");
  }

  #[test]
  fn it_should_trim_trailing_slashes_after_params() {
    let mut app = App::<Request, BasicContext>::new_basic();

    fn test_fn_1(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> MiddlewareReturnValue<BasicContext> {
      let body = &context.params.get("id").unwrap().clone();

      context.body(body);
      Box::new(future::ok(context))
    };

    app.get("/test/:id", middleware![BasicContext => test_fn_1]);

    let response = testing::get(&app, "/test/1/");

    assert!(response.body == "1");
  }

  #[test]
  fn it_should_execute_all_middlware_with_a_given_request_based_on_method() {
    let mut app = App::<Request, BasicContext>::new_basic();

    fn test_fn_1(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      let existing_body = context.get_body().clone();

      context.body(&format!("{}{}", existing_body, "1"));
      Box::new(future::ok(context))
    };

    fn test_fn_2(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      let existing_body = context.get_body().clone();

      context.body(&format!("{}{}", existing_body, "2"));
      Box::new(future::ok(context))
    };

    app.get("/test", middleware![BasicContext => test_fn_1]);
    app.post("/test", middleware![BasicContext => test_fn_2]);

    let response = testing::get(&app, "/test");

    assert!(response.body == "1");
  }

  #[test]
  fn it_should_execute_all_middlware_with_a_given_request_up_and_down() {
    let mut app = App::<Request, BasicContext>::new_basic();

    fn test_fn_1(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      let existing_body = context.get_body().clone();

      context.body(&format!("{}{}", existing_body, "1"));
      Box::new(future::ok(context))
    };

    fn test_fn_2(mut context: BasicContext, next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      let existing_body = context.get_body().clone();

      context.body(&format!("{}{}", existing_body, "2"));

      let context_with_body = next(context)
        .and_then(|mut _context| {
          let _existing_body = _context.get_body().clone();

          _context.body(&format!("{}{}", _existing_body, "2"));
          future::ok(_context)
        });

      Box::new(context_with_body)
    };

    app.get("/test", middleware![BasicContext => test_fn_2, BasicContext => test_fn_1]);

    let response = testing::get(&app, "/test");

    assert!(response.body == "212");
  }

  #[test]
  fn it_should_return_whatever_was_set_as_the_body_of_the_context() {
    let mut app = App::<Request, BasicContext>::new_basic();

    fn test_fn_1(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      context.body("Hello world");
      Box::new(future::ok(context))
    };

    app.get("/test", middleware![BasicContext => test_fn_1]);

    let response = testing::get(&app, "/test");

    assert!(response.body == "Hello world");
  }

  #[test]
  fn it_should_first_run_use_then_methods() {
    let mut app = App::<Request, BasicContext>::new_basic();

    fn method_agnostic(mut context: BasicContext, next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      context.body("agnostic");
      let updated_context = next(context);

      let body_with_copied_context = updated_context
          .and_then(|context| {
            future::ok(context)
          });

      Box::new(body_with_copied_context)
    }

    fn test_fn_1(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      let body = context.get_body().clone();

      context.body(&format!("{}-1", body));
      Box::new(future::ok(context))
    };

    app.use_middleware("/", middleware![BasicContext => method_agnostic]);
    app.get("/test", middleware![BasicContext => test_fn_1]);

    let response = testing::get(&app, "/test");

    assert!(response.body == "agnostic-1");
  }

  #[test]
  fn it_should_be_able_to_correctly_route_sub_apps() {
    let mut app1 = App::<Request, BasicContext>::new_basic();

    fn test_fn_1(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      context.body("1");
      Box::new(future::ok(context))
    };

    app1.get("/test", middleware![BasicContext => test_fn_1]);

    let mut app2 = App::<Request, BasicContext>::new_basic();
    app2.use_sub_app("/", app1);

    let response = testing::get(&app2, "/test");

    assert!(response.body == "1");
  }

  #[test]
  fn it_should_be_able_to_correctly_route_sub_apps_with_wildcards() {
    let mut app1 = App::<Request, BasicContext>::new_basic();

    fn test_fn_1(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      context.body("1");
      Box::new(future::ok(context))
    };

    app1.get("/*", middleware![BasicContext => test_fn_1]);

    let mut app2 = App::<Request, BasicContext>::new_basic();
    app2.use_sub_app("/", app1);

    let response = testing::get(&app2, "/a");

    assert!(response.body == "1");
  }

  #[test]
  fn it_should_be_able_to_correctly_prefix_route_sub_apps() {
    let mut app1 = App::<Request, BasicContext>::new_basic();

    fn test_fn_1(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      context.body("1");
      Box::new(future::ok(context))
    };

    app1.get("/test", middleware![BasicContext => test_fn_1]);

    let mut app2 = App::<Request, BasicContext>::new_basic();
    app2.use_sub_app("/sub", app1);

    let response = testing::get(&app2, "/sub/test");

    assert!(response.body == "1");
  }

  #[test]
  fn it_should_be_able_to_correctly_prefix_the_root_of_sub_apps() {
    let mut app1 = App::<Request, BasicContext>::new_basic();

    fn test_fn_1(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      context.body("1");
      Box::new(future::ok(context))
    };

    app1.get("/", middleware![BasicContext => test_fn_1]);

    let mut app2 = App::<Request, BasicContext>::new_basic();
    app2.use_sub_app("/sub", app1);

    let response = testing::get(&app2, "/sub");

    assert!(response.body == "1");
  }

  #[test]
  fn it_should_be_able_to_correctly_handle_not_found_routes() {
    let mut app = App::<Request, BasicContext>::new_basic();

    fn test_fn_1(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      context.body("1");
      Box::new(future::ok(context))
    };

    fn test_404(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      context.body("not found");
      Box::new(future::ok(context))
    };

    app.get("/", middleware![BasicContext => test_fn_1]);
    app.set404(middleware![BasicContext => test_404]);

    let response = testing::get(&app, "/not_found");

    assert!(response.body == "not found");
  }


  #[test]
  fn it_should_be_able_to_correctly_handle_not_found_at_the_root() {
    let mut app = App::<Request, BasicContext>::new_basic();

    fn test_fn_1(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      context.body("1");
      Box::new(future::ok(context))
    };

    fn test_404(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      context.body("not found");
      Box::new(future::ok(context))
    };

    app.get("/a", middleware![BasicContext => test_fn_1]);
    app.set404(middleware![BasicContext => test_404]);

    let response = testing::get(&app, "/");

    assert!(response.body == "not found");
  }

  #[test]
  fn it_should_be_able_to_correctly_handle_deep_not_found_routes() {
    let mut app = App::<Request, BasicContext>::new_basic();

    fn test_fn_1(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      context.body("1");
      Box::new(future::ok(context))
    };

    fn test_404(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      context.body("not found");
      Box::new(future::ok(context))
    };

    app.get("/a/b", middleware![BasicContext => test_fn_1]);
    app.set404(middleware![BasicContext => test_404]);

    let response = testing::get(&app, "/a/not_found/");

    assert!(response.body == "not found");
  }

  #[test]
  fn it_should_be_able_to_correctly_handle_deep_not_found_routes_after_paramaterized_routes() {
    let mut app = App::<Request, BasicContext>::new_basic();

    fn test_fn_1(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      context.body("1");
      Box::new(future::ok(context))
    };

    fn test_404(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      context.body("not found");
      Box::new(future::ok(context))
    };

    app.get("/a/:b/c", middleware![BasicContext => test_fn_1]);
    app.set404(middleware![BasicContext => test_404]);

    let response = testing::get(&app, "/a/1/d");

    assert!(response.body == "not found");
  }

  #[test]
  fn it_should_be_able_to_correctly_handle_deep_not_found_routes_after_paramaterized_routes_with_extra_pieces() {
    let mut app = App::<Request, BasicContext>::new_basic();

    fn test_fn_1(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      context.body("1");
      Box::new(future::ok(context))
    };

    fn test_404(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      context.body("not found");
      Box::new(future::ok(context))
    };

    app.get("/a/:b/c", middleware![BasicContext => test_fn_1]);
    app.set404(middleware![BasicContext => test_404]);

    let response = testing::get(&app, "/a/1/d/e/f/g");

    assert!(response.body == "not found");
  }

#[test]
  fn it_should_be_able_to_correctly_handle_deep_not_found_routes_after_root_route() {
    let mut app1 = App::<Request, BasicContext>::new_basic();
    let mut app2 = App::<Request, BasicContext>::new_basic();
    let mut app3 = App::<Request, BasicContext>::new_basic();

    fn test_fn_1(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      context.body("1");
      Box::new(future::ok(context))
    };

    fn test_404(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      context.body("not found");
      Box::new(future::ok(context))
    };

    app1.get("/", middleware![BasicContext => test_fn_1]);
    app2.get("*", middleware![BasicContext => test_404]);
    app3.use_sub_app("/", app2);
    app3.use_sub_app("/a", app1);

    let response = testing::get(&app3, "/a/1/d");

    assert!(response.body == "not found");
  }

  #[test]
  fn it_should_handle_routes_without_leading_slash() {
    let mut app = App::<Request, BasicContext>::new_basic();

    fn test_fn_1(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      context.body("1");
      Box::new(future::ok(context))
    };

    app.get("*", middleware![BasicContext => test_fn_1]);

    let response = testing::get(&app, "/a/1/d/e/f/g");

    assert!(response.body == "1");
  }

  #[test]
  fn it_should_handle_routes_within_a_subapp_without_leading_slash() {
    let mut app1 = App::<Request, BasicContext>::new_basic();
    let mut app2 = App::<Request, BasicContext>::new_basic();

    fn test_fn_1(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      context.body("1");
      Box::new(future::ok(context))
    };

    app1.get("*", middleware![BasicContext => test_fn_1]);
    app2.use_sub_app("/", app1);

    let response = testing::get(&app2, "/a/1/d/e/f/g");

    assert!(response.body == "1");
  }

  #[test]
  fn it_should_handle_multiple_subapps_with_wildcards() {
    let mut app1 = App::<Request, BasicContext>::new_basic();
    let mut app2 = App::<Request, BasicContext>::new_basic();
    let mut app3 = App::<Request, BasicContext>::new_basic();

    fn test_fn_1(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      context.body("1");
      Box::new(future::ok(context))
    };

    fn test_fn_2(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> Box<Future<Item=BasicContext, Error=io::Error> + Send> {
      context.body("2");
      Box::new(future::ok(context))
    };

    app1.get("/*", middleware![BasicContext => test_fn_1]);
    app2.get("/*", middleware![BasicContext => test_fn_2]);
    app3.use_sub_app("/", app1);
    app3.use_sub_app("/a/b", app2);

    let response = testing::get(&app3, "/a/1/d/e/f/g");

    assert!(response.body == "1");
  }
}
