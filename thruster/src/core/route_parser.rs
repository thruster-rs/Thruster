use std::collections::HashMap;

use crate::core::context::Context;
use crate::core::route_tree::RouteTree;

use crate::core::middleware::MiddlewareChain;

pub struct MatchedRoute<'a, 'b, T: 'static + Context + Send> {
    pub middleware: &'a MiddlewareChain<T>,
    pub value: String,
    pub path: &'b str,
    pub params: Option<HashMap<String, String>>,
    pub query_params: Option<HashMap<String, String>>,
}

pub struct RouteParser<T: 'static + Context + Send> {
    pub route_tree: RouteTree<T>,
    pub shortcuts: Vec<(String, (MiddlewareChain<T>, String))>,
}

impl<T: Context + Send> RouteParser<T> {
    pub fn new() -> RouteParser<T> {
        RouteParser {
            route_tree: RouteTree::new(),
            shortcuts: vec![],
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
                self.shortcuts.push((
                    (&path[1..]).to_owned(),
                    (middleware, (&path[1..]).to_string()),
                ));
            }
        }
    }

    #[inline]
    pub fn match_route<'a>(&'a self, route: String) -> MatchedRoute<'a, '_, T> {
        let split_route = route.find('?');
        let mut no_query_route = match split_route {
            Some(index) => &route[0..index],
            None => &route,
        };

        // Trim trailing slashes
        while &no_query_route[no_query_route.len() - 1..no_query_route.len()] == "/" {
            no_query_route = &no_query_route[0..no_query_route.len() - 1];
        }

        for (shortcut_route, (shortcut, path)) in &self.shortcuts {
            if shortcut_route == no_query_route {
                return MatchedRoute {
                    middleware: shortcut,
                    value: route,
                    path,
                    params: None,
                    query_params: None,
                };
            }
        }

        let matched = self.route_tree.match_route(&no_query_route);

        MatchedRoute {
            middleware: matched.1,
            value: route,
            path: matched.2,
            params: matched.0,
            query_params: None,
        }
    }
}

impl<T: Context + Send> Default for RouteParser<T> {
    fn default() -> RouteParser<T> {
        RouteParser::new()
    }
}
