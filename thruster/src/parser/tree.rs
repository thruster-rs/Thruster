use crate::ReusableBoxFuture;
use fnv::FnvHashMap;

use std::str::Split;
use std::{fmt, fmt::Debug};

use crate::core::context::Context;
use crate::core::errors::ThrusterError;
use crate::parser::middleware_traits::*;

const ROOT_ROUTE_ID: &str = "__root__";
const WILDCARD_ROUTE_ID: char = '*';
const PARAM_ROUTE_LEADING_CHAR: char = ':';

/// A single node in the route parse tree.
pub struct Node<T: Clone + Send> {
    /// The value of the node. Not every node has a value, for example in the path /a/b, two nodes are created;
    /// a which has no value, but a child of b, and b, which has no children, but a value of a.
    value: Option<MiddlewareTuple<T>>,

    /// The wildcard node which will be matched against given no children match this value.
    wildcard_node: Option<Box<Node<T>>>,

    /// The path piece of the param that this node matches against.
    path_piece: String,

    /// The param name that will be named with this node. If a param name is set, then this node will match for
    /// all input paths.
    param_name: Option<String>,

    /// The nodes which are children to this node. Empty if node is a leaf.
    children: Vec<Node<T>>,

    /// The committed middleware, only usable once node has been consumed and replaced.
    committed_middleware:
        Box<dyn Fn(T) -> ReusableBoxFuture<Result<T, ThrusterError<T>>> + Send + Sync>,

    /// Whether or not this node has committed middleware.
    has_committed_middleware: bool,

    /// The committed middleware represented as a tuple. This is helpful because the tuple is clonable.
    committed_value: Option<MiddlewareTuple<T>>,

    /// Is the node a leaf node or not. Leaf nodes are nodes in which a route can be terminated.
    is_leaf: bool,

    /// If there is a non-leaf value to this node, then this is the middleware that represents it
    /// and should be pushed down the tree on commit.
    non_leaf_value: Option<MiddlewareTuple<T>>,

    /// A shortcut for fast matching so that we don't have to traverse the tree for trivial matches
    fastmatch_map: FnvHashMap<
        String,
        Box<dyn Fn(T) -> ReusableBoxFuture<Result<T, ThrusterError<T>>> + Send + Sync>,
    >,

    /// Should respect "strict mode", that is disambiguating /a and /a/
    pub(crate) strict_mode: bool,
}

impl<T: Clone + Send> Debug for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
            .field("path", &self.path_piece)
            .field("has value?", &self.value.is_some())
            .finish()
    }
}

#[derive(Debug, Default)]
pub struct Params {
    inner: Vec<Param>,
}

impl Params {
    pub fn get(&self, key: &str) -> Option<&Param> {
        self.inner.iter().find(|p| p.key == key)
    }

    pub fn add(&mut self, key: &str, val: &str) {
        self.inner.push(Param {
            key: key.to_owned(),
            param: val.to_owned(),
        })
    }
}

#[derive(Debug, Default)]
pub struct Param {
    pub key: String,
    pub param: String,
}

pub struct NodeOutput<'m, T> {
    pub value: &'m Box<dyn Fn(T) -> ReusableBoxFuture<Result<T, ThrusterError<T>>> + Send + Sync>,
    pub params: Params,
    pub path: String,
    pub was_terminal_leaf: bool,
}

pub struct OwnedNodeOutput<'m, T> {
    pub value: &'m Box<dyn Fn(T) -> ReusableBoxFuture<Result<T, ThrusterError<T>>> + Send + Sync>,
    pub params: Option<std::collections::HashMap<String, String>>,
    pub path: String,
}

impl<'m, T> NodeOutput<'m, T> {
    pub fn into_owned(self) -> OwnedNodeOutput<'m, T> {
        OwnedNodeOutput {
            value: self.value,
            params: None,
            path: self.path.to_owned(),
        }
    }
}

impl<T: 'static + Context + Clone + Send> Default for Node<T> {
    fn default() -> Node<T> {
        Node {
            value: None,
            wildcard_node: None,
            path_piece: ROOT_ROUTE_ID.to_string(),
            param_name: None,
            children: vec![],
            committed_middleware: Box::new(|mut c| {
                ReusableBoxFuture::new(async move {
                    c.status(404);

                    Err(ThrusterError {
                        context: c,
                        message: "Not found".to_string(),
                        cause: None,
                    })
                })
            }),
            committed_value: None,
            has_committed_middleware: false,
            is_leaf: false,
            non_leaf_value: None,
            fastmatch_map: FnvHashMap::default(),
            strict_mode: false,
        }
    }
}

// impl<T, A: MiddlewareFunc<T> + 'static, B: MiddlewareFunc<T> + 'static>
//     CombineMiddleware<T, (A,), (B,), (A, B)> for (A,)
// {
//     fn combine(self, funcs: (B,)) -> (A, B) {
//         #[allow(unused_parens, non_snake_case)]
//         let (a,) = self;
//         #[allow(unused_parens, non_snake_case)]
//         let (b,) = funcs;

//         (a, b)
//     }
// }

impl<T: 'static + Context + Clone + Send> Node<T> {
    /// Gets the value at the end of the path.
    pub fn get_value_at_path<'m, 'k: 'm>(&'k self, path: String) -> NodeOutput<'m, T> {
        if let Some(value) = self.fastmatch_map.get(&path) {
            return NodeOutput {
                value,
                params: Params::default(),
                path,
                was_terminal_leaf: true,
            };
        }

        // Trim the query parameters -- we can let a middleware handle that later.
        let mut split = path.split('?').next().unwrap_or("").split('/');
        let _ = split.next();

        self.get_value_at_split_path(split)
    }

    /// Adds the value at the given path.
    pub fn add_value_at_path(&mut self, path: &str, value: MiddlewareTuple<T>) {
        let mut split = path.split('/');

        // Strip any initial '/' chars
        let mut chars = path.chars();
        while chars.next() == Some('/') {
            let _ = split.next();
        }

        self.add_value_at_split_path(split, value, true)
    }

    /// Adds the value at the given path as a non-leaf (i.e. middleware.)
    pub fn add_non_leaf_value_at_path(&mut self, path: &str, value: MiddlewareTuple<T>) {
        let mut split = path.split('/');

        // Strip any initial '/' chars
        let mut chars = path.chars();
        while chars.next() == Some('/') {
            let _ = split.next();
        }

        self.add_value_at_split_path(split, value, false)
    }

    /// Merges this node with another node, attempting to combine values at matching paths.
    pub fn add_node_at_path(&mut self, path: &str, mut added_node: Node<T>) {
        let mut split = path.split('/');

        // Strip any initial '/' chars
        let mut chars = path.chars();
        while chars.next() == Some('/') {
            let _ = split.next();
        }

        let mut node = self.get_node_at_split_path(split.clone());

        if let Some(node) = node.as_mut() {
            node.value = match node.value.take() {
                Some(val) => {
                    if let Some(added_middleware) = added_node.value {
                        Some(val.combine(added_middleware))
                    } else {
                        Some(val)
                    }
                }
                None => added_node.value,
            };

            // Also consider wildcards here
            let wildcard_node = node.wildcard_node.take();
            let added_wildcard_node = added_node.wildcard_node.take();

            node.wildcard_node = match (wildcard_node, added_wildcard_node) {
                (Some(mut wildcard_node), Some(mut added_wildcard_node)) => {
                    let wildcard_node_middleware = wildcard_node.value.take();
                    let added_wildcard_node_middleware = added_wildcard_node.value.take();

                    let middleware =
                        match (wildcard_node_middleware, added_wildcard_node_middleware) {
                            (
                                Some(wildcard_node_middleware),
                                Some(added_wildcard_node_middleware),
                            ) => Some(
                                wildcard_node_middleware.combine(added_wildcard_node_middleware),
                            ),
                            (None, Some(added_wildcard_node_middleware)) => {
                                Some(added_wildcard_node_middleware)
                            }
                            (Some(wildcard_node_middleware), None) => {
                                Some(wildcard_node_middleware)
                            }
                            _ => None,
                        };
                    wildcard_node.value = middleware;

                    Some(wildcard_node)
                }
                (None, Some(added_wildcard_node)) => Some(added_wildcard_node),
                (Some(wildcard_node), None) => Some(wildcard_node),
                _ => None,
            };

            for child in added_node.children {
                self.add_node_at_path(&format!("{}/{}", path, child.path_piece), child);
            }
        } else {
            let mut last_node = self;
            let split_vec = split.collect::<Vec<&str>>();

            for i in 0..split_vec.len() - 1 {
                let piece = split_vec.get(i).unwrap().to_owned();
                let children: &mut Vec<Node<T>> = &mut last_node.children;

                if let Some(index) = children.iter().position(|n| n.path_piece == piece) {
                    last_node = children.get_mut(index).unwrap();
                } else {
                    let next_node = Node::<T> {
                        path_piece: piece.to_owned(),
                        ..Default::default()
                    };

                    children.push(next_node);

                    last_node = children.get_mut(0).unwrap();
                }
            }

            added_node.path_piece = split_vec.last().unwrap().to_string();
            last_node.children.push(added_node);
        }
    }

    pub(crate) fn get_node_at_split_path(
        &mut self,
        mut split: Split<char>,
    ) -> Option<&mut Node<T>> {
        let path_piece = split.next();

        match path_piece {
            None => Some(self),
            Some(path_piece) => match path_piece.chars().next() {
                Some(PARAM_ROUTE_LEADING_CHAR) => self
                    .wildcard_node
                    .as_mut()
                    .and_then(|w| w.get_node_at_split_path(split)),
                Some(WILDCARD_ROUTE_ID) => self
                    .wildcard_node
                    .as_mut()
                    .and_then(|w| w.get_node_at_split_path(split)),
                Some(_) => {
                    for child in self.children.iter_mut() {
                        if child.path_piece == path_piece {
                            return child.get_node_at_split_path(split);
                        }
                    }

                    None
                }
                None => None,
            },
        }
    }

    #[allow(dead_code)]
    pub(crate) fn merge_node(&mut self, mut node: Node<T>) {
        // Merge this node
        let node_value = node.value.take();
        self.value = self.value.take().map(|v| match node_value {
            Some(node_value) => v.combine(node_value),
            None => v,
        });

        let mut missing_nodes = vec![];

        // Merge child nodes, yes n^2, but this is on build so it's only done on init.
        for incoming_child in node.children.into_iter() {
            let path_piece = incoming_child.path_piece.clone();

            let child = self
                .children
                .iter_mut()
                .find(|child| child.path_piece == path_piece);

            match child {
                Some(child) => child.merge_node(incoming_child),
                None => missing_nodes.push(incoming_child),
            };
        }

        self.children.append(&mut missing_nodes);
    }

    pub(crate) fn get_value_at_split_path<'m, 'k: 'm>(
        &'k self,
        mut path: Split<char>,
    ) -> NodeOutput<'m, T> {
        let path_piece = path.next();

        match path_piece {
            None => NodeOutput {
                value: &self.committed_middleware,
                params: Params::default(),
                path: "".to_owned(),
                was_terminal_leaf: self.is_leaf,
            },
            Some(path_piece) => {
                // Check exact children
                for child in self.children.iter() {
                    if child.path_piece == path_piece {
                        let mut res = child.get_value_at_split_path(path);

                        if let Some(param) = &self.param_name {
                            res.params.add(param, path_piece);
                        }

                        if res.was_terminal_leaf {
                            return res;
                        // If we have found a match, but it's non-terminal, then
                        // we should check any wildcards at this level.
                        } else if let Some(wildcard) = &self.wildcard_node {
                            return NodeOutput {
                                value: &wildcard.committed_middleware,
                                params: Params::default(),
                                path: "".to_owned(),
                                was_terminal_leaf: wildcard.is_leaf,
                            };
                        // Otherwise just toss the result and hope something higher up picks
                        // it up.
                        } else {
                            return res;
                        }
                    }
                }

                // Check wildcard child
                match self.wildcard_node.as_ref() {
                    Some(wildcard_node) => {
                        let mut res = wildcard_node.get_value_at_split_path(path);
                        if let Some(param) = &self.param_name {
                            res.params.add(param, path_piece);
                        }

                        res
                    }
                    None => NodeOutput {
                        value: &self.committed_middleware,
                        params: Params::default(),
                        path: "".to_owned(),
                        was_terminal_leaf: self.is_leaf,
                    },
                }
            }
        }
    }

    pub(crate) fn add_value_at_split_path(
        &mut self,
        mut path: Split<char>,
        value: MiddlewareTuple<T>,
        is_leaf: bool,
    ) {
        let path_piece = path.next();

        match path_piece {
            Some(path_piece) => {
                match path_piece.chars().next() {
                    Some(PARAM_ROUTE_LEADING_CHAR) => match self.wildcard_node.as_mut() {
                        Some(wildcard_node) => {
                            wildcard_node.add_value_at_split_path(path, value, is_leaf);
                            self.param_name = Some(path_piece[1..].to_string());
                        }
                        None => {
                            let mut wildcard_node = Node::<T> {
                                path_piece: WILDCARD_ROUTE_ID.to_string(),
                                strict_mode: self.strict_mode,
                                ..Default::default()
                            };
                            wildcard_node.add_value_at_split_path(path, value, is_leaf);

                            self.param_name = Some(path_piece[1..].to_string());
                            self.wildcard_node = Some(Box::new(wildcard_node));
                        }
                    },
                    Some(WILDCARD_ROUTE_ID) => match self.wildcard_node.as_mut() {
                        Some(wildcard_node) => {
                            wildcard_node.add_value_at_split_path(path, value, is_leaf);
                        }
                        None => {
                            let mut wildcard_node = Node::<T> {
                                path_piece: WILDCARD_ROUTE_ID.to_string(),
                                strict_mode: self.strict_mode,
                                ..Default::default()
                            };
                            wildcard_node.add_value_at_split_path(path, value, is_leaf);

                            self.wildcard_node = Some(Box::new(wildcard_node));
                        }
                    },
                    Some(_) => {
                        self.add_value_at_split_path_helper(path_piece, path, value, is_leaf);
                    }
                    // Route ending with a `/`
                    None => {
                        if self.strict_mode {
                            self.add_value_at_split_path_helper(path_piece, path, value, is_leaf);
                        } else {
                            self.is_leaf = is_leaf;

                            if self.is_leaf {
                                self.value = Some(value)
                            } else {
                                self.non_leaf_value = Some(value);
                            }
                        }
                    }
                };
            }
            None => {
                self.is_leaf = is_leaf;

                if self.is_leaf {
                    self.value = Some(value)
                } else {
                    self.non_leaf_value = Some(value);
                }
            }
        }
    }

    fn add_value_at_split_path_helper(
        &mut self,
        path_piece: &str,
        path: Split<char>,
        value: MiddlewareTuple<T>,
        is_leaf: bool,
    ) {
        let existing_index = self
            .children
            .iter()
            .position(|n| n.path_piece == path_piece);

        // Note: This operation no longer preserves order.
        let mut child_node = match existing_index {
            Some(i) => self.children.remove(i),
            None => Node::<T> {
                strict_mode: self.strict_mode,
                ..Node::default()
            },
        };

        child_node.path_piece = path_piece.to_owned();
        child_node.add_value_at_split_path(path, value, is_leaf);
        self.children.push(child_node);
    }

    pub(crate) fn enumerate(
        &self,
        path: &str,
    ) -> Vec<(
        String,
        Option<MiddlewareTuple<T>>,
        Option<MiddlewareTuple<T>>,
        bool,
    )> {
        let path = format!("{}/{}", path, self.path_piece);

        let mut enumeration = vec![(
            path.clone(),
            self.committed_value.clone(),
            self.value.clone(),
            self.wildcard_node.is_some(),
        )];

        for child in &self.children {
            enumeration.append(&mut child.enumerate(&path));
        }

        enumeration
    }

    pub(crate) fn commit(self) -> Self {
        let mut committed = self.commit_inner(None);

        let enumerations = committed.enumerate("");

        for (path, committed_value, _value, _wildcard_exists) in enumerations {
            if let Some(value) = committed_value {
                committed.fastmatch_map.insert(path, value.middleware());
            }
        }

        committed
    }

    fn commit_inner(mut self, collected_middleware: Option<MiddlewareTuple<T>>) -> Self {
        let updated_collected_middleware = match self.non_leaf_value {
            Some(non_leaf_value) => match collected_middleware {
                Some(collected_middleware) => Some(collected_middleware.combine(non_leaf_value)),
                None => Some(non_leaf_value),
            },
            None => collected_middleware,
        };

        let children = self
            .children
            .into_iter()
            .map(|c| c.commit_inner(updated_collected_middleware.clone()))
            .collect();
        let has_committed_middleware = self.value.is_some();
        let (committed, committed_tuple) = match self.value.take() {
            Some(v) => match updated_collected_middleware.clone() {
                Some(updated_collected_middleware) => {
                    let combined = updated_collected_middleware.combine(v);

                    (combined.clone().middleware(), Some(combined))
                }
                None => (v.clone().middleware(), Some(v)),
            },
            None => (self.committed_middleware, None),
        };

        Node {
            value: None,
            wildcard_node: self
                .wildcard_node
                .map(|n| Box::new(n.commit_inner(updated_collected_middleware.clone()))),
            path_piece: self.path_piece,
            param_name: self.param_name,
            children,
            committed_middleware: committed,
            committed_value: committed_tuple,
            has_committed_middleware,
            is_leaf: self.is_leaf,
            non_leaf_value: None,
            fastmatch_map: FnvHashMap::default(),
            strict_mode: self.strict_mode,
        }
    }
}

// impl<T: Debug> Node<T> {
//     /// Prints the tree at the current level and all levels beneath with appropraite indentation starting
//     /// with zero indentation.
//     pub fn print(&self) -> String {
//         self.print_with_indentation(0)
//     }

//     pub(crate) fn print_with_indentation(&self, indentation_level: usize) -> String {
//         let mut val = format!(
//             "{}{}: {}",
//             "    ".repeat(indentation_level),
//             self.path_piece,
//             self.has_committed_middleware
//         );

//         for child in self.children.iter() {
//             val = format!(
//                 "{}\n{}",
//                 val,
//                 child.print_with_indentation(indentation_level + 1)
//             );
//         }

//         if let Some(ref wildcard_node) = self.wildcard_node {
//             val = format!(
//                 "{}\n{}",
//                 val,
//                 wildcard_node.print_with_indentation(indentation_level + 1)
//             );
//         }

//         val
//     }
// }

impl<T: Clone + Send> Node<T> {
    /// Prints the tree at the current level and all levels beneath with appropraite indentation starting
    /// with zero indentation.
    pub fn to_string(&self) -> String {
        self.print_with_indentation(0)
    }

    pub(crate) fn print_with_indentation(&self, indentation_level: usize) -> String {
        let mut val = format!(
            "{}{}: {} ({})",
            "    ".repeat(indentation_level),
            self.path_piece,
            self.value.is_some(),
            if self.has_committed_middleware {
                "committed"
            } else {
                "uncommitted"
            }
        );

        for child in self.children.iter() {
            val = format!(
                "{}\n{}",
                val,
                child.print_with_indentation(indentation_level + 1)
            );
        }

        if let Some(ref wildcard_node) = self.wildcard_node {
            val = format!(
                "{}\n{}",
                val,
                wildcard_node.print_with_indentation(indentation_level + 1)
            );
        }

        val
    }
}

#[cfg(test)]
pub mod test {
    use bytes::Bytes;

    use super::*;
    use crate::pinbox;

    impl Context for i32 {
        type Response = i32;

        fn get_response(self) -> Self::Response {
            0
        }

        /// set_body is used to set the body using a vec of bytes on the context. The
        /// contents will be used later for generating the correct response.
        fn set_body(&mut self, body: Vec<u8>) {
            let _ = std::mem::replace(
                self,
                str::parse::<i32>(std::str::from_utf8(&body).unwrap_or("")).unwrap_or(0),
            );
        }

        /// set_body_byte is used to set the body using a Bytes object on the context.
        /// The contents will be used later for generating the correct response.
        fn set_body_bytes(&mut self, bytes: Bytes) {
            let _ = std::mem::replace(
                self,
                str::parse::<i32>(std::str::from_utf8(&bytes).unwrap_or("")).unwrap_or(0),
            );
        }

        /// route is used to return the route from the incoming request as a string.
        fn route(&self) -> &str {
            ""
        }

        /// set is used to set a header on the outgoing response.
        fn set(&mut self, _key: &str, _value: &str) {
            panic!("Don't set a header...");
        }

        /// remove is used to remove a header on the outgoing response.
        fn remove(&mut self, _key: &str) {
            panic!("Don't set a header...");
        }
    }

    #[test]
    fn it_should_print_values() {
        let mut root: Node<i32> = Node::default();

        async fn f1(a: i32, _b: NextFn<i32>) -> Result<i32, ThrusterError<i32>> {
            Ok(a + 1)
        }

        async fn f2(a: i32, _b: NextFn<i32>) -> Result<i32, ThrusterError<i32>> {
            Ok(a + 2)
        }

        root.add_value_at_path("a/b/c", MiddlewareTuple::A(pinbox!(i32, f1)));
        root.add_value_at_path("a/b/d", MiddlewareTuple::A(pinbox!(i32, f2)));

        let printed = root.commit().to_string();

        assert!(
            printed
                == format!(
                    "{}: false (uncommitted)
    a: false (uncommitted)
        b: false (uncommitted)
            c: false (committed)
            d: false (committed)",
                    ROOT_ROUTE_ID
                ),
            "it should print correct looking trees."
        );
    }

    #[test]
    fn it_should_return_the_value_at_a_simple_path() {
        async fn f1(a: i32, _b: NextFn<i32>) -> Result<i32, ThrusterError<i32>> {
            Ok(a + 1)
        }

        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                let mut root: Node<i32> = Node::default();

                root.add_value_at_path("a", MiddlewareTuple::A(pinbox!(i32, f1)));
                let committed = root.commit();

                let node = committed.get_value_at_path("/a".to_owned());

                let value = node.value;
                assert!((value)(0_i32).await.unwrap() == 1);
            });
    }

    #[test]
    fn it_should_return_the_value_at_a_wildcard_path() {
        async fn f1(a: i32, _b: NextFn<i32>) -> Result<i32, ThrusterError<i32>> {
            Ok(a + 1)
        }

        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                let mut root: Node<i32> = Node::default();

                root.add_value_at_path("/a/*/c", MiddlewareTuple::A(pinbox!(i32, f1)));
                let committed = root.commit();

                let node = committed.get_value_at_path("/a/b/c".to_owned());

                let value = node.value;
                assert!((value)(0_i32).await.unwrap() == 1);
            });
    }

    #[test]
    fn it_should_return_the_value_ending_with_a_wildcard_path() {
        async fn f1(a: i32, _b: NextFn<i32>) -> Result<i32, ThrusterError<i32>> {
            Ok(a - 1)
        }

        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                let mut root_a: Node<i32> = Node::default();
                let root_b: Node<i32> = Node::default();

                root_a.add_node_at_path("a/b/d", root_b);
                root_a.add_value_at_path("a/b/*", MiddlewareTuple::A(pinbox!(i32, f1)));

                let committed = root_a.commit();

                let node = committed.get_value_at_path("/a/b/d".to_owned());
                let value = node.value;
                assert_eq!((value)(0_i32).await.unwrap(), -1);
            });
    }

    #[test]
    fn it_should_return_the_value_at_a_paramaterized_path() {
        async fn f1(a: i32, _b: NextFn<i32>) -> Result<i32, ThrusterError<i32>> {
            Ok(a + 1)
        }

        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                let mut root: Node<i32> = Node::default();

                root.add_value_at_path("/a/:v/c", MiddlewareTuple::A(pinbox!(i32, f1)));
                let committed = root.commit();

                let node = committed.get_value_at_path("/a/b/c".to_owned());

                let value = node.value;
                assert!((value)(0_i32).await.unwrap() == 1);
            });
    }

    #[test]
    fn it_should_return_the_value_at_a_path_with_query_params() {
        async fn f1(a: i32, _b: NextFn<i32>) -> Result<i32, ThrusterError<i32>> {
            Ok(a + 1)
        }

        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                let mut root: Node<i32> = Node::default();

                root.add_value_at_path("/a", MiddlewareTuple::A(pinbox!(i32, f1)));
                let committed = root.commit();

                let node = committed.get_value_at_path("/a?q=2".to_owned());

                let value = node.value;
                assert!((value)(0_i32).await.unwrap() == 1);
            });
    }

    #[test]
    fn it_should_prefer_specificity_to_genericity() {
        async fn f1(a: i32, _b: NextFn<i32>) -> Result<i32, ThrusterError<i32>> {
            Ok(a + 1)
        }
        async fn f2(a: i32, _b: NextFn<i32>) -> Result<i32, ThrusterError<i32>> {
            Ok(a + 2)
        }

        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                let mut root: Node<i32> = Node::default();

                root.add_value_at_path("/a/b/:c", MiddlewareTuple::A(pinbox!(i32, f1)));
                root.add_value_at_path("/a/:d/e", MiddlewareTuple::A(pinbox!(i32, f2)));

                let committed = root.commit();

                let node = committed.get_value_at_path("/a/b/c".to_owned());

                let value = node.value;
                assert!((value)(0_i32).await.unwrap() == 1);
            });
    }

    #[test]
    fn it_should_prefer_long_matches_to_short() {
        async fn f1(a: i32, _b: NextFn<i32>) -> Result<i32, ThrusterError<i32>> {
            Ok(a + 1)
        }
        async fn f2(a: i32, _b: NextFn<i32>) -> Result<i32, ThrusterError<i32>> {
            Ok(a + 2)
        }

        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                let mut root: Node<i32> = Node::default();

                root.add_value_at_path("/a/b/c", MiddlewareTuple::A(pinbox!(i32, f1)));
                root.add_value_at_path("/a", MiddlewareTuple::A(pinbox!(i32, f2)));

                let committed = root.commit();

                let node = committed.get_value_at_path("/a/b/c".to_owned());

                let value = node.value;
                assert!((value)(0_i32).await.unwrap() == 1);
            });
    }

    #[test]
    fn it_should_return_the_param_for_a_route() {
        async fn f1(a: i32, _b: NextFn<i32>) -> Result<i32, ThrusterError<i32>> {
            Ok(a + 1)
        }

        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                let mut root: Node<i32> = Node::default();

                root.add_value_at_path("/a/:b/c", MiddlewareTuple::A(pinbox!(i32, f1)));

                let committed = root.commit();

                let node = committed.get_value_at_path("/a/b/c".to_owned());

                assert!(node.params.get("b").unwrap().param == "b");
            });
    }

    #[test]
    fn it_should_be_able_to_match_multiple_params_in_a_route() {
        async fn f1(a: i32, _b: NextFn<i32>) -> Result<i32, ThrusterError<i32>> {
            Ok(a + 1)
        }

        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                let mut root: Node<i32> = Node::default();

                root.add_value_at_path("/a/:b/:c", MiddlewareTuple::A(pinbox!(i32, f1)));

                let committed = root.commit();

                let node = committed.get_value_at_path("/a/b/c".to_owned());

                assert!(node.params.get("b").unwrap().param == "b");
                assert!(node.params.get("c").unwrap().param == "c");
            });
    }

    #[test]
    fn it_should_be_able_to_mix_params_and_wildcards() {
        async fn f1(a: i32, _b: NextFn<i32>) -> Result<i32, ThrusterError<i32>> {
            Ok(a + 1)
        }

        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                let mut root: Node<i32> = Node::default();

                root.add_value_at_path("/a/*/:c", MiddlewareTuple::A(pinbox!(i32, f1)));

                let committed = root.commit();

                let node = committed.get_value_at_path("/a/b/c".to_owned());

                assert!(node.params.get("c").unwrap().param == "c");
            });
    }

    #[test]
    fn it_should_be_able_to_add_a_node() {
        async fn f1(a: i32, _b: NextFn<i32>) -> Result<i32, ThrusterError<i32>> {
            Ok(a + 1)
        }
        async fn f2(a: i32, _b: NextFn<i32>) -> Result<i32, ThrusterError<i32>> {
            Ok(a + 2)
        }
        let mut root_a: Node<i32> = Node::default();
        let mut root_b: Node<i32> = Node::default();

        root_a.add_value_at_path("/a/b/c", MiddlewareTuple::A(pinbox!(i32, f1)));
        root_b.add_value_at_path("/d", MiddlewareTuple::A(pinbox!(i32, f2)));
        root_a.add_node_at_path("/a/b", root_b);

        let committed = root_a.commit();

        let printed = committed.to_string();

        assert!(
            printed
                == format!(
                    "{}: false (uncommitted)
    a: false (uncommitted)
        b: false (uncommitted)
            c: false (committed)
            d: false (committed)",
                    ROOT_ROUTE_ID
                ),
            "it should add a node correctly."
        );
    }

    #[test]
    fn it_should_attempt_to_merge_common_nodes() {
        async fn f1(a: i32, _b: NextFn<i32>) -> Result<i32, ThrusterError<i32>> {
            Ok(a + 1)
        }
        async fn f2(a: i32, _b: NextFn<i32>) -> Result<i32, ThrusterError<i32>> {
            Ok(a + 2)
        }

        let mut root_a: Node<i32> = Node::default();
        let mut root_b: Node<i32> = Node::default();

        root_a.add_value_at_path("a/b/c", MiddlewareTuple::A(pinbox!(i32, f1)));
        root_b.add_value_at_path("c", MiddlewareTuple::A(pinbox!(i32, f2)));
        root_a.add_node_at_path("a/b", root_b);

        let printed = root_a.commit().to_string();

        assert!(
            printed
                == format!(
                    "{}: false (uncommitted)
    a: false (uncommitted)
        b: false (uncommitted)
            c: false (committed)",
                    ROOT_ROUTE_ID
                ),
            "it should correctly merge nodes."
        );
    }

    #[test]
    fn it_should_attempt_to_merge_common_nodes_with_wildcards() {
        async fn f1(a: i32, _b: NextFn<i32>) -> Result<i32, ThrusterError<i32>> {
            Ok(a + 1)
        }
        async fn f2(a: i32, _b: NextFn<i32>) -> Result<i32, ThrusterError<i32>> {
            Ok(a + 2)
        }

        let mut root_a: Node<i32> = Node::default();
        let mut root_b: Node<i32> = Node::default();

        root_a.add_value_at_path("a/b/c", MiddlewareTuple::A(pinbox!(i32, f1)));
        root_b.add_value_at_path("c/*/d", MiddlewareTuple::A(pinbox!(i32, f2)));
        root_a.add_node_at_path("a/b", root_b);

        let printed = root_a.commit().to_string();

        assert!(
            printed
                == format!(
                    "{}: false (uncommitted)
    a: false (uncommitted)
        b: false (uncommitted)
            c: false (committed)
                *: false (uncommitted)
                    d: false (committed)",
                    ROOT_ROUTE_ID
                ),
            "it should correctly merge nodes."
        );
    }

    #[test]
    fn it_should_be_able_to_push_middleware_down_the_tree() {
        async fn f1(a: i32, _b: NextFn<i32>) -> Result<i32, ThrusterError<i32>> {
            Ok(a + 1)
        }

        async fn f2(a: i32, b: NextFn<i32>) -> Result<i32, ThrusterError<i32>> {
            Ok(b(a).await.unwrap() + 2)
        }

        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                let mut root: Node<i32> = Node::default();

                root.add_value_at_path("/a/b", MiddlewareTuple::A(pinbox!(i32, f1)));
                root.add_non_leaf_value_at_path("/", MiddlewareTuple::A(pinbox!(i32, f2)));
                let committed = root.commit();

                let node = committed.get_value_at_path("/a/b".to_owned());

                let value = node.value;
                let result = (value)(0_i32).await.unwrap();

                assert!(result == 3);
            });
    }

    #[test]
    fn it_should_be_able_to_enumerate() {
        async fn f1(a: i32, _b: NextFn<i32>) -> Result<i32, ThrusterError<i32>> {
            Ok(a + 1)
        }

        let mut root: Node<i32> = Node::default();

        root.add_value_at_path("a/b/c", MiddlewareTuple::A(pinbox!(i32, f1)));
        root.add_value_at_path("a/b/d", MiddlewareTuple::A(pinbox!(i32, f1)));
        root.add_value_at_path("e/f", MiddlewareTuple::A(pinbox!(i32, f1)));

        let enumerated = root
            .commit()
            .enumerate("")
            .into_iter()
            .filter(|v| v.1.is_some())
            .collect::<Vec<(
                String,
                Option<MiddlewareTuple<i32>>,
                Option<MiddlewareTuple<i32>>,
                bool,
            )>>();

        assert!(
            enumerated.len() == 3,
            "it should have three enumerated nodes"
        );

        assert!(
            enumerated.get(0).unwrap().0 == format!("/{}/a/b/c", ROOT_ROUTE_ID),
            "it should have a/b/c"
        );
        assert!(
            enumerated.get(1).unwrap().0 == format!("/{}/a/b/d", ROOT_ROUTE_ID),
            "it should have a/b/d"
        );
        assert!(
            enumerated.get(2).unwrap().0 == format!("/{}/e/f", ROOT_ROUTE_ID),
            "it should have e/f"
        );
    }

    #[test]
    fn it_should_respect_strict_mode() {
        async fn f1(a: i32, _b: NextFn<i32>) -> Result<i32, ThrusterError<i32>> {
            Ok(a + 1)
        }

        async fn f2(a: i32, _b: NextFn<i32>) -> Result<i32, ThrusterError<i32>> {
            Ok(a - 1)
        }

        let _ = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                let mut root: Node<i32> = Node::default();
                root.strict_mode = true;

                root.add_value_at_path("a", MiddlewareTuple::A(pinbox!(i32, f1)));
                root.add_value_at_path("a/", MiddlewareTuple::A(pinbox!(i32, f2)));
                let committed = root.commit();

                assert!(
                    (committed.get_value_at_path("/a".to_owned()).value)(0_i32)
                        .await
                        .unwrap()
                        == 1
                );
                assert!(
                    (committed.get_value_at_path("/a/".to_owned()).value)(0_i32)
                        .await
                        .unwrap()
                        == -1
                );
            });
    }
}
