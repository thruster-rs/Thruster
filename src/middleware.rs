use context::Context;

pub type Middleware = fn(Context) -> Context;
