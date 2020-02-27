use std::net::ToSocketAddrs;

use async_trait::async_trait;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::sync::Arc;

use app::app::App;
use context::basic_hyper_context::HyperRequest;
use core::context::Context;

use crate::server::ThrusterServer;

pub struct HyperServer<T: 'static + Context + Send> {
    app: App<HyperRequest, T>,
}

impl<T: 'static + Context + Send> HyperServer<T> {}

#[async_trait]
impl<T: Context<Response = Response<Body>> + Send> ThrusterServer for HyperServer<T> {
    type Context = T;
    type Response = Response<Body>;
    type Request = HyperRequest;

    fn new(app: App<Self::Request, T>) -> Self {
        HyperServer { app }
    }

    async fn build(mut self, host: &str, port: u16) {
        self.app._route_parser.optimize();

        let arc_app = Arc::new(self.app);
        let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();

        async move {
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
        }
        .await
        .expect("hyper server failed");
    }
}
