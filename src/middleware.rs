use context::Context;
use std::vec::Vec;
use std::cell::Cell;

pub type Middleware = fn(Context, chain: &MiddlewareChain) -> Context;

pub struct MiddlewareChain {
  _chain_index: Cell<usize>,
  pub middleware: Vec<Middleware>
}

impl MiddlewareChain {
  pub fn new(middleware: Vec<Middleware>) -> MiddlewareChain {
    MiddlewareChain {
      middleware: middleware,
      _chain_index: Cell::new(0)
    }
  }

  pub fn next(&self, context: Context) -> Context {
    let next_middleware = self.middleware.get(self._chain_index.get());
    self._chain_index.set(self._chain_index.get() + 1);

    match next_middleware {
      Some(middleware) => middleware(context, self),
      None => context
    }
  }
}
