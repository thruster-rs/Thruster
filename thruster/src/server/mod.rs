mod homegrown_server;
pub use homegrown_server::Server;

#[cfg(feature = "tls")]
pub mod ssl_server;

#[cfg(feature = "hyper_server")]
pub mod hyper_server;

#[cfg(feature = "unix_hyper_server")]
pub mod unix_hyper_server;

#[cfg(all(feature = "hyper_server", feature = "tls"))]
pub mod ssl_hyper_server;

#[cfg(feature = "actix_server")]
pub mod actix_server;

mod thruster_server;

pub use thruster_server::ThrusterServer;

#[cfg(feature = "tls")]
pub use crate::ssl_server::SSLServer;

#[cfg(feature = "hyper_server")]
pub use crate::hyper_server::HyperServer;

#[cfg(feature = "unix_hyper_server")]
pub use crate::unix_hyper_server::UnixHyperServer;

#[cfg(all(feature = "hyper_server", feature = "tls"))]
pub use crate::ssl_hyper_server::SSLHyperServer;

#[cfg(feature = "actix_server")]
pub use crate::actix_server::ActixServer;
