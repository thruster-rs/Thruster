use std::net::ToSocketAddrs;

use futures_legacy::Future;
use futures_preview::future::TryFutureExt;
use tokio;
use hyper::{Body, Response, Request, Server};
use hyper::service::{make_service_fn, service_fn};
use std::sync::Arc;

use thruster_app::app::App;
use thruster_core::context::Context;
use thruster_context::basic_hyper_context::HyperRequest;

use crate::thruster_server::ThrusterServer;

pub struct HyperServer<T: 'static + Context + Send> {
  app: App<HyperRequest, T>
}

impl<T: 'static + Context + Send> HyperServer<T> {
}

impl<T: Context<Response = Response<Body>> + Send> ThrusterServer for HyperServer<T> {
  type Context = T;
  type Response = Response<Body>;
  type Request = HyperRequest;

  fn new(app: App<Self::Request, T>) -> Self {
    HyperServer {
      app
    }
  }

  fn start(mut self, host: &str, port: u16) {
    let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();

    self.app._route_parser.optimize();

    let arc_app = Arc::new(self.app);
    let new_service = make_service_fn(move |_| {
      let app = arc_app.clone();

      Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
        let matched = app.resolve_from_method_and_path(
          &req.method().to_string(),
          &req.uri().to_string()
        );
        let req = HyperRequest::new(req);
        app.resolve(req, matched).compat()
      }))
    });

    let server = Server::bind(&addr)
      .serve(new_service)
      .map_err(|e| eprintln!("server error: {}", e));

    tokio::run(server);
  }
}
