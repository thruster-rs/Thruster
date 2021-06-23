use futures::FutureExt;
use hyper::service::make_service_fn;
use hyper::{Body, Response, Server};
use hyperlocal::UnixServerExt;
use std::sync::Arc;
use std::{fs, path::Path};
use tokio_util::sync::ReusableBoxFuture;

use crate::app::App;
use crate::context::basic_hyper_context::HyperRequest;
use crate::core::context::Context;
use crate::server::hyper_server::HyperService;
use crate::server::ThrusterServer;

pub struct UnixHyperServer<T: 'static + Context + Clone + Send + Sync, S: Send> {
    app: Arc<App<HyperRequest, T, S>>,
}

impl<T: Context<Response = Response<Body>> + Clone + Send + Sync, S: 'static + Send + Sync>
    ThrusterServer for UnixHyperServer<T, S>
{
    type Context = T;
    type Response = Response<Body>;
    type Request = HyperRequest;
    type State = S;

    fn new(mut app: App<Self::Request, T, Self::State>) -> Self {
        app = app.commit();

        UnixHyperServer { app: Arc::new(app) }
    }

    fn build(self, socket_path: &str, _unused_port: u16) -> ReusableBoxFuture<()> {
        let app = self.app;
        let path = Path::new(socket_path);

        if path.exists() {
            fs::remove_file(path)
                .unwrap_or_else(|_| panic!("Could not remove file: {}", socket_path));
        }

        let service = make_service_fn(move |_: &tokio::net::UnixStream| {
            let arc_app = app.clone();

            async move {
                Ok::<_, hyper::Error>(HyperService::<T, S> {
                    ip: None,
                    app: arc_app,
                })
            }
        });

        let listener_fut = Server::bind_unix(path)
            .unwrap()
            .serve(service)
            .map(|v| match v {
                Ok(_) => (),
                Err(e) => panic!("Hyper server error occurred: {:#?}", e),
            });

        ReusableBoxFuture::new(listener_fut)
    }
}
