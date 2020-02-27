#[macro_export]
macro_rules! map_try {
  [ $expr:expr, $pat:pat => $mapper:expr ] => ({
    match $expr {
        Ok(val) => val,
        $pat => {
            let __e = $mapper;

            return Err(Into::into(__e));
        }
    }
  });
}

#[macro_export]
macro_rules! async_middleware {
  ($ctx:ty, [$($x:expr),+]) => {{
      use thruster::{Chain, Middleware, MiddlewareChain, MiddlewareNext, MiddlewareReturnValue};

      const __MIDDLEWARE_ARRAY: &'static [
          fn($ctx, MiddlewareNext<$ctx>) -> MiddlewareReturnValue<$ctx>
      ] = &[$( $x ),*];

      static __MIDDLEWARE: Middleware<$ctx> = Middleware {
          middleware: __MIDDLEWARE_ARRAY
      };

      let chain = Chain::new(vec![&__MIDDLEWARE]);

      MiddlewareChain {
          chain,
          assigned: true
      }
  }}
}
