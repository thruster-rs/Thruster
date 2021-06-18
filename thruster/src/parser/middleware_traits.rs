use crate::core::errors::ThrusterError;
use crate::ReusableBoxFuture;
use paste::paste;
use std::boxed::Box;
use thruster_proc::generate_tuples;

pub type NextFn<T> =
    Box<dyn FnOnce(T) -> ReusableBoxFuture<Result<T, ThrusterError<T>>> + 'static + Send>;
pub type MiddlewareFnPointer<T> =
    fn(T, NextFn<T>) -> ReusableBoxFuture<Result<T, ThrusterError<T>>>;
pub trait MiddlewareFunc<T>:
    Fn(T, NextFn<T>) -> ReusableBoxFuture<Result<T, ThrusterError<T>>> + Send + Sync
{
}

impl<A, T: Send> MiddlewareFunc<T> for A where
    A: Fn(T, NextFn<T>) -> ReusableBoxFuture<Result<T, ThrusterError<T>>> + Send + Sync
{
}

pub trait IntoMiddleware<T, A> {
    fn middleware(
        self,
    ) -> Box<dyn Fn(T) -> ReusableBoxFuture<Result<T, ThrusterError<T>>> + Send + Sync>;
}

pub trait CombineMiddleware<T, AFuncs, BFuncs, CFuncs> {
    fn combine(self, funcs: BFuncs) -> CFuncs;
}

macro_rules! impl_into_middleware {
    ($($param: ident),*) => {
        #[allow(unused_parens)]
        impl<T: 'static + Send, $($param: MiddlewareFunc<T> + 'static + Send + Copy),*> IntoMiddleware<T, ($($param,)*)> for ($($param,)*) {
            fn middleware(self) -> Box<dyn Fn(T) -> ReusableBoxFuture<Result<T, ThrusterError<T>>> + Send + Sync> {
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
        Box::new(move |i| ($head)(i, Box::new(|i| ReusableBoxFuture::new(async move { Ok(i) }))))
    };
}

#[macro_export]
macro_rules! pinbox {
    ($type:ty, $fn:ident) => {{
        fn __internal(
            a: $type,
            b: NextFn<$type>,
        ) -> ReusableBoxFuture<Result<$type, ThrusterError<$type>>> {
            ReusableBoxFuture::new($fn(a, b))
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
