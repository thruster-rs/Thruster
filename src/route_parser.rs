use std::collections::HashMap;

#[cfg(not(feature = "async_await"))]
use crate::middleware::{MiddlewareChain};
#[cfg(feature = "async_await")]
use crate::async_middleware::{MiddlewareChain};
use crate::route_tree::RouteTree;
use crate::context::Context;
use crate::app::App;
use crate::request::RequestWithParams;

pub struct MatchedRoute<'a, R: RequestWithParams, T: 'static + Context + Send> {
  pub middleware: &'a MiddlewareChain<T>,
  pub value: String,
  pub params: HashMap<String, String>,
  pub query_params: HashMap<String, String>,
  pub sub_app: Option<App<R, T>>
}

pub struct RouteParser<T: 'static + Context + Send> {
  pub route_tree: RouteTree<T>,
  pub shortcuts: HashMap<String, MiddlewareChain<T>>
}

impl<T: Context + Send> RouteParser<T> {
  pub fn new() -> RouteParser<T> {
    RouteParser {
      route_tree: RouteTree::new(),
      shortcuts: HashMap::new()
    }
  }

  pub fn add_method_agnostic_middleware(&mut self, route: &str, middleware: MiddlewareChain<T>) {
    self.route_tree.add_use_node(route, middleware);
  }

  pub fn add_route(&mut self, route: &str, middleware: MiddlewareChain<T>) {
    self.route_tree.add_route(route, middleware);
  }

  pub fn optimize(&mut self) {
    let routes = self.route_tree.root_node.enumerate();

    for (path, middleware) in routes {
      self.shortcuts.insert((&path[1..]).to_owned(), middleware);
    }
  }

  #[inline]
  pub fn match_route<R: RequestWithParams>(&self, route: &str) -> MatchedRoute<R, T> {
    let query_params = HashMap::new();

    let split_route = route.find('?');
    let mut route = match split_route {
      Some(index) => &route[0..index],
      None => route
    };

    // Trim trailing slashes
    while &route[route.len() - 1..route.len()] == "/" {
      route = &route[0..route.len() - 1];
    }

    if let Some(shortcut) = self.shortcuts.get(route) {
      MatchedRoute {
        middleware: shortcut,
        value: route.to_owned(),
        params: HashMap::new(),
        query_params,
        sub_app: None
      }
    } else {
      let matched = self.route_tree.match_route(route);

      MatchedRoute {
        middleware: matched.1,
        value: route.to_owned(),
        params: matched.0,
        query_params,
        sub_app: None
      }
    }
  }
}

impl<T: Context + Send> Default for RouteParser<T> {
  fn default() -> RouteParser<T> {
    RouteParser::new()
  }
}

#[cfg(test)]
mod tests {
  use super::RouteParser;
  use crate::builtins::basic_context::BasicContext;
  use crate::request::Request;
  #[cfg(not(feature = "async_await"))]
use crate::middleware::{MiddlewareChain, MiddlewareReturnValue};
#[cfg(feature = "async_await")]
use crate::async_middleware::{MiddlewareChain, MiddlewareReturnValue};
  use futures_legacy::future;
  use std::boxed::Box;

  fn fake(_: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> MiddlewareReturnValue<BasicContext> {
    panic!("not implemented");
  }

  #[test]
  fn it_should_return_a_matched_path_for_a_good_route() {
    let mut route_parser = RouteParser::<BasicContext>::new();
    route_parser.add_route("1/2/3/4", middleware![BasicContext => fake]);

    assert!(route_parser.match_route::<Request>("1/2/3/4").value == "1/2/3/4");
  }

  #[test]
  fn it_should_use_not_found_for_not_handled_routes() {
    let mut route_parser = RouteParser::<BasicContext>::new();
    route_parser.add_route("1/2/3/4", middleware![BasicContext => fake]);

    assert!(route_parser.match_route::<Request>("5").value == "5");
  }

  #[test]
  fn it_should_return_a_matched_path_for_a_good_route_with_multiple_similar_routes() {
    let mut route_parser = RouteParser::<BasicContext>::new();
    route_parser.add_route("1/2/3/6", middleware![BasicContext => fake]);
    route_parser.add_route("1/2/3/4/5", middleware![BasicContext => fake]);
    route_parser.add_route("1/2/3/4", middleware![BasicContext => fake]);

    assert!(route_parser.match_route::<Request>("1/2/3/4").value == "1/2/3/4");
  }

  #[test]
  fn it_should_appropriately_define_route_params() {
    let mut route_parser = RouteParser::<BasicContext>::new();
    route_parser.add_route("1/:param/2", middleware![BasicContext => fake]);

    let matched = route_parser.match_route::<Request>("1/somevar/2");

    assert!(matched.params.get("param").unwrap() == "somevar");
  }

  #[test]
  fn when_adding_a_route_it_should_return_a_struct_with_all_appropriate_middleware() {
    let mut route_parser = RouteParser::<BasicContext>::new();
    route_parser.add_route("1/2/3", middleware![BasicContext => fake]);

    let matched = route_parser.match_route::<Request>("1/2/3");
    assert!(matched.middleware.is_assigned());
  }

  #[test]
  fn when_adding_a_route_with_method_agnostic_middleware() {
    fn method_agnostic(context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> MiddlewareReturnValue<BasicContext> {
      Box::new(future::ok(context))
    }

    fn test_function(context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> MiddlewareReturnValue<BasicContext> {
      Box::new(future::ok(context))
    }

    let mut route_parser = RouteParser::<BasicContext>::new();
    route_parser.add_method_agnostic_middleware("/", middleware![BasicContext => method_agnostic]);
    route_parser.add_route("/__GET__/1/2/3", middleware![BasicContext => test_function]);

    let _matched = route_parser.match_route::<Request>("__GET__/1/2/3");
    // assert!(matched.middleware.len() == 2);
  }
}
