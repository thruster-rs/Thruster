use std::boxed::Box;
use std::sync::Arc;
use futures_legacy::Future;
use std::io;
use std::pin::Pin;

///
/// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=d4cabc61b99c9fdd8d3199b1c51c128f
///


pub type MiddlewareReturnValue<T> = Box<Future<Output=T> + Send>;
pub type Middleware<T, M> = fn(T, next: M) -> MiddlewareReturnValue<T>;
pub type Runnable<T> = Box<Fn(T, &Option<Box<MiddlewareChain<T>>>) -> MiddlewareReturnValue<T> + Send + Sync>;

pub type ChainableReturnValue<T> = Pin<Box<std::future::Future<Output=T>>>;
pub type Chainable<T, F: std::future::Future<Output=T>> = for<'a> fn(T, &Fn(T) -> F) -> (dyn std::future::Future<Output=T> + Send);
pub type Chainable2<T> = Box<Fn(T, Box<Fn(T) -> Pin<Box<std::future::Future<Output=T>>>>) -> (Pin<Box<std::future::Future<Output=T>>>) + 'static + Send + Sync>;

// pub type Chainable<T> = fn(T, &Fn(T) -> Pin<Box<std::future::Future<Output=T>>>) -> Pin<Box<std::future::Future<Output=T>>>;



///
/// The MiddlewareChain is used to wrap a series of middleware functions in such a way that the tail can
/// be accessed and modified later on. This allows Thruster to properly compose pieces of middleware
/// into a single long chain rather than relying on disperate parts.
///
pub struct MiddlewareChain<T: 'static> {
  pub chainable_old: Chainable<T, >,
  pub next: Option<Box<MiddlewareChain<T>>>,
  is_assigned: bool
}

fn unassigned_panic<'a, T: 'static>(ctx: T, _: &(Fn(T) -> Pin<Box<std::future::Future<Output=T>>>)) -> Pin<Box<std::future::Future<Output=T> + Send>> {
  panic!("Has not been assigned")
}

impl<T: 'static> MiddlewareChain<T> {
  ///
  /// Creates a new, blank (i.e. will panic if run,) MiddlewareChain
  ///
  pub fn new() -> Self {
    MiddlewareChain {
      chainable_old: unassigned_panic,
      is_assigned: false,
      next: None
    }
  }

  ///
  /// Assign a runnable function to this middleware chain
  ///
  pub fn assign(&mut self, chain: Chainable<T>) {
    self.chainable_old = chain;
    // self.chainable = Box::new(move |context, end| {
    //   (self.chainable)(
    //     context,
    //     Box::new(move |context_next| {
    //       (chain)(
    //         context_next,
    //         end
    //       )
    //     })
    //   )
    // });
    self.is_assigned = true;
  }

  pub fn assign_legacy(&mut self, chain: Runnable<T>) {
    panic!("unimplemented");
    // self.runnable = Arc::new(chain);
    // self.is_assigned = true;
  }

  ///
  /// Run the middleware chain once
  ///
  pub fn run<'a>(&self, context: T) -> Pin<Box<std::future::Future<Output=T> + 'static + Send>> {
    // panic!("unimplemented");
    (self.chainable_old)(context, &|context| {
      panic!("End of middleware chain was reached")
    })
    // (self.chainable_old)(context, &|context| {
    //   match &self.next {
    //     Some(ref next) => next.run(context),
    //     None => panic!("End of middleware chain was reached")
    //   }
    // })
  }

  ///
  /// Concatenate two middleware chains. This will make this chains tail point
  /// to the next chain. That means that calling `next` in the final piece of
  /// this chain will invoke the next chain rather than an "End of chain" panic
  ///
  pub fn chain(&mut self, chain: MiddlewareChain<T>) {
    match self.next {
      Some(ref mut existing_chain) => existing_chain.chain(chain),
      None => self.next = Some(Box::new(chain))
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
      match self.next {
        Some(ref next) => next.is_assigned(),
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
      chainable_old: self.chainable_old.clone(),
      is_assigned: self.is_assigned,
      next: self.next.clone()
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

    fn dummy(context: $ctx, next: impl Fn($ctx) -> MiddlewareReturnValue<$ctx>  + Send + Sync) -> MiddlewareReturnValue<$ctx> {
      next(context)
    }

    // chain.assign_legacy(Box::new(|context, chain| {
    //   middleware![@internal $ctx => $head,
    //     $ctx => dummy,
    //     |ctx| {
    //       match chain {
    //         Some(val) => val.run(ctx),
    //         None => panic!("End of chain")
    //       }
    //     }](context)
    //   }));

    chain
  }};
  [ $ctx:ty => $head:expr, $($tail_t:ty => $tail_e:expr),+ ] => {{
    use std::boxed::Box;
    use futures::future::Future;

    let mut chain = MiddlewareChain::new();

    fn dummy(context: $ctx, next: impl Fn($ctx) -> MiddlewareReturnValue<$ctx>  + Send + Sync) -> MiddlewareReturnValue<$ctx> {
      next(context)
    }

    // chain.assign_legacy(Box::new(|context, chain| {
    //   middleware![@internal $ctx => $head, $( $tail_t => $tail_e ),* ,
    //     $ctx => dummy,
    //     |ctx| {
    //       match chain {
    //         Some(val) => val.run(ctx),
    //         None => panic!("End of chain")
    //       }
    //     }](context)
    //   }));

    chain
  }};
}

// #[macro_export]
// macro_rules! async_middleware {
//   [ @internal $ctx:ty => $head:expr ] => (
//     $head(context, end)
//   );
//   [ @internal $ctx:ty => $head:expr, $tail_t:ty => $tail_e:expr,+ ] => (
//     $head(context, &|context| {
//       async_middleware![$( $tail_t:ty => $tail_e ),*]
//     })
//   );
//   [ $ctx:ty => $head:expr ] => {{
//     use std::boxed::Box;
//     use futures::future::Future;

//     let mut chain: MiddlewareChain<$ctx> = MiddlewareChain::new();

//     // fn chain_to_be_replaced(context: $ctx, end: &Fn($ctx) -> ChainableReturnValue<$ctx>) -> ChainableReturnValue<$ctx> {
//     //   async_middleware![@internal $head]
//     // }

//     chain
//   }};
//   [ $ctx:ty => $head:expr, $($tail_t:ty => $tail_e:expr),+ ] => {{
//     use std::boxed::Box;
//     use futures::future::Future;

//     let mut chain = MiddlewareChain::new();

//     fn chain_to_be_replaced(context: $ctx, end: &Fn($ctx) -> ChainableReturnValue<$ctx>) -> ChainableReturnValue<$ctx> {
//       async_middleware![@internal $ctx => $head, $( $tail_t => $tail_e ),* ]
//     }

//     chain
//   }};
// }
