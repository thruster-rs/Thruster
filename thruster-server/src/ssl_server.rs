use std::net::ToSocketAddrs;

use futures_legacy::future;
use tokio;
use tokio::net::{TcpStream, TcpListener};
use tokio::prelude::*;
use tokio::codec::Framed;
use num_cpus;
use net2::TcpBuilder;
#[cfg(not(windows))]
use net2::unix::UnixTcpBuilderExt;
use std::thread;
use native_tls;
use native_tls::Identity;

use std::sync::Arc;

use thruster_app::app::App;
use thruster_core::context::Context;
use thruster_core::http::Http;
use thruster_core::response::Response;
use thruster_core::request::Request;

use crate::thruster_server::ThrusterServer;

pub struct SSLServer<T: 'static + Context<Response = Response> + Send> {
  app: App<Request, T>,
  cert: Vec<u8>,
  cert_pass: &'static str,
}

impl<T: 'static + Context<Response = Response> + Send> SSLServer<T> {
  ///
  /// Starts the app with the default tokio runtime execution model
  ///
  pub fn start_work_stealing_optimized(mut self, cert: Vec<u8>, cert_pass: &'static str, host: &str, port: u16) {
    self.cert = cert;
    self.cert_pass = cert_pass;
    self.start(host, port);
  }
}

impl<T: Context<Response = Response> + Send> ThrusterServer for SSLServer<T> {
  type Context = T;
  type Response = Response;
  type Request = Request;

  fn new(app: App<Self::Request, T>) -> Self {
    SSLServer {
      app,
      cert: Vec::new(),
      cert_pass: ""
    }
  }

  ///
  /// Alias for start_work_stealing_optimized
  ///
  fn start(mut self, host: &str, port: u16) {
    let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();

    self.app._route_parser.optimize();

    let listener = TcpListener::bind(&addr).unwrap();
    let arc_app = Arc::new(self.app);

    let cert = Identity::from_pkcs12(&self.cert, self.cert_pass)
      .expect("Could not decrypt p12 file");
    let tls_acceptor =
        tokio_tls::TlsAcceptor::from(
          native_tls::TlsAcceptor::builder(cert)
            .build()
            .expect("Could not create TLS acceptor.")
          );

    let process = move |app, socket: TcpStream| {
      let tls_accept = tls_acceptor
        .accept(socket)
        .and_then(move |tls| {
          // let (tx, rx) = tls.split();

          // let task = tx.send_all(rx.and_then(move |request: Request| {
          //       let matched = app.resolve_from_method_and_path(request.method(), request.path());
          //       app.resolve(request, matched)
          //     }))
          //     .then(|_| {
          //       future::ok(())
          //     });

          // tokio::spawn(task);

          Ok::<(), native_tls::Error>(())
        });
      // tokio::spawn(tls_accept);
    };

    let server = listener.incoming()
        .map_err(|e| println!("error = {:?}", e))
        .for_each(move |socket| {
            let _ = socket.set_nodelay(true);
            process(arc_app.clone(), socket);
            Ok(())
        });

    tokio::run(server);
  }
}
