use std::collections::HashMap;
use smallvec::SmallVec;
use std::str::Split;

use middleware::{Middleware};
use context::Context;

pub struct Node<T: Context + Send> {
  middleware: SmallVec<[Middleware<T>; 8]>,
  pub children: HashMap<String, Node<T>>,
  wildcard_node: Option<Box<Node<T>>>,
  param_key: Option<String>,
  pub value: String
}

impl<T: Context + Send> Clone for Node<T> {
  fn clone(&self) -> Self {
    let children = self.children.clone();
    let middleware = self.middleware.clone();

    let wildcard = match &self.wildcard_node {
      Some(wildcard) => (**wildcard).clone(),
      None => Node::new_wildcard(None)
    };

    Node {
      children: children,
      middleware: middleware,
      wildcard_node: Some(Box::new(wildcard)),
      value: self.value.clone(),
      param_key: self.param_key.clone()
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
      param_key: None
    }
  }

  pub fn new_wildcard(param_name: Option<String>) -> Node<T> {
    Node {
      children: HashMap::new(),
      middleware: SmallVec::new(),
      wildcard_node: None,
      value: "*".to_owned(),
      param_key: param_name
    }
  }

  pub fn add_route(&mut self, route: &str, middleware: SmallVec<[Middleware<T>; 8]>) {
    // Strip a leading slash
    let mut split_iterator = match route.chars().next() {
      Some('/') => &route[1..],
      _ => route
    }.split("/");

    if let Some(piece) = split_iterator.next() {
      if piece.len() == 0 {
        self.middleware = middleware
      } else {
        let is_param = piece.chars().next().unwrap() == ':';
         match is_param {
          false => {
            let mut child = self.children.remove(piece)
              .unwrap_or_else(|| Node::new(piece));

            child.add_route(&split_iterator.collect::<SmallVec<[&str; 8]>>().join("/"), middleware);

            self.children.insert(piece.to_owned(), child);
          },
          true => {
            let mut wildcard = Node::new_wildcard(Some(piece[1..].to_owned()));

            wildcard.add_route(&split_iterator.collect::<SmallVec<[&str; 8]>>().join("/"), middleware);

            self.wildcard_node = Some(Box::new(wildcard));
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
    }.split("/");

    if let Some(piece) = split_iterator.next() {
      if piece.len() == 0 {
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
      } else {
        let mut child = self.children.remove(piece)
          .unwrap_or_else(|| Node::new(piece));

        child.add_subtree(&split_iterator.collect::<SmallVec<[&str; 8]>>().join("/"), subtree);

        self.children.insert(piece.to_owned(), child);
      }
    }
  }

  pub fn is_terminal(&self) -> bool {
    self.middleware.len() > 0
  }

  pub fn match_route(&self, route: Split<&str>) -> (&SmallVec<[Middleware<T>; 8]>, HashMap<String, String>) {
    self.match_route_with_params(route, HashMap::new())
  }

  pub fn match_route_with_params(&self, mut route: Split<&str>, mut params: HashMap<String, String>) -> (&SmallVec<[Middleware<T>; 8]>, HashMap<String, String>) {
    if let Some(piece) = route.next() {
      match self.children.get(piece) {
        Some(child) => child.match_route_with_params(route, params),
        None => {
          match &self.wildcard_node {
            Some(wildcard_node) => {
              if let Some(param_key) = &wildcard_node.param_key {
                params.insert(param_key.to_owned(), piece.to_owned());
              }
              wildcard_node.match_route_with_params(route, params)
            }
            None => (&self.middleware, params)
          }
        }
      }
    } else {
      (&self.middleware, params)
    }
  }

  pub fn to_string(&self, indent: &str) -> String {
    let mut in_progress = "".to_owned();
    let value = self.param_key.clone().unwrap_or(self.value.to_owned());

    in_progress = format!("{}\n{}{}: {}",
      in_progress,
      indent,
      value,
      self.middleware.len());

    for (_, child) in &self.children {
      in_progress = format!("{}{}", in_progress, child.to_string(&format!("{}  ", indent)));
    }

    if let Some(wildcard_node) = &self.wildcard_node {
      in_progress = format!("{}{}", in_progress, wildcard_node.to_string(&format!("{}  ", indent)));
    }

    in_progress
  }

  pub fn enumerate(&self) -> SmallVec<[(String, SmallVec<[Middleware<T>; 8]>); 8]> {
    let mut children = SmallVec::new();

    for (_key, child) in &self.children {
      let piece = match &self.param_key {
        Some(param_key) => format!(":{}", param_key),
        None => self.value.clone()
      };

      let child_enumeration = child.enumerate();

      if child_enumeration.len() > 0 {
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
    for (key, child) in self.children.iter_mut() {
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

  pub fn push_middleware_to_populated_nodes(&mut self, other_node: &Node<T>, accumulated_middleware: SmallVec<[Middleware<T>; 8]>) {
    let fake_node = Node::new("");
    let mut _accumulated_middleware = SmallVec::new();

    let mut len = _accumulated_middleware.len();
    _accumulated_middleware.insert_many(len, &mut other_node.middleware.clone().into_iter());
    len = _accumulated_middleware.len();
    _accumulated_middleware.insert_many(len, &mut accumulated_middleware.into_iter());

    // Copy the other node's middleware over to self
    if self.middleware.len() > 0 {
      let old_middleware = self.middleware.clone();
      self.middleware = SmallVec::new();

      self.middleware.insert_many(0, &mut _accumulated_middleware.clone().into_iter());
      let len = self.middleware.len();
      self.middleware.insert_many(len, &mut other_node.middleware.clone().into_iter());
      let len = self.middleware.len();
      self.middleware.insert_many(len, &mut old_middleware.into_iter());
    }

    // Match children, recurse if child match
    for (key, child) in self.children.iter_mut() {
      // Travers the child tree, or else make a dummy node
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
