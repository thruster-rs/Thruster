use std::net::ToSocketAddrs;

use async_trait::async_trait;
use hyper::server::conn::Http;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
#[cfg(not(windows))]
use net2::unix::UnixTcpBuilderExt;
use net2::TcpBuilder;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::stream::StreamExt;

use crate::app::App;
use crate::context::hyper_request::HyperRequest;
use crate::core::context::Context;
use crate::server::ThrusterServer;

pub struct HyperServer<T: 'static + Context + Send, S: Send> {
    app: App<HyperRequest, T, S>,
}

fn reuse_listener(
    addr: &SocketAddr,
    handle: &tokio::runtime::Handle,
) -> std::io::Result<std::net::TcpListener> {
    let builder = match *addr {
        SocketAddr::V4(_) => net2::TcpBuilder::new_v4()?,
        SocketAddr::V6(_) => net2::TcpBuilder::new_v6()?,
    };

    #[cfg(unix)]
    {
        use net2::unix::UnixTcpBuilderExt;
        if let Err(e) = builder.reuse_port(true) {
            eprintln!("error setting SO_REUSEPORT: {}", e);
        }
    }

    builder.reuse_address(true)?;
    builder.bind(addr)?;
    builder.listen(1024)
    // .and_then(|l| tokio::net::TcpListener::from_listener(l, addr, handle))
}

impl<T: Context<Response = Response<Body>> + Send, S: 'static + Send + Sync> HyperServer<T, S> {
    async fn process(
        arc_app: Arc<App<HyperRequest, T, S>>,
        tcp_listener: std::net::TcpListener,
    ) -> Result<(), hyper::Error> {
        let service = make_service_fn(|_| {
            let app = arc_app.clone();

            async {
                Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                    println!("cpu_id: {:?}", std::thread::current().id());
                    let matched = app.resolve_from_method_and_path(
                        &req.method().to_string(),
                        &req.uri().path_and_query().unwrap().to_string(),
                    );

                    let req = HyperRequest::new(req);
                    app.resolve(req, matched)
                }))
            }
        });

        let server = Server::from_tcp(tcp_listener).unwrap().serve(service);

        println!("starting server on {:?}", std::thread::current().id());
        server.await?;
        println!("server started...");

        Ok::<_, hyper::Error>(())
    }

    #[allow(dead_code)]
    pub async fn build_per_thread(mut self, host: &str, port: u16) {
        self.app._route_parser.optimize();

        let arc_app = Arc::new(self.app);

        let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();
        for _ in 0..num_cpus::get() - 1 {
            let arc_app = arc_app.clone();

            std::thread::spawn(move || {
                println!("spawned on thread {:?}", std::thread::current().id());

                let mut rt = tokio::runtime::Builder::new()
                    .basic_scheduler()
                    .enable_all()
                    .build()
                    .unwrap();

                rt.handle().enter(|| {
                    let mut rt = tokio::runtime::Builder::new()
                        .basic_scheduler()
                        .enable_all()
                        .build()
                        .unwrap();
                    let mut listener = {
                        let builder = net2::TcpBuilder::new_v4().unwrap();
                        #[cfg(not(windows))]
                        builder.reuse_address(true).unwrap();
                        #[cfg(not(windows))]
                        builder.reuse_port(true).unwrap();
                        builder.bind(addr).unwrap();
                        tokio::net::TcpListener::from_std(builder.listen(2048).unwrap()).unwrap()
                    };

                    rt.block_on(async move {
                        let mut incoming = listener.incoming();
                        let mut rt = tokio::runtime::Builder::new()
                            .basic_scheduler()
                            .enable_all()
                            .build()
                            .unwrap();

                        while let Some(Ok(stream)) = incoming.next().await {
                            let cloned = arc_app.clone();
                            let mut http = Http::new();
                            http.http1_only(true);
                            http.pipeline_flush(true);

                            let s = service_fn(move |req: Request<Body>| {
                                println!("cpu_id: {:?}", std::thread::current().id());
                                let matched = cloned.resolve_from_method_and_path(
                                    &req.method().to_string(),
                                    &req.uri().path_and_query().unwrap().to_string(),
                                );

                                let req = HyperRequest::new(req);
                                cloned.resolve(req, matched)
                            });

                            rt.block_on(http.serve_connection(stream, s));
                        }
                    });
                });
            });
        }

        println!("main thread: {:?}", std::thread::current().id());
        let addr = ("0.0.0.0", 4322).to_socket_addrs().unwrap().next().unwrap();
        Self::process(arc_app, std::net::TcpListener::bind(&addr).unwrap())
            .await
            .expect("hyper server failed");
    }
}

#[async_trait]
impl<T: Context<Response = Response<Body>> + Send, S: 'static + Send + Sync> ThrusterServer
    for HyperServer<T, S>
{
    type Context = T;
    type Response = Response<Body>;
    type Request = HyperRequest;
    type State = S;

    fn new(app: App<Self::Request, T, Self::State>) -> Self {
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
                            &req.uri().path_and_query().unwrap().to_string(),
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
