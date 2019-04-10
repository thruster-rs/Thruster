use std::collections::HashMap;
use smallvec::SmallVec;
use std::str::Split;


use crate::context::Context;
#[cfg(not(feature = "thruster_async_await"))]
use crate::middleware::{MiddlewareChain};
#[cfg(feature = "thruster_async_await")]
use thruster_core_async_await::{MiddlewareChain};

// A route with params that may or may not be a terminal node.
type RouteNodeWithParams<'a, T> = (HashMap<String, String>, bool, &'a MiddlewareChain<T>);
type RouteNodeEnumeration<T> = SmallVec<[(String, MiddlewareChain<T>); 8]>;

pub struct Node<T: 'static + Context + Send> {
  runnable: MiddlewareChain<T>,
  pub children: HashMap<String, Node<T>>,
  wildcard_node: Option<Box<Node<T>>>,
  param_key: Option<String>,
  pub value: String,
  pub is_wildcard: bool,
  pub is_terminal_node: bool
}

impl<T: Context + Send> Clone for Node<T> {
  fn clone(&self) -> Self {
    let runnable = self.runnable.clone();
    let children = self.children.clone();

    let wildcard = if self.is_wildcard {
      None
    } else {
      Some(Box::new(match &self.wildcard_node {
        Some(wildcard) => (**wildcard).clone(),
        None => Node::new_wildcard(None)
      }))
    };

    Node {
      runnable,
      children,
      wildcard_node: wildcard,
      value: self.value.clone(),
      param_key: self.param_key.clone(),
      is_wildcard: self.is_wildcard,
      is_terminal_node: self.is_terminal_node
    }
  }
}

impl<T: 'static + Context + Send> Node<T> {

  pub fn new(value: &str) -> Node<T> {
    Node {
      runnable: MiddlewareChain::new(),
      children: HashMap::new(),
      wildcard_node: Some(Box::new(Node::new_wildcard(None))),
      value: value.to_owned(),
      param_key: None,
      is_wildcard: false,
      is_terminal_node: false
    }
  }

  pub fn new_wildcard(param_name: Option<String>) -> Node<T> {
    Node {
      runnable: MiddlewareChain::new(),
      children: HashMap::new(),
      wildcard_node: None,
      value: "*".to_owned(),
      param_key: param_name,
      is_wildcard: true,
      is_terminal_node: false
    }
  }

  pub fn add_route(&mut self, route: &str, middleware: MiddlewareChain<T>) {
    // Strip a leading slash
    let mut split_iterator = match route.chars().next() {
      Some('/') => &route[1..],
      _ => route
    }.split('/');

    if let Some(piece) = split_iterator.next() {
      if piece.is_empty() {
        self.is_terminal_node = true;
        self.runnable = middleware;
      } else {
        match piece.chars().next().unwrap() {
          ':' => {
            if !self.is_wildcard {
              let mut wildcard = Node::new_wildcard(Some(piece[1..].to_owned()));
              wildcard.is_terminal_node = false;

              wildcard.add_route(&split_iterator.collect::<SmallVec<[&str; 8]>>().join("/"), middleware);

              self.wildcard_node = Some(Box::new(wildcard));
            }
          },
          '*' => {
            if !self.is_wildcard {
              let mut wildcard = Node::new_wildcard(None);
              wildcard.is_terminal_node = true;

              wildcard.add_route(&split_iterator.collect::<SmallVec<[&str; 8]>>().join("/"), middleware);

              self.wildcard_node = Some(Box::new(wildcard));
            }
          },
          _ => {
            let mut child = self.children.remove(piece)
              .unwrap_or_else(|| Node::new(piece));

            child.add_route(&split_iterator.collect::<SmallVec<[&str; 8]>>().join("/"), middleware);

            self.children.insert(piece.to_owned(), child);
          }
        };
      }
    }
  }

  pub fn add_subtree(&mut self, route: &str, mut subtree: Node<T>) {
    // Strip a leading slash
    let mut split_iterator = match route.chars().next() {
      Some('/') => &route[1..],
      _ => route
    }.split('/');

    if let Some(piece) = split_iterator.next() {
      if piece.is_empty() {
        if let Some(wildcard_node) = &subtree.wildcard_node {
          if wildcard_node.param_key.is_some() {
            if let Some(ref mut existing_wildcard) = self.wildcard_node {
              for (key, child) in subtree.children.drain() {
                existing_wildcard.children.insert(key, child);
              }

              existing_wildcard.param_key = wildcard_node.param_key.clone();
              existing_wildcard.is_terminal_node = existing_wildcard.is_terminal_node || wildcard_node.is_terminal_node;
            }
          } else {
            for (key, child) in subtree.children.drain() {
              self.children.insert(key, child);
            }
          }
        }

        self.wildcard_node = subtree.wildcard_node;
        self.is_terminal_node = subtree.is_terminal_node;
      } else {
        let mut child = self.children.remove(piece)
          .unwrap_or_else(|| Node::new(piece));

        if subtree.runnable.is_assigned() {
          subtree.runnable.chain(child.runnable.clone());
          child.runnable = subtree.runnable.clone();
        }

        child.add_subtree(&split_iterator.collect::<SmallVec<[&str; 8]>>().join("/"), subtree);

        self.children.insert(piece.to_owned(), child);
      }
    }
  }

  pub fn match_route(&self, route: Split<char>) -> RouteNodeWithParams<T> {
    self.match_route_with_params(route, HashMap::new())
  }

  pub fn match_route_with_params(&self, mut route: Split<char>, mut params: HashMap<String, String>) -> RouteNodeWithParams<T> {
    if let Some(piece) = route.next() {
      match self.children.get(piece) {
        Some(child) => {
            let results = child.match_route_with_params(route, params);

            if results.1 {
              results
            } else {
              match &self.wildcard_node {
                Some(wildcard_node) => (results.0, wildcard_node.is_terminal_node, &wildcard_node.runnable),
                None => {
                  if !self.is_wildcard {
                    results
                  } else {
                    (results.0, self.is_terminal_node, &self.runnable)
                  }
                }
              }
            // }
          }
        },
        None => {
          match &self.wildcard_node {
            Some(wildcard_node) => {
              // Here we check if the current length of the node is 0 or not, if it's
              // 0 and there is a param key, then this is actually a miss, so return
              // a non-terminal node (this is the case where the defined route is like
              // /a/:b, but the incoming route to match is /a/)
              if piece.is_empty() && wildcard_node.param_key.is_some() {
                (params, false, &wildcard_node.runnable)
              } else if piece.is_empty() && wildcard_node.param_key.is_none() {
                (params, wildcard_node.is_terminal_node, &wildcard_node.runnable)
              } else {
                if let Some(param_key) = &wildcard_node.param_key {
                  params.insert(param_key.to_owned(), piece.to_owned());
                }

                let results = wildcard_node.match_route_with_params(route, params);

                // If the wildcard isn't a terminal node, then try to return this
                // wildcard
                if results.1 {
                  results
                } else {
                  (results.0, wildcard_node.is_terminal_node, &self.runnable)
                }
              }
            }
            None => (params, self.is_terminal_node, &self.runnable)
          }
        }
      }
    } else if self.is_terminal_node {
        (params, self.is_terminal_node, &self.runnable)
    } else if let Some(wildcard_node) = &self.wildcard_node {
      if wildcard_node.param_key == None {
        let results = wildcard_node.match_route_with_params(route, params);

        // If the wildcard isn't a terminal node, then try to return this
        // wildcard
        if results.1 {
          results
        } else {
          (results.0, self.is_terminal_node, &self.runnable)
        }
      } else {
        (params, self.is_terminal_node, &self.runnable)
      }
    } else {
      (params, self.is_terminal_node, &self.runnable)
    }
  }

  /// Used mostly for debugging the route tree
  /// Example usage
  /// ```ignore
  ///   let mut app = App::create(generate_context);
  ///
  ///   app.get("/plaintext", middleware![plaintext]);
  ///  println!("app: {}", app._route_parser.route_tree.root_node.to_string(""));
  ///  for (route, middleware) in app._route_parser.route_tree.root_node.enumerate() {
  ///    println!("{}: {}", route, middleware.len());
  ///  }
  /// ```
  pub fn to_string(&self, indent: &str) -> String {
    let mut in_progress = "".to_owned();
    let value = match self.param_key.clone() {
      Some(key) => format!(":{}", key),
      None => self.value.to_owned()
    };

    in_progress = format!("{}\n{}{}: {}, {}",
      in_progress,
      indent,
      value,
      self.runnable.is_assigned(),
      self.is_terminal_node);

    for child in self.children.values() {
      in_progress = format!("{}{}", in_progress, child.to_string(&format!("{}  ", indent)));
    }

    if let Some(wildcard_node) = &self.wildcard_node {
      in_progress = format!("{}{}", in_progress, wildcard_node.to_string(&format!("{}  ", indent)));
    }

    in_progress
  }

  pub fn enumerate(&self) -> RouteNodeEnumeration<T> {
    let mut children = SmallVec::new();

    for child in self.children.values() {
      let piece = match &self.param_key {
        Some(param_key) => format!(":{}", param_key),
        None => self.value.clone()
      };

      let child_enumeration = child.enumerate();

      if !child_enumeration.is_empty() {
        for child_route in child_enumeration {
          children.push((format!("{}/{}", piece, child_route.0), child_route.1));
        }
      } else {
        children.push((format!("{}/{}", piece, child.value), child.runnable.clone()));
      }
    }

    children
  }

  ///
  /// Pushes middleware down to the leaf nodes, accumulating along the way. This is helpful for
  /// propagating generic middleware down the stack
  ///
  pub fn push_middleware_to_populated_nodes(&mut self, other_node: &Node<T>, accumulated_middleware: &MiddlewareChain<T>) {
    let fake_node = Node::new("");

    let accumulating_chain = if other_node.runnable.is_assigned() {
      let mut accumulating_chain = other_node.runnable.clone();
      accumulating_chain.chain(accumulated_middleware.clone());
      accumulating_chain
    } else {
      accumulated_middleware.clone()
    };

    if self.runnable.is_assigned() {
      let mut other = other_node.runnable.clone();
      let mut other_2 = other.clone();
      let mut accumulating_chain = accumulated_middleware.clone();
      let old = self.runnable.clone();

      other_2.chain(old);
      accumulating_chain.chain(other_2);
      other.chain(accumulating_chain);

      self.runnable = other;
    }

    // Match children, recurse if child match
    for (key, child) in &mut self.children {
      // Traverse the child tree, or else make a dummy node
      let other_child = other_node.children.get(key).unwrap_or(&fake_node);

      child.push_middleware_to_populated_nodes(other_child, &accumulating_chain.clone());
    }

    // Copy over wildcards
    if let Some(ref mut wildcard_node) = self.wildcard_node {
      if let Some(other_wildcard_node) = &other_node.wildcard_node {
        wildcard_node.push_middleware_to_populated_nodes(other_wildcard_node, &accumulating_chain.clone());
      }
    }
  }
}
