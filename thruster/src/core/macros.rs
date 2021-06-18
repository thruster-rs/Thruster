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

// #[macro_export]
// macro_rules! async_middleware {
//   ($ctx:ty, [$($x:expr),+]) => {{
//     use thruster::m;

//     m![$($x,)*]
//   }}
// }

#[macro_export]
macro_rules! async_middlewarez {
  (@internal $last:ident @internal_rest $head:ident, $($tail:ident),+) => {
      |v| $head(v, Some(Box::new(async_middlewarez!(@internal $last @internal_rest $($tail),*))))
  };
  (@internal $last:ident @internal_rest $head:ident) => {
      |v| $head(v, $last)
  };

  (@return_type $ctx:ty) => {
    Pin<
      Box<
          (dyn futures::Future<Output = std::result::Result<$ctx, ThrusterError<$ctx>>>
              + std::marker::Send
              + 'static),
      >,
    >
  };

  ($outter_ctx:ty:$inner_ctx:ty, $($middlewares:ident),+) => {{
      fn _internal(ctx: $outter_ctx, last: Option<Box<(dyn Middleware<$inner_ctx, $inner_ctx> + Send + 'static)>>) -> async_middlewarez!(@return_type $outter_ctx) {
        async_middlewarez!(@internal last @internal_rest $($middlewares),*)(ctx)
      }

      _internal
  }};

  ($ctx_type:ty, $($middlewares:ident),+) => {{
      fn _internal(ctx: $ctx_type, last: Option<Box<(dyn Middleware<$ctx_type, $ctx_type> + Send + 'static)>>) -> async_middlewarez!(@return_type $ctx_type) {
        async_middlewarez!(@internal last @internal_rest $($middlewares),*)(ctx)
      }

      _internal
  }};
}
