use std::net::ToSocketAddrs;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::sync::Arc;
use tokio;

use thruster_app::app::App;
use thruster_context::basic_hyper_context::HyperRequest;
use thruster_core::context::Context;

use crate::thruster_server::ThrusterServer;

pub struct HyperServer<T: 'static + Context + Send> {
    app: App<HyperRequest, T>,
}

impl<T: 'static + Context + Send> HyperServer<T> {}

impl<T: Context<Response = Response<Body>> + Send> ThrusterServer for HyperServer<T> {
    type Context = T;
    type Response = Response<Body>;
    type Request = HyperRequest;

    fn new(app: App<Self::Request, T>) -> Self {
        HyperServer { app }
    }

    fn start(mut self, host: &str, port: u16) {
        self.app._route_parser.optimize();

        let arc_app = Arc::new(self.app);
        let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();

        let mut rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let service = make_service_fn(|_| {
                let app = arc_app.clone();

                async {
                    Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                        let matched = app.resolve_from_method_and_path(
                            &req.method().to_string(),
                            &req.uri().to_string(),
                        );

                        let req = HyperRequest::new(req);
                        app.resolve(req, matched)
                    }))
                }
            });

            let server = Server::bind(&addr).serve(service);

            server.await?;

            Ok::<_, hyper::Error>(())
        })
        .expect("hyper server failed");
    }
}
