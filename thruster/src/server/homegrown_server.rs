use futures::SinkExt;
use std::error::Error;
use std::net::ToSocketAddrs;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::net::{TcpListener, TcpStream};
use tokio::stream::StreamExt;
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

pub struct Server<T: 'static + Context<Response = Response> + Send> {
    app: App<Request, T>,
}

// impl<T: 'static + Context<Response = Response> + Send> Server<T> {
//   ///
//   /// Starts the app with the default tokio runtime execution model
//   ///
//   pub fn start_work_stealing_optimized(self, host: &str, port: u16) {
//     self.start(host, port);
//   }

//   ///
//   /// Starts the app with a thread pool optimized for small requests and quick timeouts. This
//   /// is done internally by spawning a separate thread for each reactor core. This is valuable
//   /// if all server endpoints are similar in their load, as work is divided evenly among threads.
//   /// As seanmonstar points out though, this is a very specific use case and might not be useful
//   /// for everyday work loads.alloc
//   ///
//   /// See the discussion here for more information:
//   ///
//   /// https://users.rust-lang.org/t/getting-tokio-to-match-actix-web-performance/18659/7
//   ///
//   pub fn start_small_load_optimized(mut self, host: &str, port: u16) {
//     let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();
//     let mut threads = Vec::new();
//     self.app._route_parser.optimize();
//     let arc_app = Arc::new(self.app);

//     for _ in 0..num_cpus::get() {
//       let arc_app = arc_app.clone();
//       threads.push(thread::spawn(move || {
//         let mut runtime = tokio::runtime::current_thread::Runtime::new().unwrap();

//         let server = async || {
//           let listener = {
//             let builder = TcpBuilder::new_v4().unwrap();
//             #[cfg(not(windows))]
//             builder.reuse_address(true).unwrap();
//             #[cfg(not(windows))]
//             builder.reuse_port(true).unwrap();
//             builder.bind(addr).unwrap();
//             builder.listen(2048).unwrap()
//           };
//           let listener = TcpListener::from_std(listener, &tokio::reactor::Handle::default()).unwrap();

//           listener.incoming().for_each(move |socket| {
//             process(Arc::clone(&arc_app), socket);
//             Ok(())
//           })
//           .map_err(|err| eprintln!("accept error = {:?}", err))
//         };

//         runtime.spawn(server);
//         runtime.run().unwrap();
//       }));
//     }

//     println!("Server running on {}", addr);

//     for thread in threads {
//       thread.join().unwrap();
//     }

//     fn process<T: Context<Response = Response> + Send>(app: Arc<App<Request, T>>, socket: TcpStream) {
//       let framed = Framed::new(socket, Http);
//       let (tx, rx) = framed.split();

//       let task = tx.send_all(rx.and_then(move |request: Request| {
//             let matched = app.resolve_from_method_and_path(request.method(), request.path());
//             app.resolve(request, matched)
//           }));

//       // Spawn the task that handles the connection.
//       tokio::spawn(task);
//     }
//   }
// }

#[async_trait]
impl<T: Context<Response = Response> + Send> ThrusterServer for Server<T> {
    type Context = T;
    type Response = Response;
    type Request = Request;

    fn new(app: App<Self::Request, T>) -> Self {
        Server { app }
    }

    async fn build(mut self, host: &str, port: u16) {
        let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();

        self.app._route_parser.optimize();

        let mut listener = TcpListener::bind(&addr).await.unwrap();
        let mut incoming = listener.incoming();
        let arc_app = Arc::new(self.app);

        while let Some(Ok(stream)) = incoming.next().await {
            let cloned = arc_app.clone();
            tokio::spawn(async move {
                if let Err(e) = process(cloned, stream).await {
                    println!("failed to process connection; error = {}", e);
                }
            });
        }

        async fn process<T: Context<Response = Response> + Send>(
            app: Arc<App<Request, T>>,
            socket: TcpStream,
        ) -> Result<(), Box<dyn Error>> {
            let mut framed = Framed::new(socket, Http);

            while let Some(request) = framed.next().await {
                match request {
                    Ok(request) => {
                        let matched =
                            app.resolve_from_method_and_path(request.method(), request.path());
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
