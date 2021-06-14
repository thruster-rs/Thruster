use std::{fs, path::Path};

use async_trait::async_trait;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use hyperlocal::UnixServerExt;
use std::sync::Arc;

use crate::app::App;
use crate::context::basic_hyper_context::HyperRequest;
use crate::core::context::Context;
use crate::server::ThrusterServer;

pub struct UnixHyperServer<T: 'static + Context + Send, S: Send> {
    app: App<HyperRequest, T, S>,
}

#[async_trait]
impl<T: Context<Response = Response<Body>> + Send, S: 'static + Send + Sync> ThrusterServer
    for UnixHyperServer<T, S>
{
    type Context = T;
    type Response = Response<Body>;
    type Request = HyperRequest;
    type State = S;

    fn new(app: App<Self::Request, T, Self::State>) -> Self {
        UnixHyperServer { app }
    }

    async fn build(mut self, socket_path: &str, _unused_port: u16) {
        self.app._route_parser.optimize();

        let arc_app = Arc::new(self.app);
        let path = Path::new(socket_path);

        if path.exists() {
            fs::remove_file(path).expect(&format!("Could not remove file: {}", socket_path));
        }

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

            Server::bind_unix(path).unwrap().serve(service).await?;

            Ok::<_, hyper::Error>(())
        }
        .await
        .expect("hyper server failed");
    }
}
