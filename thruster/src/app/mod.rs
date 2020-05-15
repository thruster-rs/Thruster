mod thruster_app;

#[cfg(not(feature = "hyper_server"))]
pub mod testing_async;

#[cfg(feature = "hyper_server")]
mod testing_hyper_async;

#[cfg(feature = "hyper_server")]
pub mod testing_async {
  pub use super::testing_hyper_async::*;
}

pub use thruster_app::*;
