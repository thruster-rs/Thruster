use std::net::ToSocketAddrs;

use hyper::server::conn::{AddrStream, Http};
use hyper::{Body, Response, Request, Server};
use hyper::service::{make_service_fn, service_fn};
use std::sync::Arc;
use std::error::Error;

use thruster_app::app::App;
use thruster_core::context::Context;
use thruster_context::basic_hyper_context::HyperRequest;
use native_tls;
use native_tls::Identity;
use tokio::net::{TcpStream, TcpListener};
use tokio::prelude::*;
use tokio;
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



    let rt = tokio::runtime::Runtime::new().unwrap();
    let _ = rt.block_on(async {
      // ssl_server.rs section
      let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();

      let listener = TcpListener::bind(&addr).await.unwrap();
      let mut incoming = listener.incoming();
      let arc_app = Arc::new(self.app);

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

      let service = make_service_fn(|_| {
        let cloned_app = arc_app.clone();

        async {
          Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
            let matched = cloned_app.resolve_from_method_and_path(
              &req.method().to_string(),
              &req.uri().to_string()
            );
            let req = HyperRequest::new(req);
            cloned_app.resolve(req, matched)
          }))
        }
      });

      while let Some(Ok(stream)) = incoming.next().await {
          let cloned_tls_acceptor = arc_acceptor.clone();
          tokio::spawn(async move {
            let tls = tls_acceptor.accept(stream);
            let http_proto = Http::new()
              .serve_incoming(tls, service);


            // if let Err(e) = process(cloned_app, cloned_tls_acceptor, stream).await {
            //     println!("failed to process connection; error = {}", e);
            // }
          });
      }

      // hyper_server.rs section
      // let service = make_service_fn(|_| {
      //   let app = arc_app.clone();

      //   async {
      //     Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
      //       let matched = app.resolve_from_method_and_path(
      //         &req.method().to_string(),
      //         &req.uri().to_string()
      //       );
      //       let req = HyperRequest::new(req);
      //       app.resolve(req, matched)
      //     }))
      //   }
      // });


      // let server = Server::bind(&addr)
      //     .serve(service);

      // server.await?;

      // Ok::<_, hyper::Error>(())
    });
  }
}
