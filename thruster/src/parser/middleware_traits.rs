use crate::core::errors::ThrusterError;
use paste::paste;
use std::boxed::Box;
use std::future::Future;
use std::pin::Pin;
use thruster_proc::generate_tuples;

pub type NextFn<T> = Box<
    dyn FnOnce(
            T,
        )
            -> Pin<Box<dyn Future<Output = Result<T, ThrusterError<T>>> + 'static + Sync + Send>>
        + 'static
        + Sync
        + Send,
>;
pub type MiddlewareFnPointer<T> =
    fn(
        T,
        NextFn<T>,
    ) -> Pin<Box<dyn Future<Output = Result<T, ThrusterError<T>>> + 'static + Sync + Send>>;
pub trait MiddlewareFunc<T>:
    Fn(
    T,
    NextFn<T>,
) -> Pin<Box<dyn Future<Output = Result<T, ThrusterError<T>>> + 'static + Sync + Send>>
{
}

impl<A, T: Sync + Send> MiddlewareFunc<T> for A where
    A: Fn(
        T,
        NextFn<T>,
    )
        -> Pin<Box<dyn Future<Output = Result<T, ThrusterError<T>>> + 'static + Sync + Send>>
{
}

pub trait IntoMiddleware<T, A> {
    fn middleware(
        self,
    ) -> Box<
        dyn Fn(T) -> Pin<Box<dyn Future<Output = Result<T, ThrusterError<T>>> + Sync + Send>>
            + Send
            + Sync,
    >;
}

pub trait CombineMiddleware<T, AFuncs, BFuncs, CFuncs> {
    fn combine(self, funcs: BFuncs) -> CFuncs;
}

macro_rules! impl_into_middleware {
    ($($param: ident),*) => {
        #[allow(unused_parens)]
        impl<T: 'static + Sync + Send, $($param: MiddlewareFunc<T> + 'static + Send + Sync + Copy),*> IntoMiddleware<T, ($($param,)*)> for ($($param,)*) {
            fn middleware(self) -> Box<dyn Fn(T) -> Pin<Box<dyn Future<Output = Result<T, ThrusterError<T>>> + Sync + Send>> + Send + Sync> {
                #[allow(non_snake_case)]
                let ($($param,)*) = self;

                recurse!($($param),*)
            }
        }
    }
}

macro_rules! recurse {
    ($head:ident, $($tail:ident),*) => {
        Box::new(move |i| ($head)(i, recurse!($($tail),*)))
    };
    ($head:ident) => {
        Box::new(move |i| ($head)(i, Box::new(|i| Box::pin(async move { Ok(i) }))))
    };
}

#[macro_export]
macro_rules! pinbox {
    ($type:ty, $fn:ident) => {{
        fn __internal(
            a: $type,
            b: NextFn<$type>,
        ) -> Pin<Box<dyn Future<Output = Result<$type, ThrusterError<$type>>> + Sync + Send>>
        {
            Box::pin($fn(a, b))
        }

        __internal
    }};
}

macro_rules! expand_combine {
    ($($param:ident),*) => {
        paste! {
            expand_combine!(@mid $($param),*);
            expand_combine!(@lhs $($param),*|$([<$param:lower>]),*);
        }
    };
    (@mid $head:ident, $($param:ident),*) => {
        impl_into_middleware!($head, $($param),*);
        expand_combine!(@mid $($param),*);
    };
    (@mid $head:ident) => {
        impl_into_middleware!($head);
    };
    (@lhs $head:ident, $($param:ident),*|$($right:ident),*) => {
        paste! {
            expand_combine!(@rhs $head, $($param),*|$([<$right:lower>]),*);
            expand_combine!(@lhs $($param),*|$([<$right:lower>]),*);
        }
    };
    (@lhs $head:ident|$($right:ident),*) => {
        paste! {
            expand_combine!(@rhs $head|$([<$right:lower>]),*);
        }
    };
    (@rhs $($left:ident),*|$head:ident, $($param:ident),*) => {
        expand_combine!(@rhs $($left),*|$($param),*);
    };
    (@rhs $($left:ident),*|$head:ident) => {
    };
}

type M<T> = MiddlewareFnPointer<T>;
expand_combine!(P, O, N, M, L, K, J, I, H, G, F, E, D, C, B, A);
generate_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
