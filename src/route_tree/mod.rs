mod node;

use std::collections::HashMap;

pub use self::node::Node;
use context::Context;
use middleware::{MiddlewareChain};

pub enum Method {
  DELETE,
  GET,
  POST,
  PUT,
  UPDATE
}

pub struct RouteTree<T: 'static + Context + Send> {
  pub root_node: Node<T>,
  pub generic_root_node: Node<T>,
  pub specific_root_node: Node<T>
}

fn method_to_prefix<'a>(method: &Method) -> &'a str {
  match method {
    Method::DELETE => "__DELETE__",
    Method::GET => "__GET__",
    Method::POST => "__POST__",
    Method::PUT => "__PUT__",
    Method::UPDATE => "__UPDATE__"
  }
}

impl<T: 'static + Context + Send> RouteTree<T> {
  pub fn new() -> RouteTree<T> {
    RouteTree {
      root_node: Node::new(""),
      generic_root_node: Node::new(""),
      specific_root_node: Node::new("")
    }
  }

  ///
  /// Updates the `root_node` of the tree by merging the generic tree into the specific tree. This
  /// is necessary after adding any routes to ensure that the matching functions of the tree are
  /// up to date.
  ///
  pub fn update_root_node(&mut self) {
    let mut root_node = self.specific_root_node.clone();

    root_node.push_middleware_to_populated_nodes(&self.generic_root_node, &MiddlewareChain::new());

    self.root_node = root_node;
  }

  pub fn add_use_node(&mut self, route: &str, middleware: MiddlewareChain<T>) {
    self.generic_root_node.add_route(route, middleware);
    self.update_root_node();
  }

  pub fn add_route_with_method(&mut self, method: &Method, route: &str, middleware: MiddlewareChain<T>) {
    let prefix = method_to_prefix(&method);

    let full_route = format!("{}{}", prefix, route);

    self.specific_root_node.add_route(&full_route, middleware);
    self.update_root_node();
  }

  pub fn add_route(&mut self, route: &str, middleware: MiddlewareChain<T>) {
    self.specific_root_node.add_route(route, middleware);
    self.update_root_node();
  }

  fn _adopt_sub_app_method_to_self(&mut self, route: &str, mut route_tree: RouteTree<T>, method: &Method) -> RouteTree<T> {
    let method_prefix = method_to_prefix(&method);

    let mut self_routes = self.specific_root_node.children.remove(method_prefix)
      .unwrap_or_else(|| Node::new(method_prefix));

    if let Some(tree_routes) = route_tree.root_node.children.remove(method_prefix) {
      self_routes.add_subtree(route, tree_routes);
    }

    self.specific_root_node.children.insert(method_prefix.to_owned(), self_routes);

    // Return ownership
    route_tree
  }

  pub fn add_route_tree(&mut self, route: &str, mut route_tree: RouteTree<T>) {
    route_tree = self._adopt_sub_app_method_to_self(route, route_tree, &Method::DELETE);
    route_tree = self._adopt_sub_app_method_to_self(route, route_tree, &Method::GET);
    route_tree = self._adopt_sub_app_method_to_self(route, route_tree, &Method::POST);
    route_tree = self._adopt_sub_app_method_to_self(route, route_tree, &Method::PUT);
    self._adopt_sub_app_method_to_self(route, route_tree, &Method::UPDATE);
    self.update_root_node();
  }

  pub fn match_route(&self, route: &str) -> (HashMap<String, String>, &MiddlewareChain<T>) {
    let results = self.root_node.match_route(route.split('/'));

    (results.0, results.2)
  }

  pub fn match_route_with_params(&self, route: &str, params: HashMap<String, String>) -> (HashMap<String, String>, &MiddlewareChain<T>) {
    let results = self.root_node.match_route_with_params(route.split('/'), params);

    (results.0, results.2)
  }
}

impl<T: 'static + Context + Send> Default for RouteTree<T> {
  fn default() -> RouteTree<T> {
    RouteTree::new()
  }
}

#[cfg(test)]
mod tests {
  use super::RouteTree;
  use super::Method;
  use std::collections::HashMap;
  use builtins::basic_context::BasicContext;
  use middleware::{MiddlewareChain, MiddlewareReturnValue};
  use futures::{future, Future};
  use std::boxed::Box;

  #[test]
  fn it_should_match_a_simple_route() {
    let mut route_tree = RouteTree::new();

    fn test_function(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> MiddlewareReturnValue<BasicContext> {
      context.body("Hello");

      Box::new(future::ok(context))
    }

    route_tree.add_route_with_method(&Method::GET, "/a/b", middleware![BasicContext => test_function]);
    let middleware = route_tree.match_route_with_params("__GET__/a/b", HashMap::new());

    let result = middleware.1.run(BasicContext::new())
      .wait()
      .unwrap();

    assert!(result.get_body() == "Hello");
  }

  #[test]
  fn it_should_match_a_simple_route_with_a_param() {
    let mut route_tree = RouteTree::new();

    fn test_function(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> MiddlewareReturnValue<BasicContext> {
      let body = &context.params.get("key").unwrap().clone();

      context.body(body);

      Box::new(future::ok(context))
    }

    route_tree.add_route_with_method(&Method::GET, "/:key", middleware![BasicContext => test_function]);
    let middleware = route_tree.match_route("__GET__/value");

    let mut context = BasicContext::new();
    context.params = middleware.0;

    let result = middleware.1.run(context)
      .wait()
      .unwrap();

    assert!(result.get_body() == "value");
  }

  #[test]
  fn it_should_compose_multiple_trees() {
    fn test_function1(context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> MiddlewareReturnValue<BasicContext> {
      Box::new(future::ok(context))
    }

    fn test_function2(mut context: BasicContext, _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext>  + Send + Sync) -> MiddlewareReturnValue<BasicContext> {
      context.body("Hello");

      Box::new(future::ok(context))
    }

    let mut route_tree2 = RouteTree::new();
    route_tree2.add_route_with_method(&Method::GET, "/c", middleware![BasicContext => test_function2]);

    let mut route_tree1 = RouteTree::new();
    route_tree1.add_route_with_method(&Method::GET, "/a", middleware![BasicContext => test_function1]);
    route_tree1.add_route_tree("/b", route_tree2);

    let middleware = route_tree1.match_route_with_params("__GET__/b/c", HashMap::new());

    let result = middleware.1.run(BasicContext::new())
      .wait()
      .unwrap();

    assert!(result.get_body() == "Hello");
  }
}
