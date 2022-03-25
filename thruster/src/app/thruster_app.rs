use crate::{RequestWithParams, ReusableBoxFuture};
use futures::FutureExt;

use std::io;

use crate::core::context::Context;
use crate::core::request::Request;
use crate::parser::{middleware_traits::MiddlewareTuple, tree::Node, tree::NodeOutput};
use crate::{
    context::basic_context::{generate_context, BasicContext},
    core::request::ThrusterRequest,
};

pub trait Middleware<CIn: Send, COut: Send, CErr: Send>:
    FnOnce(CIn) -> ReusableBoxFuture<Result<COut, CErr>> + Send
{
    fn val(&self) -> Self
    where
        Self: Sized + Copy,
    {
        panic!("Never use this.")
    }
}

// type ReturnValue<T> = Result<T, ThrusterError<T>>;
type ReturnValue<T> = T;

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
pub struct App<R: ThrusterRequest, T: 'static + Context + Clone + Send + Sync, S: Send> {
    pub delete_root: Node<ReturnValue<T>>,
    pub get_root: Node<ReturnValue<T>>,
    pub options_root: Node<ReturnValue<T>>,
    pub post_root: Node<ReturnValue<T>>,
    pub put_root: Node<ReturnValue<T>>,
    pub patch_root: Node<ReturnValue<T>>,
    /// Generate context is common to all `App`s. It's the function that's called upon receiving a request
    /// that translates an acutal `Request` struct to your custom Context type. It should be noted that
    /// the context_generator should be as fast as possible as this is called with every request, including
    /// 404s.
    pub context_generator: fn(R, &S, &str) -> T,
    pub state: std::sync::Arc<S>,
    /// The connection timeout for the app in milliseconds. Defaults to 3600000ms (1 hour)
    pub connection_timeout: u64,
}

impl<R: 'static + ThrusterRequest, T: Context + Clone + Send + Sync, S: 'static + Send>
    App<R, T, S>
{
    /// Creates a new instance of app with the library supplied `BasicContext`. Useful for trivial
    /// examples, likely not a good solution for real code bases. The advantage is that the
    /// context_generator is already supplied for the developer.
    pub fn new_basic() -> App<Request, BasicContext, ()> {
        App::create(generate_context, ())
    }

    /// Create a new app with the given context generator. The app does not begin listening until start
    /// is called.
    pub fn create(generate_context: fn(R, &S, &str) -> T, state: S) -> Self {
        App {
            delete_root: Node::default(),
            get_root: Node::default(),
            options_root: Node::default(),
            post_root: Node::default(),
            put_root: Node::default(),
            patch_root: Node::default(),
            context_generator: generate_context,
            state: std::sync::Arc::new(state),
            connection_timeout: 3600000,
        }
    }

    /// Add method-agnostic middleware for a route. This is useful for applying headers, logging, and
    /// anything else that might not be sensitive to the HTTP method for the endpoint.
    pub fn use_middleware(
        mut self,
        path: &str,
        middlewares: MiddlewareTuple<ReturnValue<T>>,
    ) -> Self
    where
        T: Clone,
    {
        self.get_root
            .add_non_leaf_value_at_path(path, middlewares.clone());
        self.options_root
            .add_non_leaf_value_at_path(path, middlewares.clone());
        self.post_root
            .add_non_leaf_value_at_path(path, middlewares.clone());
        self.put_root
            .add_non_leaf_value_at_path(path, middlewares.clone());
        self.delete_root
            .add_non_leaf_value_at_path(path, middlewares.clone());
        self.patch_root
            .add_non_leaf_value_at_path(path, middlewares);

        self
    }

    /// Add an app as a predetermined set of routes and middleware. Will prefix whatever string is passed
    /// in to all of the routes. This is a main feature of Thruster, as it allows projects to be extermely
    /// modular and composeable in nature.
    pub fn use_sub_app(mut self, prefix: &str, app: App<R, T, S>) -> Self {
        self.get_root.add_node_at_path(prefix, app.get_root);
        self.options_root.add_node_at_path(prefix, app.options_root);
        self.post_root.add_node_at_path(prefix, app.post_root);
        self.put_root.add_node_at_path(prefix, app.put_root);
        self.delete_root.add_node_at_path(prefix, app.delete_root);
        self.patch_root.add_node_at_path(prefix, app.patch_root);

        self
    }

    /// Add a route that responds to `GET`s to a given path
    pub fn get(mut self, path: &str, middlewares: MiddlewareTuple<ReturnValue<T>>) -> Self {
        self.get_root.add_value_at_path(path, middlewares);

        self
    }

    /// Add a route that responds to `OPTION`s to a given path
    pub fn options(mut self, path: &str, middlewares: MiddlewareTuple<ReturnValue<T>>) -> Self {
        self.options_root.add_value_at_path(path, middlewares);

        self
    }

    /// Add a route that responds to `POST`s to a given path
    pub fn post(mut self, path: &str, middlewares: MiddlewareTuple<ReturnValue<T>>) -> Self {
        self.post_root.add_value_at_path(path, middlewares);

        self
    }

    /// Add a route that responds to `PUT`s to a given path
    pub fn put(mut self, path: &str, middlewares: MiddlewareTuple<ReturnValue<T>>) -> Self {
        self.put_root.add_value_at_path(path, middlewares);

        self
    }

    /// Add a route that responds to `DELETE`s to a given path
    pub fn delete(mut self, path: &str, middlewares: MiddlewareTuple<ReturnValue<T>>) -> Self {
        self.delete_root.add_value_at_path(path, middlewares);

        self
    }

    /// Add a route that responds to `PATCH`s to a given path
    pub fn patch(mut self, path: &str, middlewares: MiddlewareTuple<ReturnValue<T>>) -> Self {
        self.patch_root.add_value_at_path(path, middlewares);

        self
    }

    /// Sets the middleware if no route is successfully matched. Note, that due to type restrictions,
    /// we Context needs to implement Clone in order to call this function, even though _clone will
    /// never be called._
    pub fn set404(mut self, middlewares: MiddlewareTuple<ReturnValue<T>>) -> Self
    where
        T: Clone,
    {
        self.get_root.add_value_at_path("/*", middlewares.clone());
        self.options_root
            .add_value_at_path("/*", middlewares.clone());
        self.post_root.add_value_at_path("/*", middlewares.clone());
        self.put_root.add_value_at_path("/*", middlewares.clone());
        self.delete_root
            .add_value_at_path("/*", middlewares.clone());
        self.patch_root.add_value_at_path("/*", middlewares);

        self
    }

    /// Sets whether this app router uses strict mode for route parsing or not. Strict mode considers
    /// `/a` to be distinct from `/a/`.
    pub fn set_strict_mode(mut self, strict_mode: bool) -> Self
    where
        T: Clone,
    {
        self.get_root.strict_mode = strict_mode;
        self.options_root.strict_mode = strict_mode;
        self.post_root.strict_mode = strict_mode;
        self.put_root.strict_mode = strict_mode;
        self.delete_root.strict_mode = strict_mode;
        self.patch_root.strict_mode = strict_mode;

        self
    }

    /// Commits and locks in the route tree for usage.
    pub fn commit(mut self) -> Self {
        self.get_root = self.get_root.commit();
        self.options_root = self.options_root.commit();
        self.post_root = self.post_root.commit();
        self.put_root = self.put_root.commit();
        self.delete_root = self.delete_root.commit();
        self.patch_root = self.patch_root.commit();

        self
    }

    pub fn resolve_from_method_and_path<'m>(
        &'m self,
        method: &str,
        path: String,
    ) -> NodeOutput<'m, ReturnValue<T>> {
        match method {
            "OPTIONS" => self.options_root.get_value_at_path(path),
            "POST" => self.post_root.get_value_at_path(path),
            "PUT" => self.put_root.get_value_at_path(path),
            "DELETE" => self.delete_root.get_value_at_path(path),
            "PATCH" => self.patch_root.get_value_at_path(path),
            // default get
            _ => self.get_root.get_value_at_path(path),
        }
    }

    // pub async fn match_and_resolve<'m>(&'m self, request: R) -> Result<T::Response, io::Error> {
    pub fn match_and_resolve<'m>(
        &'m self,
        request: R,
    ) -> ReusableBoxFuture<Result<T::Response, io::Error>>
    where
        R: RequestWithParams,
    {
        let method = request.method();
        let path = request.path();

        let node = match method {
            "OPTIONS" => self.options_root.get_value_at_path(path),
            "POST" => self.post_root.get_value_at_path(path),
            "PUT" => self.put_root.get_value_at_path(path),
            "DELETE" => self.delete_root.get_value_at_path(path),
            "PATCH" => self.patch_root.get_value_at_path(path),
            // default get
            _ => self.get_root.get_value_at_path(path),
        };

        let context = (self.context_generator)(request, &self.state, &node.path);

        ReusableBoxFuture::new((node.value)(context).map(|ctx| {
            let ctx = match ctx {
                Ok(val) => val,
                Err(e) => e.context,
            };

            Ok(ctx.get_response())
        }))

        // let copy = node.value;
        // let ctx = (copy)(context).await;

        // let ctx = match ctx {
        //     Ok(val) => val,
        //     Err(e) => e.context,
        // };

        // Ok(ctx.get_response())

        // let copy = node.value;
        // let ctx = (copy)(context).await;

        // let ctx = match ctx {
        //     Ok(val) => val,
        //     Err(e) => e.context,
        // };

        // Ok(ctx.get_response())
    }

    pub async fn resolve<'m>(
        &self,
        request: R,
        matched_route: NodeOutput<'m, T>,
    ) -> Result<T::Response, io::Error> {
        self._resolve(request, matched_route).await
    }

    async fn _resolve<'m>(
        &self,
        request: R,
        matched_route: NodeOutput<'m, T>,
    ) -> Result<T::Response, io::Error> {
        let context = (self.context_generator)(request, &self.state, &matched_route.path);

        let copy = matched_route.value;
        let ctx = (copy)(context).await;

        let ctx = match ctx {
            Ok(val) => val,
            Err(e) => e.context,
        };

        Ok(ctx.get_response())
    }
}
