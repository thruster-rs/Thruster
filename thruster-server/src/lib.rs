pub mod thruster_server;

#[cfg(feature="thruster_tls")]
pub mod ssl_server;

#[cfg(not(feature="hyper_server"))]
pub mod server;

#[cfg(feature="hyper_server")]
pub mod hyper_server;

#[cfg(feature="hyper_server")]
pub mod ssl_hyper_server;

#[cfg(not(feature="hyper_server"))]
pub use crate::thruster_server::ThrusterServer;

#[cfg(all(not(feature="hyper_server"), feature="thruster_tls"))]
pub use crate::ssl_server::SSLServer;

#[cfg(feature="hyper_server")]
pub use crate::hyper_server::HyperServer;

#[cfg(feature="hyper_server")]
pub use crate::ssl_hyper_server::SSLHyperServer;
