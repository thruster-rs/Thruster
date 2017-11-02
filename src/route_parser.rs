use std::collections::HashMap;
use std::vec::Vec;

use route_search_tree::{RouteNode, RootNode};
use processed_route::{process_route};
use context::Context;

struct ParsedRoute {
  pub params: HashMap<String, String>,
  pub search: HashMap<String, String>
}

pub struct MatchedRoute {
  pub value: String,
  pub middleware: Vec<fn(Context, fn() -> ()) -> ()>
}

impl MatchedRoute {
  fn new(value: String) -> MatchedRoute {
    MatchedRoute {
      value: value,
      middleware: Vec::new()
    }
  }
}

pub struct RouteParser {
  pub _route_root_node: RouteNode
}

impl RouteParser {
  fn new() -> RouteParser {
    let mut parser = RouteParser {
      _route_root_node: RouteNode::new()
    };

    parser
  }

  fn add_route(&mut self, route: String) -> &RouteNode {
    self._route_root_node.add_route(route)
  }

  fn match_route(&self, route: String) -> MatchedRoute {
    self._match_route(route, &self._route_root_node, MatchedRoute::new("".to_owned()))
  }

  fn _match_route<'a>(&self, route: String, node: &'a RouteNode, match_in_progress: MatchedRoute) -> MatchedRoute {
    let mut parsed_route = ParsedRoute {
      params: HashMap::new(),
      search: HashMap::new()
    };

    let processed_route = process_route(route);
    match processed_route {
      Some(route) => {
        for val in node.children.values() {
          if val.value == route.head {
            return match route.tail {
              Some(tail) => MatchedRoute::new(format!("{}/{}", val.value.clone(), self._match_route(tail, val, match_in_progress).value.clone())),
              None => self._check_for_terminal_node(val)
            };
          }

          /**
           * Must also handle variable matches here, i.e. :some_var. We should attach
           * it to a context or something.
           */
        }

        panic!("No matching route found");
      },
      None => self._check_for_terminal_node(node)
    }
  }

  fn _check_for_terminal_node(&self, node: &RouteNode) -> MatchedRoute {
    if node.is_terminal {
      MatchedRoute::new(node.value.clone())
    } else {
      panic!("No matching route found");
    }
  }
}

#[cfg(test)]
mod tests {
  use super::RouteParser;

  #[test]
  fn it_should_should_be_able_to_generate_a_simple_parsed_route() {
    let mut route_parser = RouteParser::new();
    route_parser.add_route("1/2/3".to_owned());

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
    route_parser.add_route("1/2/3/4".to_owned());

    println!("Matched route {}", route_parser.match_route("1/2/3/4".to_owned()).value);
    assert!(route_parser.match_route("1/2/3/4".to_owned()).value == "1/2/3/4");
  }

  #[test]
  #[should_panic]
  fn it_should_panic_for_a_bad_route() {
    let mut route_parser = RouteParser::new();
    route_parser.add_route("1/2/3/4".to_owned());
    route_parser.match_route("1/2/3".to_owned());
  }

  #[test]
  fn when_adding_a_route_it_should_return_a_struct_with_all_appropriate_middleware() {

  }
}
