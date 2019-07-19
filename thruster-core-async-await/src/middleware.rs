use std::boxed::Box;
use futures::future::Future;
use std::pin::Pin;
#[cfg(feature = "thruster_error_handling")]
use crate::errors::ThrusterError;

#[cfg(not(feature = "thruster_error_handling"))]
pub type MiddlewareResult<C> = C;
#[cfg(not(feature = "thruster_error_handling"))]
pub type MiddlewareReturnValue<T> = Pin<Box<dyn Future<Output=T> + Send>>;
#[cfg(not(feature = "thruster_error_handling"))]
pub type MiddlewareNext<C> = Box<dyn Fn(C) -> Pin<Box<dyn Future<Output=C> + Send>> + Send + Sync>;
#[cfg(not(feature = "thruster_error_handling"))]
type MiddlewareFn<C> = fn(C, MiddlewareNext<C>) -> Pin<Box<dyn Future<Output=C> + Send>>;

#[cfg(feature = "thruster_error_handling")]
pub type MiddlewareResult<C> = Result<C, ThrusterError<C>>;
#[cfg(feature = "thruster_error_handling")]
pub type MiddlewareReturnValue<C> = Pin<Box<dyn Future<Output=MiddlewareResult<C>> + Send>>;
#[cfg(feature = "thruster_error_handling")]
pub type MiddlewareNext<C> = Box<dyn Fn(C) -> Pin<Box<dyn Future<Output=MiddlewareResult<C>> + Send>> + Send + Sync>;
#[cfg(feature = "thruster_error_handling")]
type MiddlewareFn<C> = fn(C, MiddlewareNext<C>) -> Pin<Box<dyn Future<Output=MiddlewareResult<C>> + Send>>;

pub struct Middleware<C: 'static> {
  pub middleware: &'static [
    MiddlewareFn<C>
  ]
}

fn chained_run<C: 'static>(i: usize, j: usize, nodes: Vec<&'static Middleware<C>>) -> MiddlewareNext<C> {
  Box::new(move |ctx| {
    match nodes.get(i) {
      Some(n) => {
        match n.middleware.get(j) {
          Some(m) => m(ctx, chained_run(i, j + 1, nodes.clone())),
          None => chained_run(i + 1, 0, nodes.clone())(ctx),
        }
      },
      None => panic!("Chain ran into end of cycle")
    }
  })
}

pub struct Chain<C: 'static> {
  pub nodes: Vec<&'static Middleware<C>>,
  built: MiddlewareNext<C>
}

impl<C: 'static> Chain<C> {
  pub fn new(nodes: Vec<&'static Middleware<C>>) -> Chain<C> {
    Chain {
      nodes,
      built: Box::new(|_| panic!("Tried to run an unbuilt chain!"))
    }
  }

  fn chained_run(&self, i: usize, j: usize) -> Box<dyn Fn(C) -> Pin<Box<dyn Future<Output=MiddlewareResult<C>> + Send>> + Send + Sync> {
    chained_run(i, j, self.nodes.clone())
  }

  fn build(&mut self) {
    self.built = self.chained_run(0, 0);
  }

  fn run(&self, context: C) -> Pin<Box<dyn Future<Output=MiddlewareResult<C>> + Send>> {
    (self.built)(context)
  }
}

impl<C: 'static> Clone for Chain<C> {
  fn clone(&self) -> Self {
    let mut chain = Chain::new(self.nodes.clone());
    chain.build();
    chain
  }
}

///
/// The MiddlewareChain is used to wrap a series of middleware functions in such a way that the tail can
/// be accessed and modified later on. This allows Thruster to properly compose pieces of middleware
/// into a single long chain rather than relying on disperate parts.
///
pub struct MiddlewareChain<T: 'static> {
  pub chain: Chain<T>,
  pub assigned: bool
}

impl<T: 'static> MiddlewareChain<T> {
  ///
  /// Creates a new, blank (i.e. will panic if run,) MiddlewareChain
  ///
  pub fn new() -> Self {
    MiddlewareChain {
      chain: Chain::new(vec![]),
      assigned: false
    }
  }

  ///
  /// Assign a runnable function to this middleware chain
  ///
  pub fn assign(&mut self, chain: Chain<T>) {
    self.chain = chain;
    self.assigned = true;
  }

  pub fn assign_legacy(&mut self, chain: Chain<T>) {
    self.assign(chain);
  }

  ///
  /// Run the middleware chain once
  ///
  #[cfg(not(feature = "thruster_error_handling"))]
  pub fn run(&self, context: T) -> Pin<Box<dyn Future<Output=T> + Send>> {
    self.chain.run(context)
  }

  #[cfg(feature = "thruster_error_handling")]
  pub fn run(&self, context: T) -> Pin<Box<dyn Future<Output=MiddlewareResult<T>> + Send>> {
    self.chain.run(context)
  }

  ///
  /// Concatenate two middleware chains. This will make this chains tail point
  /// to the next chain. That means that calling `next` in the final piece of
  /// this chain will invoke the next chain rather than an "End of chain" panic
  ///
  pub fn chain(&mut self, mut chain: MiddlewareChain<T>) {
    self.chain.nodes.append(&mut chain.chain.nodes);
    self.assigned = self.assigned || chain.is_assigned();
  }

  ///
  /// Tells if the chain has been assigned OR whether it is unassigned but has
  /// an assigned tail. If is only chained but has no assigned runnable, then
  /// this chain acts as a passthrough to the next one.
  ///
  pub fn is_assigned(&self) -> bool {
    self.assigned
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
      chain: self.chain.clone(),
      assigned: self.assigned,
    }
  }
}
