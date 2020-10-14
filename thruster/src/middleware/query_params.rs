use std::collections::HashMap;
use thruster_proc::middleware_fn;

use crate::core::context::Context;
use crate::core::{MiddlewareNext, MiddlewareResult};

pub trait HasQueryParams {
    fn set_query_params(&mut self, query_params: HashMap<String, String>);
}

#[middleware_fn(_internal)]
pub async fn query_params<T: 'static + Context + HasQueryParams + Send>(
    mut context: T,
    next: MiddlewareNext<T>,
) -> MiddlewareResult<T> {
    let mut query_param_hash = HashMap::new();

    {
        let route: &str = &context.route();

        let mut iter = route.split('?');
        // Get rid of first bit (the non-query string part)
        let _ = iter.next().unwrap();

        if let Some(query_string) = iter.next() {
            for query_piece in query_string.split('&') {
                let mut query_iterator = query_piece.split('=');
                let key = query_iterator.next().unwrap().to_owned();

                match query_iterator.next() {
                    Some(val) => query_param_hash.insert(key, val.to_owned()),
                    None => query_param_hash.insert(key, "true".to_owned()),
                };
            }
        }
    }

    context.set_query_params(query_param_hash);

    next(context).await
}
