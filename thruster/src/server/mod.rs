mod homegrown_server;
pub use homegrown_server::Server;

#[cfg(feature = "tls")]
pub mod ssl_server;

#[cfg(feature = "hyper_server")]
pub mod hyper_server;

#[cfg(all(feature = "hyper_server", feature = "tls"))]
pub mod ssl_hyper_server;

mod thruster_server;

pub use thruster_server::ThrusterServer;

#[cfg(feature = "tls")]
pub use crate::ssl_server::SSLServer;

#[cfg(feature = "hyper_server")]
pub use crate::hyper_server::HyperServer;

#[cfg(all(feature = "hyper_server", feature = "tls"))]
pub use crate::ssl_hyper_server::SSLHyperServer;
