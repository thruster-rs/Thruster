mod node;

use std::collections::HashMap;

pub use self::node::Node;
use crate::core::context::Context;
use crate::core::middleware::MiddlewareChain;

pub enum Method {
    DELETE,
    GET,
    OPTIONS,
    POST,
    PUT,
    PATCH,
}

pub struct RouteTree<T: 'static + Context + Send> {
    pub root_node: Node<T>,
    pub generic_root_node: Node<T>,
    pub specific_root_node: Node<T>,
}

fn method_to_prefix<'a>(method: &Method) -> &'a str {
    match method {
        Method::DELETE => "__DELETE__",
        Method::GET => "__GET__",
        Method::OPTIONS => "__OPTIONS__",
        Method::POST => "__POST__",
        Method::PUT => "__PUT__",
        Method::PATCH => "__UPDATE__",
    }
}

impl<T: 'static + Context + Send> RouteTree<T> {
    pub fn new() -> RouteTree<T> {
        RouteTree {
            root_node: Node::new(""),
            generic_root_node: Node::new(""),
            specific_root_node: Node::new(""),
        }
    }

    ///
    /// Updates the `root_node` of the tree by merging the generic tree into the specific tree. This
    /// is necessary after adding any routes to ensure that the matching functions of the tree are
    /// up to date.
    ///
    pub fn update_root_node(&mut self) {
        let mut root_node = self.specific_root_node.clone();

        root_node
            .push_middleware_to_populated_nodes(&self.generic_root_node, &MiddlewareChain::new());

        self.root_node = root_node;
    }

    pub fn add_use_node(&mut self, route: &str, middleware: MiddlewareChain<T>) {
        self.generic_root_node.add_route(route, middleware, route.to_string());
        self.update_root_node();
    }

    pub fn add_route_with_method(
        &mut self,
        method: &Method,
        route: &str,
        middleware: MiddlewareChain<T>,
    ) {
        let prefix = method_to_prefix(&method);

        let full_route = format!("{}{}", prefix, route);

        self.specific_root_node.add_route(&full_route, middleware, full_route.clone());
        self.update_root_node();
    }

    pub fn add_route(&mut self, route: &str, middleware: MiddlewareChain<T>) {
        self.specific_root_node.add_route(route, middleware, route.to_string());
        self.update_root_node();
    }

    fn _adopt_sub_app_method_to_self(
        &mut self,
        route: &str,
        mut route_tree: RouteTree<T>,
        method: &Method,
    ) -> RouteTree<T> {
        let method_prefix = method_to_prefix(&method);

        let mut self_routes = self
            .specific_root_node
            .children
            .remove(method_prefix)
            .unwrap_or_else(|| Node::new(method_prefix));

        if let Some(tree_routes) = route_tree.root_node.children.remove(method_prefix) {
            self_routes.add_subtree(route, tree_routes);
        }

        self.specific_root_node
            .children
            .insert(method_prefix.to_owned(), self_routes);

        // Return ownership
        route_tree
    }

    pub fn add_route_tree(&mut self, route: &str, mut route_tree: RouteTree<T>) {
        route_tree = self._adopt_sub_app_method_to_self(route, route_tree, &Method::DELETE);
        route_tree = self._adopt_sub_app_method_to_self(route, route_tree, &Method::GET);
        route_tree = self._adopt_sub_app_method_to_self(route, route_tree, &Method::OPTIONS);
        route_tree = self._adopt_sub_app_method_to_self(route, route_tree, &Method::POST);
        route_tree = self._adopt_sub_app_method_to_self(route, route_tree, &Method::PUT);
        self._adopt_sub_app_method_to_self(route, route_tree, &Method::PATCH);
        self.update_root_node();
    }

    pub fn match_route(&self, route: &str) -> (HashMap<String, String>, &MiddlewareChain<T>, &str) {
        let results = self.root_node.match_route(route.split('/'));

        (results.0, results.2, results.3)
    }

    pub fn match_route_with_params(
        &self,
        route: &str,
        params: HashMap<String, String>,
    ) -> (HashMap<String, String>, &MiddlewareChain<T>, &str) {
        let results = self
            .root_node
            .match_route_with_params(route.split('/'), params);

        (results.0, results.2, results.3)
    }
}

impl<T: 'static + Context + Send> Default for RouteTree<T> {
    fn default() -> RouteTree<T> {
        RouteTree::new()
    }
}
