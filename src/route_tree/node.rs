use std::collections::HashMap;
use smallvec::SmallVec;
use std::str::Split;

use middleware::{Middleware};
use context::Context;

// A route with params that may or may not be a terminal node.
type RouteNodeWithParams<'a, T> = (&'a SmallVec<[Middleware<T>; 8]>, HashMap<String, String>, bool);
type RouteNodeEnumeration<T> = SmallVec<[(String, SmallVec<[Middleware<T>; 8]>); 8]>;

pub struct Node<T: Context + Send> {
  middleware: SmallVec<[Middleware<T>; 8]>,
  pub children: HashMap<String, Node<T>>,
  wildcard_node: Option<Box<Node<T>>>,
  param_key: Option<String>,
  pub value: String,
  pub is_wildcard: bool,
  pub is_terminal_node: bool
}

impl<T: Context + Send> Clone for Node<T> {
  fn clone(&self) -> Self {
    let children = self.children.clone();
    let middleware = self.middleware.clone();

    let wildcard = if self.is_wildcard {
      None
    } else {
      Some(Box::new(match &self.wildcard_node {
        Some(wildcard) => (**wildcard).clone(),
        None => Node::new_wildcard(None)
      }))
    };

    Node {
      children,
      middleware,
      wildcard_node: wildcard,
      value: self.value.clone(),
      param_key: self.param_key.clone(),
      is_wildcard: self.is_wildcard,
      is_terminal_node: self.is_terminal_node
    }
  }
}

impl<T: Context + Send> Node<T> {

  pub fn new(value: &str) -> Node<T> {
    Node {
      children: HashMap::new(),
      middleware: SmallVec::new(),
      wildcard_node: Some(Box::new(Node::new_wildcard(None))),
      value: value.to_owned(),
      param_key: None,
      is_wildcard: false,
      is_terminal_node: false
    }
  }

  pub fn new_wildcard(param_name: Option<String>) -> Node<T> {
    Node {
      children: HashMap::new(),
      middleware: SmallVec::new(),
      wildcard_node: None,
      value: "*".to_owned(),
      param_key: param_name,
      is_wildcard: true,
      is_terminal_node: false
    }
  }

  pub fn add_route(&mut self, route: &str, middleware: SmallVec<[Middleware<T>; 8]>) {
    // Strip a leading slash
    let mut split_iterator = match route.chars().next() {
      Some('/') => &route[1..],
      _ => route
    }.split('/');

    if let Some(piece) = split_iterator.next() {
      if piece.is_empty() {
        self.is_terminal_node = true;
        self.middleware = middleware
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

  pub fn add_subtree(&mut self, route: &str, subtree: Node<T>) {
    // Strip a leading slash
    let mut split_iterator = match route.chars().next() {
      Some('/') => &route[1..],
      _ => route
    }.split('/');

    if let Some(piece) = split_iterator.next() {
      if piece.is_empty() {
        if let Some(wildcard_node) = &subtree.wildcard_node {
          if wildcard_node.param_key.is_some() {
            self.wildcard_node = match &self.wildcard_node {
              Some(existing_wildcard) => {
                let mut new_wildcard_node = (**existing_wildcard).clone();
                for (key, child) in &subtree.children {
                  new_wildcard_node.children.insert(key.to_owned(), child.clone());
                }

                new_wildcard_node.param_key = wildcard_node.param_key.clone();
                new_wildcard_node.middleware = wildcard_node.middleware.clone();
                new_wildcard_node.is_terminal_node = new_wildcard_node.is_terminal_node || wildcard_node.is_terminal_node;

                Some(Box::new(new_wildcard_node))
              },
              None => Some(wildcard_node.clone())
            };
          } else {
            for (key, child) in &subtree.children {
              self.children.insert(key.to_owned(), child.clone());
            }

            self.middleware = subtree.middleware.clone();
          }
        }
        self.wildcard_node = subtree.wildcard_node.clone();
        self.is_terminal_node = subtree.is_terminal_node;
      } else {
        let mut child = self.children.remove(piece)
          .unwrap_or_else(|| Node::new(piece));

        child.middleware.insert_many(0, &mut subtree.middleware.clone().into_iter());
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

            if results.2 {
              results
            } else {
              match &self.wildcard_node {
                Some(wildcard_node) => (&wildcard_node.middleware, results.1, wildcard_node.is_terminal_node),
                None => {
                  if !self.is_wildcard {
                    results
                  } else {
                    (&self.middleware, results.1, self.is_terminal_node)
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
                (&wildcard_node.middleware, params, false)
              } else if piece.is_empty() && wildcard_node.param_key.is_none() {
                (&wildcard_node.middleware, params, wildcard_node.is_terminal_node)
              } else {
                if let Some(param_key) = &wildcard_node.param_key {
                  params.insert(param_key.to_owned(), piece.to_owned());
                }

                let results = wildcard_node.match_route_with_params(route, params);

                // If the wildcard isn't a terminal node, then try to return this
                // wildcard
                if results.2 {
                  results
                } else {
                  (&self.middleware, results.1, wildcard_node.is_terminal_node)
                }
              }
            }
            None => (&self.middleware, params, self.is_terminal_node)
          }
        }
      }
    } else if self.is_terminal_node {
        (&self.middleware, params, self.is_terminal_node)
    } else if let Some(wildcard_node) = &self.wildcard_node {
      if wildcard_node.param_key == None {
        let results = wildcard_node.match_route_with_params(route, params);

        // If the wildcard isn't a terminal node, then try to return this
        // wildcard
        if results.2 {
          results
        } else {
          (&self.middleware, results.1, self.is_terminal_node)
        }
      } else {
        (&self.middleware, params, self.is_terminal_node)
      }
    } else {
      (&self.middleware, params, self.is_terminal_node)
    }
  }

  /// Used mostly for debugging the route tree
  /// Example usage
  /// ```ignore
  ///   let mut app = App::create(generate_context);
  ///
  ///   app.get("/plaintext", vec![plaintext]);
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
      self.middleware.len(),
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
        children.push((format!("{}/{}", piece, child.value), child.middleware.clone()));
      }
    }

    children
  }

  pub fn copy_node_middleware(&mut self, other_node: &Node<T>) {
    // Copy the other node's middlware over to self
    let len = self.middleware.len();
    self.middleware.insert_many(len, &mut other_node.middleware.clone().into_iter());

    // Match children, recurse if child match
    for (key, child) in &mut self.children {
      if let Some(other_child) = other_node.children.get(key) {
        child.copy_node_middleware(other_child);
      }
    }

    // Copy over wildcards
    if let Some(mut wildcard_node) = self.wildcard_node.clone() {
      if let Some(other_wildcard_node) = &other_node.wildcard_node {
        wildcard_node.copy_node_middleware(other_wildcard_node);
        self.wildcard_node = Some(wildcard_node);
      }
    }
  }

  ///
  /// Pushes middleware down to the leaf nodes, accumulating along the way. This is helpful for
  /// propagating generic middleware down the stack
  ///
  pub fn push_middleware_to_populated_nodes(&mut self, other_node: &Node<T>, accumulated_middleware: SmallVec<[Middleware<T>; 8]>) {
    let fake_node = Node::new("");
    let mut _accumulated_middleware = SmallVec::new();

    let mut len = _accumulated_middleware.len();
    _accumulated_middleware.insert_many(len, &mut other_node.middleware.clone().into_iter());
    len = _accumulated_middleware.len();
    _accumulated_middleware.insert_many(len, &mut accumulated_middleware.into_iter());

    // Copy the other node's middleware over to self
    if !self.middleware.is_empty() {
      let old_middleware = self.middleware.clone();
      self.middleware = SmallVec::new();

      self.middleware.insert_many(0, &mut _accumulated_middleware.clone().into_iter());
      let len = self.middleware.len();
      self.middleware.insert_many(len, &mut other_node.middleware.clone().into_iter());
      let len = self.middleware.len();
      self.middleware.insert_many(len, &mut old_middleware.into_iter());
    }

    // Match children, recurse if child match
    for (key, child) in &mut self.children {
      // Traverse the child tree, or else make a dummy node
      let other_child = other_node.children.get(key).unwrap_or(&fake_node);

      child.push_middleware_to_populated_nodes(other_child, _accumulated_middleware.clone());
    }

    // Copy over wildcards
    if let Some(mut wildcard_node) = self.wildcard_node.clone() {
      if let Some(other_wildcard_node) = &other_node.wildcard_node {
        wildcard_node.push_middleware_to_populated_nodes(other_wildcard_node, _accumulated_middleware.clone());
        self.wildcard_node = Some(wildcard_node);
      }
    }
  }
}
