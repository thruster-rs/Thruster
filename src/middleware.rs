use context::{Context};
use std::vec::Vec;
use std::cell::Cell;
use std::boxed::Box;
use futures::{future, Future};
use std::io;

// pub type Middleware<T: Context> = fn(T, chain: &MiddlewareChain<T>) -> T;
pub type MiddlewareReturnValue<T> = Box<Future<Item=T, Error=io::Error> + Send>;
pub type Middleware<T> = fn(T, chain: &MiddlewareChain<T>) -> MiddlewareReturnValue<T>;

pub struct MiddlewareChain<T: Context> {
  _chain_index: Cell<usize>,
  pub middleware: Vec<Middleware<T>>
}

impl<T: 'static + Context + Send> MiddlewareChain<T> {
  pub fn new(middleware: Vec<Middleware<T>>) -> MiddlewareChain<T> {
    MiddlewareChain {
      middleware: middleware,
      _chain_index: Cell::new(0)
    }
  }

  pub fn next(&self, context: T) -> MiddlewareReturnValue<T> {
    let next_middleware = self.middleware.get(self._chain_index.get());
    self._chain_index.set(self._chain_index.get() + 1);

    match next_middleware {
      Some(middleware) => middleware(context, self),
      None => Box::new(future::ok(context))
    }
  }
}
