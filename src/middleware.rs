use context::Context;

pub type Middleware = fn(Context, fn() -> ()) -> ();
