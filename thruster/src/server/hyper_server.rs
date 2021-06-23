use crate::ReusableBoxFuture;
use futures::FutureExt;
use hyper::server::conn::Http;
use hyper::service::make_service_fn;
use hyper::service::Service;
use hyper::{Body, Request, Response};
use socket2::{Domain, Socket, Type};
use std::net::IpAddr;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::task::{Context as TaskContext, Poll};
use tokio_stream::wrappers::TcpListenerStream;

use crate::context::hyper_request::HyperRequest;
use crate::core::context::Context;
use crate::server::ThrusterServer;
use crate::{app::App, core::request::ThrusterRequest};

impl ThrusterRequest for HyperRequest {
    fn method(&self) -> &str {
        self.request.method().as_str()
    }

    fn path(&self) -> String {
        self.request.uri().to_string()
    }
}

pub struct HyperServer<T: 'static + Context + Clone + Send + Sync, S: 'static + Send> {
    app: App<HyperRequest, T, S>,
}

impl<T: Context<Response = Response<Body>> + Clone + Send + Sync, S: 'static + Send + Sync>
    HyperServer<T, S>
{
    async fn process(
        app: Arc<App<HyperRequest, T, S>>,
        addr: SocketAddr,
    ) -> Result<(), hyper::Error> {
        let listener = TcpListenerStream::new({
            let socket = Socket::new(Domain::IPV4, Type::STREAM, None).unwrap();

            let address = addr.into();
            socket.set_reuse_address(true).unwrap();
            #[cfg(unix)]
            socket.set_reuse_port(true).unwrap();
            socket.bind(&address).unwrap();
            socket.listen(1024).unwrap();
            socket.set_nonblocking(true).unwrap();
            let _ = socket.set_nodelay(true);

            let listener: std::net::TcpListener = socket.into();
            tokio::net::TcpListener::from_std(listener).unwrap()
        });

        let service = make_service_fn(|stream: &tokio::net::TcpStream| {
            let ip = stream.peer_addr().map(|v| v.ip()).ok();
            let arc_app = app.clone();

            async move { Ok::<_, hyper::Error>(HyperService::<T, S> { ip, app: arc_app }) }
        });

        let mut http = Http::new();
        http.http1_only(true);

        let server =
            hyper::server::Builder::new(hyper::server::accept::from_stream(listener), http)
                .serve(service);

        server.await?;

        Ok::<_, hyper::Error>(())
    }

    #[allow(dead_code)]
    pub async fn build_per_thread(self, host: &str, port: u16) {
        // self.app._route_parser.optimize();

        let arc_app = Arc::new(self.app);
        let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();

        for _ in 0..num_cpus::get() - 1 {
            let arc_app = arc_app.clone();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_io()
                    .build()
                    .unwrap();

                rt.spawn(async move {
                    Self::process(arc_app, addr)
                        .await
                        .expect("Unable to spawn hyper server thread.");
                });
            });
        }

        Self::process(arc_app, addr)
            .await
            .expect("Unable to spawn hyper server thread.");
    }
}

impl<T: Context<Response = Response<Body>> + Clone + Send + Sync, S: 'static + Send + Sync>
    ThrusterServer for HyperServer<T, S>
{
    type Context = T;
    type Response = Response<Body>;
    type Request = HyperRequest;
    type State = S;

    fn new(mut app: App<Self::Request, T, Self::State>) -> Self {
        app = app.commit();

        HyperServer { app }
    }

    fn build(self, host: &str, port: u16) -> ReusableBoxFuture<()> {
        // self.app._route_parser.optimize();

        let arc_app = Arc::new(self.app);

        let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();
        ReusableBoxFuture::new(Self::process(arc_app, addr).map(|_| ()))
        // .await
        // .expect("hyper server failed");
    }
}

pub(crate) struct HyperService<T: 'static + Context + Clone + Send + Sync, S: Send> {
    pub(crate) app: Arc<App<HyperRequest, T, S>>,
    pub(crate) ip: Option<IpAddr>,
}

impl<T: 'static + Context + Clone + Send + Sync, S: 'static + Send + Sync> Service<Request<Body>>
    for HyperService<T, S>
{
    type Response = T::Response;
    type Error = std::io::Error;
    type Future = ReusableBoxFuture<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut TaskContext<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let mut req = HyperRequest::new(req);
        req.ip = self.ip;

        self.app.clone().match_and_resolve(req)
    }
}
