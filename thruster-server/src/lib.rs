pub mod thruster_server;

#[cfg(feature="thruster_tls")]
pub mod ssl_server;

#[cfg(not(feature="hyper_server"))]
pub mod server;

#[cfg(feature="hyper_server")]
pub mod hyper_server;

#[cfg(not(feature="hyper_server"))]
pub use crate::thruster_server::ThrusterServer;

#[cfg(not(feature="hyper_server"))]
pub use crate::ssl_server::SSLServer;

#[cfg(feature="hyper_server")]
pub use crate::hyper_server::HyperServer;
