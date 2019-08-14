use std::net::ToSocketAddrs;

use futures_legacy::future;
use tokio;
use tokio::net::{TcpStream, TcpListener};
use tokio::prelude::*;
use tokio::codec::Framed;
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
  cert: Option<Vec<u8>>,
  cert_pass: &'static str,
}

impl<T: 'static + Context<Response = Response> + Send> SSLServer<T> {
  ///
  /// Sets the cert on the server
  ///
  pub fn cert(&mut self, cert: Vec<u8>) {
    self.cert = Some(cert);
  }

  pub fn cert_pass(&mut self, cert_pass: &'static str) {
    self.cert_pass = cert_pass;
  }
}

impl<T: Context<Response = Response> + Send> ThrusterServer for SSLServer<T> {
  type Context = T;
  type Response = Response;
  type Request = Request;

  fn new(app: App<Self::Request, T>) -> Self {
    SSLServer {
      app,
      cert: None,
      cert_pass: ""
    }
  }

  ///
  /// Alias for start_work_stealing_optimized
  ///
  fn start(mut self, host: &str, port: u16) {
    if self.cert.is_none() {
      panic!("Cert is required to be set via SSLServer::cert() before starting the server");
    }

    let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();

    self.app._route_parser.optimize();

    let listener = TcpListener::bind(&addr).unwrap();
    let arc_app = Arc::new(self.app);

    // TODO(trezm): Add an argument here for the tls_acceptor to be passed in.
    fn process<T: Context<Response = Response> + Send>(app: Arc<App<Request, T>>, tls_acceptor: Arc<tokio_tls::TlsAcceptor>, socket: TcpStream) {
      let tls_accept = tls_acceptor
        .accept(socket)
        .and_then(move |tls| {
          let framed = Framed::new(tls, Http);
          let (tx, rx) = framed.split();

          let task = tx.send_all(rx.and_then(move |request: Request| {
                let matched = app.resolve_from_method_and_path(request.method(), request.path());
                app.resolve(request, matched)
              }))
              .then(|_| {
                future::ok(())
              });

          tokio::spawn(task);

          Ok(())
        })
        .map_err(|err| {
          println!("TLS accept error: {:?}", err);
        });
      tokio::spawn(tls_accept);
    };

    let cert = self.cert.unwrap().clone();
    let cert_pass = self.cert_pass;
    let cert = Identity::from_pkcs12(&cert, cert_pass)
      .expect("Could not decrypt p12 file");
    let tls_acceptor =
        tokio_tls::TlsAcceptor::from(
          native_tls::TlsAcceptor::builder(cert)
            .build()
            .expect("Could not create TLS acceptor.")
          );
    let arc_acceptor = Arc::new(tls_acceptor);

    let server = listener.incoming()
        .map_err(|e| println!("error = {:?}", e))
        .for_each(move |socket| {
          let _ = socket.set_nodelay(true);
          process(arc_app.clone(), arc_acceptor.clone(), socket);
          Ok(())
        });


    tokio::run(server);
  }
}
