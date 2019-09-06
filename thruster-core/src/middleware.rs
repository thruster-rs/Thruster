use std::boxed::Box;
use std::sync::Arc;
use std::future::Future;
use std::io;

pub type MiddlewareReturnValue<T> = Box<dyn Future<Output=Result<T, io::Error>> + Send>;
pub type Middleware<T, M> = fn(T, next: M) -> MiddlewareReturnValue<T>;
pub type Runnable<T> = Box<dyn Fn(T, &Option<Box<MiddlewareChain<T>>>) -> MiddlewareReturnValue<T> + Send + Sync>;

///
/// The MiddlewareChain is used to wrap a series of middleware functions in such a way that the tail can
/// be accessed and modified later on. This allows Thruster to properly compose pieces of middleware
/// into a single long chain rather than relying on disperate parts.
///
pub struct MiddlewareChain<T: 'static> {
  pub runnable: Arc<Runnable<T>>,
  is_assigned: bool,
  chained: Option<Box<MiddlewareChain<T>>>
}

impl<T: 'static> MiddlewareChain<T> {
  ///
  /// Creates a new, blank (i.e. will panic if run,) MiddlewareChain
  ///
  pub fn new() -> Self {
    MiddlewareChain {
      runnable: Arc::new(Box::new(|_: T, _: &Option<Box<MiddlewareChain<T>>>| panic!("Use of unassigned middleware chain, be sure to run `assign` first."))),
      is_assigned: false,
      chained: None
    }
  }

  ///
  /// Assign a runnable function to this middleware chain
  ///
  pub fn assign(&mut self, chain: Runnable<T>) {
    self.runnable = Arc::new(chain);
    self.is_assigned = true;
  }

  ///
  /// Run the middleware chain once
  ///
  pub fn run(&self, context: T) -> MiddlewareReturnValue<T> {
    if self.is_assigned {
      (self.runnable)(context, &self.chained)
    } else if let Some(ref chain) = self.chained {
      chain.run(context)
    } else {
      (self.runnable)(context, &self.chained)
    }
  }

  ///
  /// Concatenate two middleware chains. This will make this chains tail point
  /// to the next chain. That means that calling `next` in the final piece of
  /// this chain will invoke the next chain rather than an "End of chain" panic
  ///
  pub fn chain(&mut self, chain: MiddlewareChain<T>) {
    match self.chained {
      Some(ref mut existing_chain) => existing_chain.chain(chain),
      None => self.chained = Some(Box::new(chain))
    };
  }

  ///
  /// Tells if the chain has been assigned OR whether it is unassigned but has
  /// an assigned tail. If is only chained but has no assigned runnable, then
  /// this chain acts as a passthrough to the next one.
  ///
  pub fn is_assigned(&self) -> bool {
    if self.is_assigned {
      true
    } else {
      match self.chained {
        Some(ref chained) => chained.is_assigned(),
        None => false
      }
    }
  }
}

impl<T: 'static> Default for MiddlewareChain<T> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T: 'static> Clone for MiddlewareChain<T> {
  fn clone(&self) -> Self {
    MiddlewareChain {
      runnable: self.runnable.clone(),
      is_assigned: self.is_assigned,
      chained: self.chained.clone()
    }
  }
}


///
/// The middleware macro takes a series of functions whose contexts all have `into` implemented,
/// and then links them together to run in series. The result of this macro is a single `MiddlewareChain`.
///
#[macro_export]
macro_rules! middleware {
  [ @tailtype $ctx:ty => $head:expr ] => { $ctx };
  [ @tailtype $ctx:ty => $head:expr, $($tail_t:ty => $tail_e:expr),+ ] => { middleware![@tailtype $( $tail_t => $tail_e ),*] };
  [ @internal $ctx:ty => $head:expr, $ender:expr ] => (
    |context: $ctx| {
      Box::new(
        $head(
          context.into(), |ctx| {
            $ender(ctx.into())
          })
          .map(|ctx| ctx.into()))
    }
  );
  [ @internal $ctx:ty => $head:expr, $($tail_t:ty => $tail_e:expr),+ , $ender:expr ] => (
    |context: $ctx| {
      Box::new(
        $head(
          context.into(), |ctx: $ctx| {
            middleware![@internal $( $tail_t => $tail_e ),*, $ender](ctx.into())
          })
          .map(|ctx| ctx.into()))
    }
  );
  [ $ctx:ty => $head:expr ] => {{
    use std::boxed::Box;
    use futures::future::Future;

    let mut chain: MiddlewareChain<$ctx> = MiddlewareChain::new();

    fn dummy(context: $ctx, next: impl Fn($ctx) -> MiddlewareReturnValue<$ctx>  + Send) -> MiddlewareReturnValue<$ctx> {
      next(context)
    }

    chain.assign(Box::new(|context, chain| {
      middleware![@internal $ctx => $head,
        $ctx => dummy,
        |ctx| {
          match chain {
            Some(val) => val.run(ctx),
            None => panic!("End of chain")
          }
        }](context)
      }));

    chain
  }};
  [ $ctx:ty => $head:expr, $($tail_t:ty => $tail_e:expr),+ ] => {{
    use std::boxed::Box;
    use futures::future::Future;

    let mut chain = MiddlewareChain::new();

    fn dummy(context: $ctx, next: impl Fn($ctx) -> MiddlewareReturnValue<$ctx>  + Send) -> MiddlewareReturnValue<$ctx> {
      next(context)
    }

    chain.assign(Box::new(|context, chain| {
      middleware![@internal $ctx => $head, $( $tail_t => $tail_e ),* ,
        $ctx => dummy,
        |ctx| {
          match chain {
            Some(val) => val.run(ctx),
            None => panic!("End of chain")
          }
        }](context)
      }));

    chain
  }};
}
