use std::collections::HashMap;
use std::vec::Vec;

use route_search_tree::{RouteNode, RootNode};
use processed_route::{process_route};
use util::{strip_leading_slash, wildcardify_params};
use middleware::{Middleware};
use context::Context;
use app::App;

pub struct MatchedRoute<T: 'static + Context> {
  pub value: String,
  pub params: HashMap<String, String>,
  pub middleware: Vec<Middleware<T>>,
  pub sub_app: Option<App<T>>
}

impl<T: Context> MatchedRoute<T> {
  fn new(value: String) -> MatchedRoute<T> {
    MatchedRoute {
      value: value,
      params: HashMap::new(),
      middleware: Vec::new(),
      sub_app: None
    }
  }

  fn new_sub_app(value: String, sub_app: App<T>) -> MatchedRoute<T> {
    MatchedRoute {
      value: value,
      params: HashMap::new(),
      middleware: Vec::new(),
      sub_app: Some(sub_app)
    }
  }

  fn from_matched_route(value: String, old_matched_route: MatchedRoute<T>) -> MatchedRoute<T> {
    match old_matched_route.sub_app {
      Some(old_app) => {
        let mut matched_route = MatchedRoute::new_sub_app(value, old_app);

        matched_route.params = old_matched_route.params.clone();

        matched_route
      },
      None => {
        let mut matched_route = MatchedRoute::new(value);

        matched_route.params = old_matched_route.params.clone();

        matched_route
      }
    }
  }
}

pub struct RouteParser<T: 'static + Context> {
  pub _method_agnostic_middleware: HashMap<String, Vec<Middleware<T>>>,
  pub _wildcarded_method_agnostic_middleware: HashMap<String, Vec<Middleware<T>>>,
  pub _route_root_node: RouteNode,
  pub _not_found_route: Vec<Middleware<T>>,
  pub middleware: HashMap<String, Vec<Middleware<T>>>,
  pub wildcarded_middleware: HashMap<String, Vec<Middleware<T>>>,
  pub sub_apps: HashMap<String, App<T>>,
}

impl<T: Context> RouteParser<T> {
  pub fn new() -> RouteParser<T> {
    let parser = RouteParser {
      _method_agnostic_middleware: HashMap::new(),
      _wildcarded_method_agnostic_middleware: HashMap::new(),
      _route_root_node: RouteNode::new(),
      _not_found_route: vec![],
      middleware: HashMap::new(),
      wildcarded_middleware: HashMap::new(),
      sub_apps: HashMap::new(),
    };

    parser
  }

  pub fn add_method_agnostic_middleware(&mut self, route: &str, middleware: Middleware<T>) {
    let _route = strip_leading_slash(route).clone();

    let updated_vector: Vec<Middleware<T>> = match self._method_agnostic_middleware.get(_route) {
      Some(val) => {
        let mut _in_progress: Vec<Middleware<T>> = Vec::new();
        for func in val {
          _in_progress.push(func.clone());
        }

        _in_progress.push(middleware);
        _in_progress
      },
      None => vec![middleware]
    };

    self._method_agnostic_middleware.insert(_route.to_owned(), updated_vector.clone());
    self._wildcarded_method_agnostic_middleware.insert(wildcardify_params(&_route), updated_vector);
  }

  pub fn set_not_found(&mut self, middleware: Vec<Middleware<T>>) {
    self._not_found_route = middleware;
  }

  pub fn add_route(&mut self, route: &str, middleware: Vec<Middleware<T>>) -> &RouteNode {
    let _route = strip_leading_slash(route);

    self.middleware.insert(_route.to_owned(), middleware.clone());
    self.wildcarded_middleware.insert(wildcardify_params(&_route), middleware);
    self._route_root_node.add_route(_route.to_owned())
  }

  pub fn match_route(&self, route: &str) -> MatchedRoute<T> {
    let route = strip_leading_slash(route);
    let route_iterator = route.split("/");
    let mut accumulator = "".to_owned();
    let mut current_node = &self._route_root_node;
    let mut middleware = Vec::new();
    let mut has_terminal_node = false;

    for piece in route_iterator {
      for mut val in current_node.children.values() {
        if val.value == piece {
          if val.is_terminal {
            accumulator = templatify! { ""; &accumulator ;"/"; piece ;"" };
            let wildcarded_params = wildcardify_params(&accumulator);
            let wildcarded_route = strip_leading_slash(&wildcarded_params);
            match self.wildcarded_middleware.get(wildcarded_route) {
              Some(_middleware) => {
                for func in _middleware {
                  middleware.push(*func);
                }
                has_terminal_node = true;
                break;
              },
              None => ()
            };
          } else {
            current_node = val;
            break;
          }
        }
      }

      accumulator = templatify! { ""; &accumulator ;"/"; piece ;"" };
    }

    if !has_terminal_node {
      for func in &self._not_found_route {
        middleware.push(*func);
      }
    }

    MatchedRoute {
      value: route.to_owned(),
      params: HashMap::new(),
      middleware: middleware,
      sub_app: None
    }
  }

  pub fn match_route_legacy(&self, route: &str, ) -> MatchedRoute<T> {
    let _route = strip_leading_slash(route);

    /**
     * for piece of path
     *   add path to accumulator
     *   for child of current_node
     *     copy the method agnostic middleware based on accumulator
     *     if child == head
     *       copy the middleware of the node
     *       if child is terminal node
     *         break
     * if not child, add 404
     * return middleware
     */

    let mut matched = match self._match_route(&_route, &self._route_root_node, &MatchedRoute::new("".to_owned())) {
      Some(_matched) => _matched,
      None => {
        let mut not_found = MatchedRoute::new(_route.to_owned());
        not_found.middleware = self._not_found_route.clone();
        not_found
      }
    };

    let wildcarded_route = wildcardify_params(&matched.value);

    match self.wildcarded_middleware.get(&wildcarded_route) {
      Some(middleware) => {
        let mut accumulating_middleware = middleware.clone();

        let mut accumulator = Vec::new();

        let mut accumulator_index = 0;
        let mut part_iterator = _route.split("/");
        // Drop the method
        part_iterator.next();
        for part in part_iterator {

          match self._method_agnostic_middleware.get(&accumulator.join("/")) {
            Some(val) => {
              for func in val {
                accumulating_middleware.insert(accumulator_index, func.clone());
                accumulator_index = accumulator_index + 1;
              }
            },
            None => ()
          };

          accumulator.push(part);
        }

        matched.middleware = accumulating_middleware;
      },
      None => ()
    };

    matched
  }

  fn _match_route<'a>(&self, route: &str, node: &'a RouteNode, match_in_progress: &MatchedRoute<T>) -> Option<MatchedRoute<T>> {
    let processed_route = process_route(&route);
    match processed_route {
      Some(mut route) => {
        let mut result: Option<MatchedRoute<T>> = None;
        for val in node.children.values() {
          if val.value == route.head ||
            val.has_params {

            let recursive = match route.tail.take() {
              Some(tail) => {
                match self._match_route(&tail, val, &match_in_progress) {
                  Some(child_match) => Some(MatchedRoute::from_matched_route(format!("{}/{}", val.value.clone(), child_match.value.clone()), child_match)),
                  None => None
                }
              },
              None => self._check_for_terminal_node(val)
            };

            result = match recursive {
              Some(mut _result) => {
                if val.has_params {
                  _result.params.insert(val.value[1..].to_owned(), route.head.clone());
                }

                Some(_result)
              },
              None => None
            };
            break;
          }
        }

        result
      },
      None => self._check_for_terminal_node(node)
    }
  }

  fn _check_for_terminal_node(&self, node: &RouteNode) -> Option<MatchedRoute<T>> {
    if node.is_terminal {
      Some(MatchedRoute::new(node.value.clone()))
    } else {
      None
    }
  }
}

#[cfg(test)]
mod tests {
  use super::RouteParser;
  use context::BasicContext;
  use middleware::MiddlewareChain;
  use futures::{future, Future};
  use std::io;
  use std::boxed::Box;

  #[test]
  fn it_should_should_be_able_to_generate_a_simple_parsed_route() {
    let mut route_parser = RouteParser::<BasicContext>::new();
    route_parser.add_route("1/2/3", Vec::new());

    let route_node = route_parser._route_root_node.children.get(&"1".to_owned()).unwrap();
    let second_child_node = route_node.children.get(&"2".to_owned()).unwrap();
    let third_child_node = second_child_node.children.get(&"3".to_owned()).unwrap();

    assert!(route_node.value == "1".to_owned());
    assert!(route_node.children.len() == 1);

    assert!(second_child_node.value == "2".to_owned());
    assert!(second_child_node.children.len() == 1);

    assert!(third_child_node.value == "3".to_owned());
    assert!(third_child_node.children.len() == 0);
  }

  #[test]
  fn it_should_return_a_matched_path_for_a_good_route() {
    let mut route_parser = RouteParser::<BasicContext>::new();
    route_parser.add_route("1/2/3/4", Vec::new());

    assert!(route_parser.match_route("1/2/3/4").value == "1/2/3/4");
  }

  #[test]
  fn it_should_use_not_found_for_not_handled_routes() {
    let mut route_parser = RouteParser::<BasicContext>::new();
    route_parser.add_route("1/2/3/4", Vec::new());
    route_parser.set_not_found(Vec::new());

    assert!(route_parser.match_route("5").value == "5");
  }

  #[test]
  fn it_should_return_a_matched_path_for_a_good_route_with_multiple_similar_routes() {
    let mut route_parser = RouteParser::<BasicContext>::new();
    route_parser.add_route("1/2/3/6", Vec::new());
    route_parser.add_route("1/2/3/4/5", Vec::new());
    route_parser.add_route("1/2/3/4", Vec::new());

    assert!(route_parser.match_route("1/2/3/4").value == "1/2/3/4");
  }

  #[test]
  fn it_should_appropriately_define_route_params() {
    let mut route_parser = RouteParser::<BasicContext>::new();
    route_parser.add_route("1/:param/2", Vec::new());

    let matched = route_parser.match_route("1/somevar/2");
    assert!(matched.value == "1/:param/2");
    assert!(matched.params.get("param").unwrap() == "somevar");
  }

  #[test]
  fn when_adding_a_route_it_should_return_a_struct_with_all_appropriate_middleware() {
    fn test_function(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> Box<Future<Item=BasicContext, Error=io::Error>> {
      Box::new(future::ok(context))
    }

    let mut route_parser = RouteParser::<BasicContext>::new();
    route_parser.add_route("1/2/3", vec![test_function]);

    let matched = route_parser.match_route("1/2/3");
    assert!(matched.middleware.len() == 1);
    // assert!(matched.middleware.get(0).unwrap() == &(test_function as Middleware));
  }

  #[test]
  fn when_adding_a_route_with_method_agnostic_middleware() {
    fn method_agnostic(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> Box<Future<Item=BasicContext, Error=io::Error>> {
      Box::new(future::ok(context))
    }

    fn test_function(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> Box<Future<Item=BasicContext, Error=io::Error>> {
      Box::new(future::ok(context))
    }

    let mut route_parser = RouteParser::<BasicContext>::new();
    route_parser.add_method_agnostic_middleware("/", method_agnostic);
    route_parser.add_route("1/2/3", vec![test_function]);

    let matched = route_parser.match_route("1/2/3");
    assert!(matched.middleware.len() == 2);
    // assert!(matched.middleware.get(0).unwrap() == method_agnostic);
    // assert!(matched.middleware.get(1).unwrap() == &(test_function as Middleware));
  }
}
