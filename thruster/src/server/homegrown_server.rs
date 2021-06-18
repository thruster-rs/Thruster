use crate::ReusableBoxFuture;
use async_trait::async_trait;
use futures::{FutureExt, SinkExt, StreamExt};
use socket2::{Domain, Socket, Type};
use std::net::ToSocketAddrs;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::codec::Framed;

use crate::app::App;
use crate::core::context::Context;
use crate::core::http::Http;
use crate::core::request::Request;
use crate::core::response::Response;

// use std::thread;
// use num_cpus;
// use net2::TcpBuilder;
// #[cfg(not(windows))]
// use net2::unix::UnixTcpBuilderExt;

use crate::server::ThrusterServer;

pub struct Server<
    T: 'static + Context<Response = Response> + Clone + Send + Sync,
    S: 'static + Send + Sync,
> {
    app: Arc<App<Request, T, S>>,
}

impl<T: 'static + Context<Response = Response> + Clone + Send + Sync, S: 'static + Send + Sync>
    Server<T, S>
{
    ///
    /// Starts the app with the default tokio runtime execution model
    ///
    pub fn start_work_stealing_optimized(self, host: &str, port: u16) {
        self.start(host, port);
    }

    ///
    /// Starts the app with a thread pool optimized for small requests and quick timeouts. This
    /// is done internally by spawning a separate thread for each reactor core. This is valuable
    /// if all server endpoints are similar in their load, as work is divided evenly among threads.
    /// As seanmonstar points out though, this is a very specific use case and might not be useful
    /// for everyday work loads.alloc
    ///
    /// See the discussion here for more information:
    ///
    /// https://users.rust-lang.org/t/getting-tokio-to-match-actix-web-performance/18659/7
    ///
    pub fn start_small_load_optimized(self, host: &str, port: u16) {
        // panic!("Nope!");

        let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();
        let mut threads = Vec::new();

        let arc_app = Arc::new(self.app);

        for _ in 0..num_cpus::get() {
            let arc_app = arc_app.clone();
            threads.push(std::thread::spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_io()
                    .build()
                    .unwrap();

                let server = async move {
                    let listener = {
                        let socket = Socket::new(Domain::IPV4, Type::STREAM, None).unwrap();

                        let address = addr.into();
                        socket.set_reuse_address(true).unwrap();
                        #[cfg(unix)]
                        socket.set_reuse_port(true).unwrap();
                        socket.bind(&address).unwrap();
                        socket.listen(1024).unwrap();
                        socket.set_nonblocking(true).unwrap();

                        let listener: std::net::TcpListener = socket.into();
                        tokio::net::TcpListener::from_std(listener).unwrap()
                    };

                    TcpListenerStream::new(listener)
                        .for_each(move |socket| {
                            process(Arc::clone(&arc_app), socket.unwrap());
                            async {}
                        })
                        .await;
                };

                runtime.block_on(server);
            }));
        }

        for thread in threads {
            thread.join().unwrap();
        }
    }
}

#[async_trait]
impl<T: Context<Response = Response> + Clone + Send + Sync, S: 'static + Send + Sync> ThrusterServer
    for Server<T, S>
{
    type Context = T;
    type Response = Response;
    type Request = Request;
    type State = S;

    fn new(mut app: App<Self::Request, T, S>) -> Self {
        app = app.commit();

        Server { app: Arc::new(app) }
    }

    fn build(self, host: &str, port: u16) -> ReusableBoxFuture<()> {
        let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();

        // self.app._route_parser.optimize();

        let arc_app = self.app;
        let listener_fut = TcpListener::bind(addr).then(move |listener| {
            TcpListenerStream::new(listener.unwrap()).for_each(move |res| {
                if let Ok(stream) = res {
                    let cloned = arc_app.clone();
                    tokio::spawn(process(cloned, stream));
                }

                async { () }
            })
        });

        ReusableBoxFuture::new(listener_fut)
    }
}

struct _Error {
    _message: String,
}

fn process<T: Context<Response = Response> + Clone + Send + Sync, S: 'static + Send + Sync>(
    app: Arc<App<Request, T, S>>,
    socket: TcpStream,
) -> ReusableBoxFuture<Result<(), _Error>> {
    let app = app.clone();

    ReusableBoxFuture::new(async move {
        let mut framed = Framed::new(socket, Http);

        while let Some(request) = framed.next().await {
            match request {
                Ok(request) => {
                    let path = request.path().to_owned();
                    let method = &request.method().to_owned();
                    let matched = app.resolve_from_method_and_path(method, path);
                    let response = app.resolve(request, matched).await.map_err(|e| _Error {
                        _message: e.to_string(),
                    })?;
                    framed.send(response).await.map_err(|e| _Error {
                        _message: e.to_string(),
                    })?;
                }
                Err(e) => {
                    return Err(_Error {
                        _message: e.to_string(),
                    })
                }
            }
        }

        Ok(())
    })
}
