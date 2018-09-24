use std::collections::HashMap;

use context::Context;
use middleware::{MiddlewareChain, MiddlewareReturnValue};

pub trait HasQueryParams {
  fn set_query_params(&mut self, query_params: HashMap<String, String>);
  fn route(&self) -> &str;
}

pub fn query_params<T: 'static + Context + HasQueryParams + Send>(mut context: T, chain: &MiddlewareChain<T>) -> MiddlewareReturnValue<T> {
  let mut query_param_hash = HashMap::new();

  {
    let route: &str = &context.route();

    let mut iter = route.split("?");
    // Get rid of first bit (the non-query string part)
    let _ = iter.next().unwrap();
    match iter.next() {
      Some(query_string) => {
        for query_piece in query_string.split("&") {
          let mut query_iterator = query_piece.split("=");
          let key = query_iterator.next().unwrap().to_owned();
          match query_iterator.next() {
            Some(val) => query_param_hash.insert(key, val.to_owned()),
            None => query_param_hash.insert(key, "true".to_owned())
          };
        }
      },
      None => ()
    };
  }

  context.set_query_params(query_param_hash);

  chain.next(context)
}
