use std::collections::HashMap;

use crate::core::context::Context;
use crate::core::route_tree::RouteTree;

use crate::core::middleware::MiddlewareChain;

pub struct MatchedRoute<'a, T: 'static + Context + Send> {
    pub middleware: &'a MiddlewareChain<T>,
    pub value: String,
    pub params: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
}

pub struct RouteParser<T: 'static + Context + Send> {
    pub route_tree: RouteTree<T>,
    pub shortcuts: HashMap<String, MiddlewareChain<T>>,
}

impl<T: Context + Send> RouteParser<T> {
    pub fn new() -> RouteParser<T> {
        RouteParser {
            route_tree: RouteTree::new(),
            shortcuts: HashMap::new(),
        }
    }

    pub fn add_method_agnostic_middleware(&mut self, route: &str, middleware: MiddlewareChain<T>) {
        self.route_tree.add_use_node(route, middleware);
    }

    pub fn add_route(&mut self, route: &str, middleware: MiddlewareChain<T>) {
        self.route_tree.add_route(route, middleware);
    }

    pub fn optimize(&mut self) {
        let routes = self.route_tree.root_node.get_route_list();

        for (path, middleware, is_terminal_node) in routes {
            if is_terminal_node {
                self.shortcuts.insert((&path[1..]).to_owned(), middleware);
            }
        }
    }

    #[inline]
    pub fn match_route(&self, route: &str) -> MatchedRoute<T> {
        let query_params = HashMap::new();

        let split_route = route.find('?');
        let mut route = match split_route {
            Some(index) => &route[0..index],
            None => route,
        };

        // Trim trailing slashes
        while &route[route.len() - 1..route.len()] == "/" {
            route = &route[0..route.len() - 1];
        }

        if let Some(shortcut) = self.shortcuts.get(route) {
            MatchedRoute {
                middleware: shortcut,
                value: route.to_owned(),
                params: HashMap::new(),
                query_params,
            }
        } else {
            let matched = self.route_tree.match_route(route);

            MatchedRoute {
                middleware: matched.1,
                value: route.to_owned(),
                params: matched.0,
                query_params,
            }
        }
    }
}

impl<T: Context + Send> Default for RouteParser<T> {
    fn default() -> RouteParser<T> {
        RouteParser::new()
    }
}
