use context::{Context};
use std::vec::Vec;
use std::cell::Cell;

pub type Middleware<T: Context> = fn(T, chain: &MiddlewareChain<T>) -> T;

pub struct MiddlewareChain<T: Context> {
  _chain_index: Cell<usize>,
  pub middleware: Vec<Middleware<T>>
}

impl<T: Context> MiddlewareChain<T> {
  pub fn new(middleware: Vec<Middleware<T>>) -> MiddlewareChain<T> {
    MiddlewareChain {
      middleware: middleware,
      _chain_index: Cell::new(0)
    }
  }

  pub fn next(&self, context: T) -> T {
    let next_middleware = self.middleware.get(self._chain_index.get());
    self._chain_index.set(self._chain_index.get() + 1);

    match next_middleware {
      Some(middleware) => middleware(context, self),
      None => context
    }
  }
}
