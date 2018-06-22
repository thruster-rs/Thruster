use std::collections::HashMap;
use smallvec::SmallVec;

use middleware::{Middleware};
use route_tree::RouteTree;
use context::Context;
use app::App;

pub struct MatchedRoute<'a, T: 'static + Context + Send> {
  pub value: String,
  pub params: HashMap<String, String>,
  pub query_params: HashMap<String, String>,
  pub middleware: &'a SmallVec<[Middleware<T>; 8]>,
  pub sub_app: Option<App<T>>
}

pub struct RouteParser<T: 'static + Context + Send> {
  pub route_tree: RouteTree<T>
}

impl<T: Context + Send> RouteParser<T> {
  pub fn new() -> RouteParser<T> {
    let parser = RouteParser {
      route_tree: RouteTree::new(),
    };

    parser
  }

  pub fn add_method_agnostic_middleware(&mut self, route: &str, middleware: Middleware<T>) {
    self.route_tree.add_use_node(route, smallvec![middleware]);
  }

  pub fn add_route(&mut self, route: &str, middleware: SmallVec<[Middleware<T>; 8]>) {
    self.route_tree.add_route(route, middleware);
  }

  pub fn match_route(&self, route: &str) -> MatchedRoute<T> {
    let mut query_params = HashMap::new();

    let mut iter = route.split("?");
    let route = iter.next().unwrap();
    match iter.next() {
      Some(query_string) => {
        for query_piece in query_string.split("&") {
          let mut query_iterator = query_piece.split("=");
          let key = query_iterator.next().unwrap().to_owned();
          match query_iterator.next() {
            Some(val) => query_params.insert(key, val.to_owned()),
            None => query_params.insert(key, "true".to_owned())
          };
        }
      },
      None => ()
    };

    let matched = self.route_tree.match_route(route);

    MatchedRoute {
      value: route.to_owned(),
      params: matched.1,
      query_params: query_params,
      middleware: matched.0,
      sub_app: None
    }
  }
}

#[cfg(test)]
mod tests {
  use super::RouteParser;
  use context::BasicContext;
  use middleware::{Middleware, MiddlewareChain, MiddlewareReturnValue};
  use futures::future;
  use std::boxed::Box;
  use smallvec::SmallVec;

  #[test]
  fn it_should_return_a_matched_path_for_a_good_route() {
    let mut route_parser = RouteParser::<BasicContext>::new();
    route_parser.add_route("1/2/3/4", SmallVec::new());

    assert!(route_parser.match_route("1/2/3/4").value == "1/2/3/4");
  }

  #[test]
  fn it_should_use_not_found_for_not_handled_routes() {
    let mut route_parser = RouteParser::<BasicContext>::new();
    route_parser.add_route("1/2/3/4", SmallVec::new());

    assert!(route_parser.match_route("5").value == "5");
  }

  #[test]
  fn it_should_return_a_matched_path_for_a_good_route_with_multiple_similar_routes() {
    let mut route_parser = RouteParser::<BasicContext>::new();
    route_parser.add_route("1/2/3/6", SmallVec::new());
    route_parser.add_route("1/2/3/4/5", SmallVec::new());
    route_parser.add_route("1/2/3/4", SmallVec::new());

    assert!(route_parser.match_route("1/2/3/4").value == "1/2/3/4");
  }

  #[test]
  fn it_should_appropriately_define_route_params() {
    let mut route_parser = RouteParser::<BasicContext>::new();
    route_parser.add_route("1/:param/2", SmallVec::new());

    let matched = route_parser.match_route("1/somevar/2");

    assert!(matched.params.get("param").unwrap() == "somevar");
  }

  #[test]
  fn when_adding_a_route_it_should_return_a_struct_with_all_appropriate_middleware() {
    fn test_function(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> MiddlewareReturnValue<BasicContext> {
      Box::new(future::ok(context))
    }

    let mut route_parser = RouteParser::<BasicContext>::new();
    route_parser.add_route("1/2/3", smallvec![test_function as Middleware<BasicContext>]);

    let matched = route_parser.match_route("1/2/3");
    assert!(matched.middleware.len() == 1);
    // assert!(matched.middleware.get(0).unwrap() == &(test_function as Middleware));
  }

  #[test]
  fn when_adding_a_route_with_method_agnostic_middleware() {
    fn method_agnostic(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> MiddlewareReturnValue<BasicContext> {
      Box::new(future::ok(context))
    }

    fn test_function(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> MiddlewareReturnValue<BasicContext> {
      Box::new(future::ok(context))
    }

    let mut route_parser = RouteParser::<BasicContext>::new();
    route_parser.add_method_agnostic_middleware("/", method_agnostic);
    route_parser.add_route("/__GET__/1/2/3", smallvec![test_function as Middleware<BasicContext>]);

    let matched = route_parser.match_route("__GET__/1/2/3");
    assert!(matched.middleware.len() == 2);
  }
}
