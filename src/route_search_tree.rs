use std::collections::HashMap;
use std::vec::Vec;

use processed_route::{process_route};
use context::Context;

pub struct RouteNode {
  pub value: String,
  pub children: HashMap<String, RouteNode>,
  pub is_terminal: bool,
  pub middleware: Vec<fn(Context, fn() -> ()) -> ()>
}

trait FromStatic {
  fn new(value: &'static str) -> RouteNode;
}

impl FromStatic for RouteNode {
  fn new(value: &'static str) -> RouteNode {
    RouteNode::new(value.to_owned())
  }
}

pub trait RootNode {
  fn new() -> RouteNode;
}

impl RootNode for RouteNode {
  fn new() -> RouteNode {
    RouteNode::new("__root__".to_owned())
  }
}

impl RouteNode {
  fn new(value: String) -> RouteNode {
    println!("Making a route node for {}", value);

    let processed_route = process_route(value)
      .expect("Incorrect route passed to RouteNode -- No head");

    let mut route_node = RouteNode {
      value: processed_route.head,
      children: HashMap::new(),
      is_terminal: false,
      middleware: Vec::new()
    };

    match processed_route.tail {
      Some(tail) => { route_node.add_route(tail); },
      None => { (); }
    };

    route_node
  }

  pub fn add_route(&mut self, route: String) -> &mut RouteNode {
    if route.len() == 0 {
      self.is_terminal = true;
      return self
    }

    let mut split = route.split("/");

    match split.next() {
      Some(value) => {
        // See if the node already exists, if it doesn't then insert it.
        let mut joined = (match split.next() {
          Some(value) => value,
          None => ""
        }).to_owned();

        for part in split {
          joined = format!("{}/{}", joined, part);
        }

        let mut added_child = None;
        match self.children.get_mut(value) {
          Some(existing_node) => {
            existing_node.add_route(joined);
          },
          None => {
            let mut new_node = RouteNode::new(value.to_owned());
            new_node.add_route(joined);
            added_child = Some(new_node);
          }
        }

        match added_child {
          Some(child) => self.children.insert(value.to_owned(), child),
          None => None
        };
      },
      None => (),
    };

    self
  }
}

#[cfg(test)]
mod tests {
  use super::RouteNode;

  #[test]
  fn it_adds_chilren_based_off_of_iterator() {
    let mut route_node = RouteNode::new("1".to_owned());

    route_node.add_route("2/3/4".to_owned());

    assert!(route_node.children.len() == 1);
  }

  #[test]
  fn it_correctly_breaks_down_a_route_into_a_tree() {
    let route_node = RouteNode::new("1/2/3".to_owned());
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
  fn it_correctly_adds_nodes_based_on_a_route() {
    let mut route_node = RouteNode::new("1/2/3".to_owned());
    route_node.add_route("2/4".to_owned());

    assert!(route_node.value == "1".to_owned());
    assert!(route_node.children.len() == 1);

    let second_child_node = route_node.children.get(&"2".to_owned()).unwrap();
    assert!(second_child_node.value == "2".to_owned());
    assert!(second_child_node.children.len() == 2);

    let third_child_node = second_child_node.children.get(&"3".to_owned()).unwrap();
    assert!(third_child_node.value == "3".to_owned());
    assert!(third_child_node.children.len() == 0);

    let fourth_child_node = second_child_node.children.get(&"4".to_owned()).unwrap();
    assert!(fourth_child_node.value == "4".to_owned());
    assert!(fourth_child_node.children.len() == 0);
  }

  #[test]
  fn it_should_allow_paramaterized_routes() {
    let route_node = RouteNode::new("1/:param".to_owned());

    assert!(route_node.children.len() == 1);
  }
}
