use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use socket2::{Domain, Socket, Type};
use std::error::Error;
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
    S: 'static + Send,
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
                            async {  }
                        })
                        .await;
                };

                runtime.block_on(server);
            }));
        }

        for thread in threads {
            thread.join().unwrap();
        }

        fn process<
            T: Context<Response = Response> + Clone + Send + Sync,
            S: 'static + Send + Sync,
        >(
            app: Arc<App<Request, T, S>>,
            socket: TcpStream,
        ) {
            // let framed = Framed::new(socket, Http);
            // let (tx, rx) = framed.split();

            // let task = tx.send_all(&mut rx.and_then(move |request: Request| {
            //     let matched =
            //         app.resolve_from_method_and_path(request.method(), request.path().to_owned());
            //     std::boxed::Box::pin(app.resolve(request, matched))
            // }));

            // Spawn the task that handles the connection.
            tokio::spawn(async move {
                let mut framed = Framed::new(socket, Http);

                while let Some(request) = framed.next().await {
                    match request {
                        Ok(request) => {
                            let path = request.path().to_owned();
                            let method = &request.method().to_owned();
                            let matched = app.resolve_from_method_and_path(method, path);
                            let response = app.resolve(request, matched).await.unwrap();
                            framed.send(response).await.unwrap();
                        }
                        Err(_e) => return ,
                    }
                }

                
            });
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

    async fn build(mut self, host: &str, port: u16) {
        let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();

        // self.app._route_parser.optimize();

        let mut listener = TcpListenerStream::new(TcpListener::bind(&addr).await.unwrap());
        let arc_app = self.app;

        while let Some(Ok(stream)) = listener.next().await {
            let cloned = arc_app.clone();
            tokio::spawn(async move {
                if let Err(e) = process(&cloned, stream).await {
                    println!("failed to process connection; error = {}", e);
                }
            });
        }

        async fn process<
            T: Context<Response = Response> + Clone + Send + Sync,
            S: 'static + Send,
        >(
            app: &App<Request, T, S>,
            socket: TcpStream,
        ) -> Result<(), Box<dyn Error>> {
            let mut framed = Framed::new(socket, Http);

            while let Some(request) = framed.next().await {
                match request {
                    Ok(request) => {
                        let path = request.path().to_owned();
                        let method = &request.method().to_owned();
                        let matched = app.resolve_from_method_and_path(method, path);
                        let response = app.resolve(request, matched).await?;
                        framed.send(response).await?;
                    }
                    Err(e) => return Err(e.into()),
                }
            }

            Ok(())
        }
    }
}
