pub use thruster_core_async_await::errors::ThrusterError;
use crate::context::Context;

pub trait Error<C> {
  fn build_context(self) -> C;
}

impl<C: Context> Error<C> for ThrusterError<C> {
  fn build_context(self) -> C {
    let mut context = self.context;

    context.set_body(format!("{{\"message\": \"{}\",\"success\":false}}", self.message).as_bytes().to_vec());
    context
  }
}
