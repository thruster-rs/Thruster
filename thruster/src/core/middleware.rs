use crate::core::context::Context;
use crate::core::errors::ThrusterError;
use std::boxed::Box;
use std::future::Future;
use std::pin::Pin;

pub type MiddlewareResult<C> = Result<C, ThrusterError<C>>;
pub type MiddlewareReturnValue<C> = Pin<Box<dyn Future<Output = MiddlewareResult<C>> + Send>>;
pub type MiddlewareNext<C> =
    Box<dyn Fn(C) -> Pin<Box<dyn Future<Output = MiddlewareResult<C>> + Send>> + Send + Sync>;
pub type MiddlewareFn<C> =
    fn(C, MiddlewareNext<C>) -> Pin<Box<dyn Future<Output = MiddlewareResult<C>> + Send>>;

pub trait MiddlewareTrait<C> {
    fn invoke(&self, context: C, next: MiddlewareNext<C>) -> Pin<Box<dyn Future<Output = MiddlewareResult<C>> + Send>>;
}

impl<C> MiddlewareTrait<C> for MiddlewareFn<C> {
    fn invoke(&self, context: C, next: MiddlewareNext<C>) -> Pin<Box<dyn Future<Output = MiddlewareResult<C>> + Send>> {
        (self)(context, next)
    }
}

pub struct Middleware<C: 'static> {
    pub middleware: &'static [Box<dyn MiddlewareTrait<C> + Send + Sync>],
}

fn chained_run<C: 'static + Context + Send>(
    i: usize,
    j: usize,
    nodes: Vec<&'static Middleware<C>>,
) -> MiddlewareNext<C> {
    Box::new(move |ctx| match nodes.get(i) {
        Some(n) => match n.middleware.get(j) {
            Some(m) => m.invoke(ctx, chained_run(i, j + 1, nodes.clone())),
            None => chained_run(i + 1, 0, nodes.clone())(ctx),
        },
        // TODO(trezm): Make this somehow default to a 404
        None => {
            warn!("Running through the middlewares, 'next' was called without another next in the chain. That's not great! Make sure you have a 404 set.

\tThe route in question is: '{}'
", ctx.route());
            Box::pin(async { Ok(ctx) })
        }
    })
}

pub struct Chain<C: 'static + Context + Send> {
    pub nodes: Vec<&'static Middleware<C>>,
    built: MiddlewareNext<C>,
}

impl<C: 'static + Context + Send> Chain<C> {
    pub fn new(nodes: Vec<&'static Middleware<C>>) -> Chain<C> {
        Chain {
            nodes,
            built: Box::new(|_| panic!("Tried to run an unbuilt chain!")),
        }
    }

    fn chained_run(&self, i: usize, j: usize) -> MiddlewareNext<C> {
        chained_run(i, j, self.nodes.clone())
    }

    fn build(&mut self) {
        self.built = self.chained_run(0, 0);
    }

    fn run(&self, context: C) -> Pin<Box<dyn Future<Output = MiddlewareResult<C>> + Send>> {
        (self.built)(context)
    }
}

impl<C: 'static + Context + Send> Clone for Chain<C> {
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
pub struct MiddlewareChain<T: 'static + Context + Send> {
    pub chain: Chain<T>,
    pub assigned: bool,
}

impl<T: 'static + Context + Send> MiddlewareChain<T> {
    ///
    /// Creates a new, blank (i.e. will panic if run,) MiddlewareChain
    ///
    pub fn new() -> Self {
        MiddlewareChain {
            chain: Chain::new(vec![]),
            assigned: false,
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

    pub fn run(&self, context: T) -> Pin<Box<dyn Future<Output = MiddlewareResult<T>> + Send>> {
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

impl<T: 'static + Context + Send> Default for MiddlewareChain<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: 'static + Context + Send> Clone for MiddlewareChain<T> {
    fn clone(&self) -> Self {
        MiddlewareChain {
            chain: self.chain.clone(),
            assigned: self.assigned,
        }
    }
}
