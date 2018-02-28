use std::collections::HashMap;
use std::vec::Vec;

use util::{strip_leading_slash, strip_trailing_slash};
use middleware::{Middleware};
use context::Context;
use app::App;

pub struct RouteNode<T: 'static + Context> {
  pub has_param: bool,
  pub param_name: String,
  pub associated_middleware: Vec<Middleware<T>>
}

pub struct MatchedRoute<T: 'static + Context> {
  pub value: String,
  pub params: HashMap<String, String>,
  pub middleware: Vec<Middleware<T>>,
  pub sub_app: Option<App<T>>
}

pub struct RouteParser<T: 'static + Context> {
  pub _method_agnostic_middleware: HashMap<String, Vec<Middleware<T>>>,
  pub _wildcarded_method_agnostic_middleware: HashMap<String, Vec<Middleware<T>>>,
  pub _not_found_route: Vec<Middleware<T>>,
  pub middleware: HashMap<String, RouteNode<T>>,
  pub wildcarded_middleware: HashMap<String, Vec<Middleware<T>>>,
  pub sub_apps: HashMap<String, App<T>>,
}

impl<T: Context> RouteParser<T> {
  pub fn new() -> RouteParser<T> {
    let parser = RouteParser {
      _method_agnostic_middleware: HashMap::new(),
      _wildcarded_method_agnostic_middleware: HashMap::new(),
      _not_found_route: vec![],
      middleware: HashMap::new(),
      wildcarded_middleware: HashMap::new(),
      sub_apps: HashMap::new(),
    };

    parser
  }

  pub fn add_method_agnostic_middleware(&mut self, route: &str, middleware: Middleware<T>) {

    fn _add_full_route<T: 'static + Context>(parser: &mut RouteParser<T>, route: &str, middleware: Middleware<T>) {
      let updated_vector: Vec<Middleware<T>> = match parser.middleware.get(route) {
        Some(val) => {
          let mut _in_progress: Vec<Middleware<T>> = Vec::new();
          for func in &val.associated_middleware {
            _in_progress.push(*func);
          }

          _in_progress.push(middleware);
          _in_progress
        },
        None => vec![middleware]
      };

      parser.middleware.insert(route.to_owned(), RouteNode {
        has_param: false,
        param_name: "".to_owned(),
        associated_middleware: updated_vector
      });
    }

    let _route = strip_leading_slash(route).clone();

    _add_full_route(self, strip_trailing_slash(&templatify! { "/__DELETE__/"; _route ;"" }), middleware);
    _add_full_route(self, strip_trailing_slash(&templatify! { "/__GET__/"; _route ;"" }), middleware);
    _add_full_route(self, strip_trailing_slash(&templatify! { "/__POST__/"; _route ;"" }), middleware);
    _add_full_route(self, strip_trailing_slash(&templatify! { "/__PUT__/"; _route ;"" }), middleware);
    _add_full_route(self, strip_trailing_slash(&templatify! { "/__UPDATE__/"; _route ;"" }), middleware);
  }

  pub fn set_not_found(&mut self, middleware: Vec<Middleware<T>>) {
    self._not_found_route = middleware;
  }

  pub fn add_route(&mut self, route: &str, middleware: Vec<Middleware<T>>) {
    let _route = strip_leading_slash(route);

    let mut split_iterator = route.split("/");

    let first_piece = split_iterator.next();

    assert!(first_piece.is_some());

    let mut accumulator = first_piece.unwrap_or("").to_owned();

    for piece in split_iterator {
      let lead_piece = piece.chars().nth(0).unwrap_or(' ');
      if lead_piece == ':' {
        accumulator = templatify! { ""; &accumulator ;"/*" }; // Test vs. a join
        self.middleware.insert(accumulator.clone(), RouteNode {
          has_param: true,
          param_name: piece[1..].to_owned(),
          associated_middleware: Vec::new()
        });
      } else {
        accumulator = templatify! { ""; &accumulator ;"/"; piece ;"" }; // Test vs. a join
      }
    }

    let new_node = match self.middleware.get(&accumulator) {
      Some(existing) => {
        let mut new_associated_middleware = Vec::new();
        new_associated_middleware.append(&mut existing.associated_middleware.clone());
        new_associated_middleware.append(&mut middleware.clone());

        RouteNode {
          has_param: existing.has_param,
          param_name: existing.param_name.clone(),
          associated_middleware: new_associated_middleware
        }
      },
      None => {
        RouteNode {
          has_param: false,
          param_name: "".to_owned(),
          associated_middleware: middleware.clone()
        }
      }
    };

    self.middleware.insert(accumulator.clone(), new_node);
  }

  pub fn match_route(&self, route: &str) -> MatchedRoute<T> {
    let mut split_iterator = route.split("/");
    let mut params = HashMap::new();
    let mut composed_middleware = Vec::new();

    // split_iterator.next();
    let first_piece = split_iterator.next();

    assert!(first_piece.is_some());

    // let mut accumulator = templatify! { "/"; first_piece.unwrap_or("") ;"" };
    let mut accumulator = first_piece.unwrap_or("").to_owned();

    for piece in split_iterator {
      // Match the middleware for a wildcard
      match self.middleware.get(&templatify! { ""; &accumulator ;"/*" }) {
        Some(hit) => {
          for func in &hit.associated_middleware {
            composed_middleware.push(*func);
          }
          params.insert((&hit.param_name).to_owned(), piece.to_owned());
        },
        None => ()
      };

      accumulator = templatify! { ""; &accumulator ;"/"; piece ;"" }; // Test vs. a join

      // Match the middleware strictly
      match self.middleware.get(&accumulator) {
        Some(hit) => {
          for func in &hit.associated_middleware {
            composed_middleware.push(*func);
          }
        },
        None => ()
      };
    }

    if composed_middleware.len() == 0 {
      composed_middleware = self._not_found_route.clone();
    }

    MatchedRoute {
      value: route.to_owned(),
      params: params,
      middleware: composed_middleware,
      sub_app: None
    }
  }

  // pub fn match_route(&self, route: &str) -> MatchedRoute<T> {
  //   let route = strip_leading_slash(route);
  //   let route_iterator = route.split("/");
  //   let mut accumulator = "".to_owned();
  //   let mut current_node = &self._route_root_node;
  //   let mut middleware = Vec::new();
  //   let mut has_terminal_node = false;
  //   let mut params = HashMap::new();

  //   for piece in route_iterator {
  //     // let wildcarded_params = wildcardify_params(&accumulator);
  //     let wildcarded_params = accumulator.to_owned();
  //     let wildcarded_route = strip_leading_slash(&wildcarded_params);
  //     match self._method_agnostic_middleware.get(wildcarded_route) {
  //       Some(_middleware) => {
  //         for func in _middleware {
  //           middleware.push(*func);
  //         }
  //       },
  //       None => ()
  //     };

  //     for mut val in current_node.children.values() {
  //       if val.value == piece || val.has_params {
  //         println!("piece: {}", piece);

  //         if val.has_params {
  //           params.insert(val.value[1..].to_owned(), piece.to_owned());
  //           // accumulator = templatify! { ""; &accumulator ;"/*" };
  //         } else {
  //           // accumulator = templatify! { ""; &accumulator ;"/"; piece ;"" };
  //         }

  //         println!("accumulator: {}", accumulator);

  //         if val.is_terminal {
  //           // if val.has_params {
  //           //   accumulator = templatify! { ""; &accumulator ;"/*" };
  //           // } else {
  //           //   accumulator = templatify! { ""; &accumulator ;"/"; piece ;"" };
  //           // }

  //           let wildcarded_params = wildcardify_params(&accumulator);
  //           let wildcarded_route = strip_leading_slash(&wildcarded_params);
  //           match self.wildcarded_middleware.get(wildcarded_route) {
  //             Some(_middleware) => {
  //               for func in _middleware {
  //                 middleware.push(*func);
  //               }
  //               has_terminal_node = true;
  //               break;
  //             },
  //             None => ()
  //           };
  //         } else {
  //           current_node = val;
  //           break;
  //         }
  //       }
  //     }

  //     match piece.chars().nth(0) {
  //       Some(val) => {
  //         if val == ':' {
  //           accumulator = templatify! { ""; &accumulator ;"/*" };
  //         } else {
  //           accumulator = templatify! { ""; &accumulator ;"/"; piece ;"" };
  //         }
  //       },
  //       None => ()
  //     };
  //   }

  //   if !has_terminal_node {
  //     for func in &self._not_found_route {
  //       middleware.push(*func);
  //     }
  //   }

  //   MatchedRoute {
  //     value: route.to_owned(),
  //     params: params,
  //     middleware: middleware,
  //     sub_app: None
  //   }
  // }
}

#[cfg(test)]
mod tests {
  use super::RouteParser;
  use context::BasicContext;
  use middleware::MiddlewareChain;
  use futures::{future, Future};
  use std::io;
  use std::boxed::Box;

  // #[test]
  // fn it_should_should_be_able_to_generate_a_simple_parsed_route() {
  //   let mut route_parser = RouteParser::<BasicContext>::new();
  //   route_parser.add_route("1/2/3", Vec::new());

  //   let route_node = route_parser._route_root_node.children.get(&"1".to_owned()).unwrap();
  //   let second_child_node = route_node.children.get(&"2".to_owned()).unwrap();
  //   let third_child_node = second_child_node.children.get(&"3".to_owned()).unwrap();

  //   assert!(route_node.value == "1".to_owned());
  //   assert!(route_node.children.len() == 1);

  //   assert!(second_child_node.value == "2".to_owned());
  //   assert!(second_child_node.children.len() == 1);

  //   assert!(third_child_node.value == "3".to_owned());
  //   assert!(third_child_node.children.len() == 0);
  // }

  #[test]
  fn it_should_return_a_matched_path_for_a_good_route() {
    let mut route_parser = RouteParser::<BasicContext>::new();
    route_parser.add_route("1/2/3/4", Vec::new());

    assert!(route_parser.match_route("1/2/3/4").value == "1/2/3/4");
  }

  #[test]
  fn it_should_use_not_found_for_not_handled_routes() {
    let mut route_parser = RouteParser::<BasicContext>::new();
    route_parser.add_route("1/2/3/4", Vec::new());
    route_parser.set_not_found(Vec::new());

    assert!(route_parser.match_route("5").value == "5");
  }

  #[test]
  fn it_should_return_a_matched_path_for_a_good_route_with_multiple_similar_routes() {
    let mut route_parser = RouteParser::<BasicContext>::new();
    route_parser.add_route("1/2/3/6", Vec::new());
    route_parser.add_route("1/2/3/4/5", Vec::new());
    route_parser.add_route("1/2/3/4", Vec::new());

    assert!(route_parser.match_route("1/2/3/4").value == "1/2/3/4");
  }

  #[test]
  fn it_should_appropriately_define_route_params() {
    let mut route_parser = RouteParser::<BasicContext>::new();
    route_parser.add_route("1/:param/2", Vec::new());

    let matched = route_parser.match_route("1/somevar/2");

    assert!(matched.params.get("param").unwrap() == "somevar");
  }

  #[test]
  fn when_adding_a_route_it_should_return_a_struct_with_all_appropriate_middleware() {
    fn test_function(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> Box<Future<Item=BasicContext, Error=io::Error>> {
      Box::new(future::ok(context))
    }

    let mut route_parser = RouteParser::<BasicContext>::new();
    route_parser.add_route("1/2/3", vec![test_function]);

    let matched = route_parser.match_route("1/2/3");
    assert!(matched.middleware.len() == 1);
    // assert!(matched.middleware.get(0).unwrap() == &(test_function as Middleware));
  }

  #[test]
  fn when_adding_a_route_with_method_agnostic_middleware() {
    fn method_agnostic(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> Box<Future<Item=BasicContext, Error=io::Error>> {
      Box::new(future::ok(context))
    }

    fn test_function(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> Box<Future<Item=BasicContext, Error=io::Error>> {
      Box::new(future::ok(context))
    }

    let mut route_parser = RouteParser::<BasicContext>::new();
    route_parser.add_method_agnostic_middleware("/", method_agnostic);
    route_parser.add_route("/__GET__/1/2/3", vec![test_function]);

    let matched = route_parser.match_route("/__GET__/1/2/3");
    assert!(matched.middleware.len() == 2);
    // assert!(matched.middleware.get(0).unwrap() == method_agnostic);
    // assert!(matched.middleware.get(1).unwrap() == &(test_function as Middleware));
  }
}
