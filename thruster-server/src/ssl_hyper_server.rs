use std::net::ToSocketAddrs;

use hyper::server::conn::Http;
use hyper::{Body, Response, Request, Server};
use hyper::service::{make_service_fn, service_fn};
use std::sync::Arc;

use thruster_app::app::App;
use thruster_core::context::Context;
use thruster_context::basic_hyper_context::HyperRequest;
use native_tls;
use native_tls::Identity;
use tokio::net::{TcpStream, TcpListener};
use tokio::prelude::*;
use std::io;

use crate::thruster_server::ThrusterServer;

pub struct SSLHyperServer<T: 'static + Context + Send> {
  app: App<HyperRequest, T>,
  cert: Option<Vec<u8>>,
  cert_pass: &'static str,
}

impl<T: 'static + Context + Send> SSLHyperServer<T> {
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

impl<T: Context<Response = Response<Body>> + Send> ThrusterServer for SSLHyperServer<T> {
  type Context = T;
  type Response = Response<Body>;
  type Request = HyperRequest;

  fn new(app: App<Self::Request, T>) -> Self {
    SSLHyperServer {
      app,
      cert: None,
      cert_pass: "",
    }
  }

  fn start(self, host: &str, port: u16) {
    let arc_app = Arc::new(self.app);
    let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();
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

    let rt = tokio::runtime::Runtime::new().unwrap();
    let _ = rt.block_on(async {
      // A this point, we should break down the service fn into
      // whatever the lowest point to get the stream is.
      let service = make_service_fn(|_| {
        let app = arc_app.clone();

        async {
          Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
            let matched = app.resolve_from_method_and_path(
              &req.method().to_string(),
              &req.uri().to_string()
            );
            let req = HyperRequest::new(req);
            app.resolve(req, matched)
          }))
        }
      });

      // let srv = TcpListener::bind(&addr).await.unwrap();
      // let http_proto = Http::new();
      // let https_service = http_proto
      //   .serve_incoming(
      //     async {
      //       let cloned_tls_acceptor = arc_acceptor.clone();
      //       let socket = srv.incoming();
      //       cloned_tls_acceptor
      //         .accept(socket)
      //         .await
      //         .unwrap();
      //     },
      //     service
      //   );

      // https_service.await?;
      let server = Server::bind(&addr)
          .serve(service);

      server.await?;

      Ok::<_, hyper::Error>(())
    });
  }
}
