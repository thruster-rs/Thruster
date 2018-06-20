use context::{Context};
use std::vec::Vec;
use std::cell::Cell;
use std::boxed::Box;
use futures::{future, Future};
use std::io;

/// `MiddlewareReturnValue`s are the values that Thruster expects middleware to return. It's
/// shorthand for a Future, where the Item is a Context associated with the app, and the
/// error is an io::Error.
pub type MiddlewareReturnValue<T> = Box<Future<Item=T, Error=io::Error> + Send>;

/// The `Middleware` type simply defines the signature of a thruster middleware function.
/// It takes a context of the type of the thruster app, followed by a MiddlewareChain.
pub type Middleware<T> = fn(T, chain: &MiddlewareChain<T>) -> MiddlewareReturnValue<T>;

/// The `MiddlewareChain` represents a chain of futures that is every piece of middleware
/// following the current one. If you wish to not continue down the chain, simply do not call
/// `chain.next`, otherwise, you can call it and wait for the return value of the future and
/// proceed with work accordingly.
pub struct MiddlewareChain<'a, T: 'a + Context> {
  _chain_index: Cell<usize>,
  pub middleware: &'a Vec<Middleware<T>>,
  pub not_found: &'a Vec<Middleware<T>>
}

impl<'a, T: 'static + Context + Send> MiddlewareChain<'a, T> {
  /// Create a new `MiddlewareChain` with a vector of middleware to be executed.
  pub fn new(middleware: &'a Vec<Middleware<T>>, not_found: &'a Vec<Middleware<T>>) -> MiddlewareChain<'a, T> {
    MiddlewareChain {
      middleware: middleware,
      _chain_index: Cell::new(0),
      not_found: not_found
    }
  }

  pub fn next(&self, context: T) -> MiddlewareReturnValue<T> {
    let next_middleware = self.middleware.get(self._chain_index.get())
      .or_else(|| self.not_found.get(self._chain_index.get() - self.middleware.len()));
    self._chain_index.set(self._chain_index.get() + 1);

    match next_middleware {
      Some(middleware) => middleware(context, self),
      None => Box::new(future::ok(context))
    }
  }
}
