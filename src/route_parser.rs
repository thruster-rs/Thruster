use std::collections::HashMap;
use route_search_tree::{RouteNode, RootNode};

struct ParsedRoute {
  pub params: HashMap<String, String>,
  pub search: HashMap<String, String>
}

pub struct RouteParser {
  _route_root_node: RouteNode
}

impl RouteParser {
  fn new() -> RouteParser {
    let mut parser = RouteParser {
      _route_root_node: RouteNode::new()
    };

    parser
  }

  fn add_route(&mut self, route: String) -> &mut RouteParser {
    self._route_root_node.add_route(route);

    self
  }

  fn _match_route(route: String) {
    let mut parsed_route = ParsedRoute {
      params: HashMap::new(),
      search: HashMap::new()
    };

    let split = route.split("/");

    for part in split {

    }
  }
}

#[cfg(test)]
mod tests {
  use super::RouteParser;

  #[test]
  fn it_should_should_be_able_to_generate_a_simple_parsed_route() {
  }
}
