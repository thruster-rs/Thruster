use std::io;

use std::future::Future;
use crate::context::basic_context::{generate_context, BasicContext};
use crate::core::context::Context;
use crate::core::middleware::MiddlewareChain;
use crate::core::request::{Request, RequestWithParams};
use crate::core::route_parser::{MatchedRoute, RouteParser};
use crate::core::errors::Error;

enum Method {
    DELETE,
    GET,
    OPTIONS,
    POST,
    PUT,
    UPDATE,
}

// Warning, this method is slow and shouldn't be used for route matching, only for route adding
fn _add_method_to_route(method: &Method, path: &str) -> String {
    let prefix = match method {
        Method::DELETE => "__DELETE__",
        Method::GET => "__GET__",
        Method::OPTIONS => "__OPTIONS__",
        Method::POST => "__POST__",
        Method::PUT => "__PUT__",
        Method::UPDATE => "__UPDATE__",
    };

    match &path[0..1] {
        "/" => format!("{}{}", prefix, path),
        _ => format!("{}/{}", prefix, path),
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
/// 3. Build the app future with `App.build` and spawn it on the executor
///
/// # Examples
/// Subapp
///
/// ```rust, ignore
/// let mut app1 = App::<Request, BasicContext>::new();
///
/// fn test_fn_1(context: BasicContext, next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send) -> MiddlewareReturnValue<BasicContext> {
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
/// app2.use_sub_app("/test", app1);
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
    pub context_generator: fn(R) -> T,
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
            context_generator: generate_context,
        }
    }

    /// Add method-agnostic middleware for a route. This is useful for applying headers, logging, and
    /// anything else that might not be sensitive to the HTTP method for the endpoint.
    pub fn use_middleware(
        &mut self,
        path: &'static str,
        middleware: MiddlewareChain<T>,
    ) -> &mut App<R, T> {
        self._route_parser
            .add_method_agnostic_middleware(path, middleware);

        self
    }

    /// Add an app as a predetermined set of routes and middleware. Will prefix whatever string is passed
    /// in to all of the routes. This is a main feature of Thruster, as it allows projects to be extermely
    /// modular and composeable in nature.
    pub fn use_sub_app(&mut self, prefix: &'static str, app: App<R, T>) -> &mut App<R, T> {
        self._route_parser
            .route_tree
            .add_route_tree(prefix, app._route_parser.route_tree);

        self
    }

    /// Return the route parser for a given app
    pub fn get_route_parser(&self) -> &RouteParser<T> {
        &self._route_parser
    }

    /// Add a route that responds to `GET`s to a given path
    pub fn get(&mut self, path: &'static str, middlewares: MiddlewareChain<T>) -> &mut App<R, T> {
        self._route_parser
            .add_route(&_add_method_to_route(&Method::GET, path), middlewares);

        self
    }

    /// Add a route that responds to `OPTION`s to a given path
    pub fn options(
        &mut self,
        path: &'static str,
        middlewares: MiddlewareChain<T>,
    ) -> &mut App<R, T> {
        self._route_parser
            .add_route(&_add_method_to_route(&Method::OPTIONS, path), middlewares);

        self
    }

    /// Add a route that responds to `POST`s to a given path
    pub fn post(&mut self, path: &'static str, middlewares: MiddlewareChain<T>) -> &mut App<R, T> {
        self._route_parser
            .add_route(&_add_method_to_route(&Method::POST, path), middlewares);

        self
    }

    /// Add a route that responds to `PUT`s to a given path
    pub fn put(&mut self, path: &'static str, middlewares: MiddlewareChain<T>) -> &mut App<R, T> {
        self._route_parser
            .add_route(&_add_method_to_route(&Method::PUT, path), middlewares);

        self
    }

    /// Add a route that responds to `DELETE`s to a given path
    pub fn delete(
        &mut self,
        path: &'static str,
        middlewares: MiddlewareChain<T>,
    ) -> &mut App<R, T> {
        self._route_parser
            .add_route(&_add_method_to_route(&Method::DELETE, path), middlewares);

        self
    }

    /// Add a route that responds to `UPDATE`s to a given path
    pub fn update(
        &mut self,
        path: &'static str,
        middlewares: MiddlewareChain<T>,
    ) -> &mut App<R, T> {
        self._route_parser
            .add_route(&_add_method_to_route(&Method::UPDATE, path), middlewares);

        self
    }

    /// Sets the middleware if no route is successfully matched.
    pub fn set404(&mut self, middlewares: MiddlewareChain<T>) -> &mut App<R, T> {
        self._route_parser.add_route(
            &_add_method_to_route(&Method::GET, "/*"),
            middlewares.clone(),
        );
        self._route_parser.add_route(
            &_add_method_to_route(&Method::POST, "/*"),
            middlewares.clone(),
        );
        self._route_parser.add_route(
            &_add_method_to_route(&Method::PUT, "/*"),
            middlewares.clone(),
        );
        self._route_parser.add_route(
            &_add_method_to_route(&Method::UPDATE, "/*"),
            middlewares.clone(),
        );
        self._route_parser
            .add_route(&_add_method_to_route(&Method::DELETE, "/*"), middlewares);

        self
    }

    pub fn resolve_from_method_and_path(&self, method: &str, path: &str) -> MatchedRoute<T> {
        let path_with_method = &_add_method_to_route_from_str(method, path);

        self._route_parser.match_route(path_with_method)
    }

    /// Resolves a request, returning a future that is processable into a Response
    #[cfg(feature = "hyper_server")]
    pub fn resolve(
        &self,
        request: R,
        matched_route: MatchedRoute<T>,
    ) -> impl Future<Output = Result<T::Response, io::Error>> + Send {
        self._resolve(request, matched_route)
    }

    #[cfg(not(feature = "hyper_server"))]
    pub fn resolve(
        &self,
        request: R,
        matched_route: MatchedRoute<T>,
    ) -> impl Future<Output = Result<T::Response, io::Error>> + Send {
        self._resolve(request, matched_route)
    }

    fn _resolve(
        &self,
        request: R,
        matched_route: MatchedRoute<T>,
    ) -> impl Future<Output = Result<T::Response, io::Error>> + Send {
        request.set_params(matched_route.params);

        let context = (self.context_generator)(request);

        let copy = matched_route.middleware.clone();
        async move {
            let ctx = copy.run(context).await;

            let ctx = match ctx {
                Ok(val) => val,
                Err(e) => e.build_context(),
            };

            Ok(ctx.get_response())
        }
    }
}
