use std::collections::HashMap;
use std::vec::Vec;
use regex::Regex;

use route_search_tree::{RouteNode, RootNode};
use processed_route::{process_route};
use middleware::Middleware;

lazy_static! {
  static ref PARAM_REGEX: Regex = Regex::new(r"^:(\w+)$").unwrap();
}

struct ParsedRoute {
  pub params: HashMap<String, String>,
  pub search: HashMap<String, String>
}

pub struct MatchedRoute {
  pub value: String,
  pub params: HashMap<String, String>,
  pub middleware: Vec<Middleware>
}

impl MatchedRoute {
  fn new(value: String) -> MatchedRoute {
    MatchedRoute {
      value: value,
      params: HashMap::new(),
      middleware: Vec::new()
    }
  }

  fn from_matched_route(value: String, old_matched_route: MatchedRoute) -> MatchedRoute {
    let mut matched_route = MatchedRoute::new(value);

    matched_route.params = old_matched_route.params.clone();

    matched_route
  }
}

pub struct RouteParser {
  pub _route_root_node: RouteNode,
  pub middleware: HashMap<String, Vec<Middleware>>
}

impl RouteParser {
  pub fn new() -> RouteParser {
    let mut parser = RouteParser {
      _route_root_node: RouteNode::new(),
      middleware: HashMap::new()
    };

    parser
  }

  pub fn add_route(&mut self, route: String, middleware: Vec<Middleware>) -> &RouteNode {
    self.middleware.insert(route.clone(), middleware);
    self._route_root_node.add_route(route)
  }

  pub fn match_route(&self, route: String) -> MatchedRoute {
    let mut matched = self._match_route(route.clone(), &self._route_root_node, &MatchedRoute::new("".to_owned()))
      .expect(&format!("Could not match route {}", route));

    match self.middleware.get(&route) {
      Some(middleware) => matched.middleware = middleware.clone(),
      None => ()
    };

    matched
  }

  fn _match_route<'a>(&self, route: String, node: &'a RouteNode, match_in_progress: &MatchedRoute) -> Option<MatchedRoute> {
    let mut parsed_route = ParsedRoute {
      params: HashMap::new(),
      search: HashMap::new()
    };

    let processed_route = process_route(route);
    match processed_route {
      Some(mut route) => {
        let mut result: Option<MatchedRoute> = None;
        for val in node.children.values() {
          if val.value == route.head ||
            PARAM_REGEX.is_match(&val.value) {

            let recursive = match route.tail.take() {
              Some(tail) => {
                match self._match_route(tail, val, &match_in_progress) {
                  Some(child_match) => Some(MatchedRoute::from_matched_route(format!("{}/{}", val.value.clone(), child_match.value.clone()), child_match)),
                  None => None
                }
              },
              None => self._check_for_terminal_node(val)
            };

            match recursive {
              Some(_result) => result = Some(_result),
              None => ()
            };

            result = match result {
              Some(mut result) => {
                for cap in PARAM_REGEX.captures_iter(&val.value) {
                  result.params.insert((&cap[1]).to_owned(), route.head.clone());
                }

                Some(result)
              },
              None => None
            };
          }
        }

        result
      },
      None => self._check_for_terminal_node(node)
    }
  }

  fn _check_for_terminal_node(&self, node: &RouteNode) -> Option<MatchedRoute> {
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
  use context::Context;
  use fanta_error::FantaError;
  use middleware::Middleware;
  use futures::future;
  use futures::future::FutureResult;

  #[test]
  fn it_should_should_be_able_to_generate_a_simple_parsed_route() {
    let mut route_parser = RouteParser::new();
    route_parser.add_route("1/2/3".to_owned(), Vec::new());

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
    let mut route_parser = RouteParser::new();
    route_parser.add_route("1/2/3/4".to_owned(), Vec::new());

    assert!(route_parser.match_route("1/2/3/4".to_owned()).value == "1/2/3/4");
  }

  #[test]
  fn it_should_return_a_matched_path_for_a_good_route_with_multiple_similar_routes() {
    let mut route_parser = RouteParser::new();
    route_parser.add_route("1/2/3/6".to_owned(), Vec::new());
    route_parser.add_route("1/2/3/4/5".to_owned(), Vec::new());
    route_parser.add_route("1/2/3/4".to_owned(), Vec::new());

    assert!(route_parser.match_route("1/2/3/4".to_owned()).value == "1/2/3/4");
  }

  #[test]
  #[should_panic]
  fn it_should_panic_for_a_bad_route() {
    let mut route_parser = RouteParser::new();
    route_parser.add_route("1/2/3/4".to_owned(), Vec::new());
    route_parser.match_route("1/2/3".to_owned());
  }

  #[test]
  fn it_should_appropriately_define_route_params() {
    let mut route_parser = RouteParser::new();
    route_parser.add_route("1/:param/2".to_owned(), Vec::new());

    let matched = route_parser.match_route("1/somevar/2".to_owned());
    assert!(matched.value == "1/:param/2");
    assert!(matched.params.get("param").unwrap() == "somevar");
  }

  #[test]
  fn when_adding_a_route_it_should_return_a_struct_with_all_appropriate_middleware() {
    fn test_function(context: Context) -> Context {
      context
    }

    let mut route_parser = RouteParser::new();
    route_parser.add_route("1/2/3".to_owned(), vec![test_function]);

    let matched = route_parser.match_route("1/2/3".to_owned());
    assert!(matched.middleware.len() == 1);
    assert!(matched.middleware.get(0).unwrap() == &(test_function as Middleware));
  }
}
