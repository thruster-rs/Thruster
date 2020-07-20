use std::net::ToSocketAddrs;

use async_trait::async_trait;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::net::SocketAddr;
use std::sync::Arc;

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
        cpu_id: Arc<usize>,
    ) -> Result<(), hyper::Error> {
        let service = make_service_fn(|_| {
            let app = arc_app.clone();
            let cpu_id = cpu_id.clone();

            async {
                Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                    println!("cpu_id: {}", cpu_id.clone());
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

        server.await?;

        Ok::<_, hyper::Error>(())
    }

    #[allow(dead_code)]
    pub async fn build_per_thread(mut self, host: &str, port: u16) {
        self.app._route_parser.optimize();

        let arc_app = Arc::new(self.app);

        let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();
        let arc_addr = Arc::new(addr);
        for i in 0..num_cpus::get() - 1 {
            let arc_app = arc_app.clone();
            let arc_addr = arc_addr.clone();
            std::thread::spawn(move || {
                let mut rt = tokio::runtime::Runtime::new().unwrap();
                let tcp_listener = reuse_listener(&arc_addr.clone(), &rt.handle()).unwrap();
                tokio::task::LocalSet::new().block_on(&mut rt, async move {
                    Self::process(arc_app, tcp_listener, Arc::new(i))
                        .await
                        .expect("hyper server failed");
                });
            });
        }

        let handle = tokio::runtime::Handle::current();
        let tcp_listener = reuse_listener(&arc_addr.clone(), &handle).unwrap();

        Self::process(arc_app, tcp_listener, Arc::new(num_cpus::get() - 1))
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
