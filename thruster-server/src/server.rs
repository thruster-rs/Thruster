use std::net::ToSocketAddrs;

use futures_legacy::future;
use tokio;
use tokio::net::{TcpStream, TcpListener};
use tokio::prelude::*;
use tokio_codec::Framed;
use num_cpus;
use net2::TcpBuilder;
#[cfg(not(windows))]
use net2::unix::UnixTcpBuilderExt;
use std::thread;

use std::sync::Arc;

use thruster_app::app::App;
use thruster_core::context::Context;
use thruster_core::http::Http;
use thruster_core::response::Response;
use thruster_core::request::Request;

use crate::thruster_server::ThrusterServer;

pub struct Server<T: 'static + Context<Response = Response> + Send> {
  app: App<Request, T>
}

impl<T: 'static + Context<Response = Response> + Send> Server<T> {
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
  pub fn start_small_load_optimized(mut self, host: &str, port: u16) {
    let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();
    let mut threads = Vec::new();
    self.app._route_parser.optimize();
    let arc_app = Arc::new(self.app);

    for _ in 0..num_cpus::get() {
      let arc_app = arc_app.clone();
      threads.push(thread::spawn(move || {
        let mut runtime = tokio::runtime::current_thread::Runtime::new().unwrap();

        let server = future::lazy(move || {
          let listener = {
            let builder = TcpBuilder::new_v4().unwrap();
            #[cfg(not(windows))]
            builder.reuse_address(true).unwrap();
            #[cfg(not(windows))]
            builder.reuse_port(true).unwrap();
            builder.bind(addr).unwrap();
            builder.listen(2048).unwrap()
          };
          let listener = TcpListener::from_std(listener, &tokio::reactor::Handle::default()).unwrap();

          listener.incoming().for_each(move |socket| {
            process(Arc::clone(&arc_app), socket);
            Ok(())
          })
          .map_err(|err| eprintln!("accept error = {:?}", err))
        });

        runtime.spawn(server);
        runtime.run().unwrap();
      }));
    }

    println!("Server running on {}", addr);

    for thread in threads {
      thread.join().unwrap();
    }

    fn process<T: Context<Response = Response> + Send>(app: Arc<App<Request, T>>, socket: TcpStream) {
      let framed = Framed::new(socket, Http);
      let (tx, rx) = framed.split();

      let task = tx.send_all(rx.and_then(move |request: Request| {
            let matched = app.resolve_from_method_and_path(request.method(), request.path());
            app.resolve(request, matched)
          }))
          .then(|_| future::ok(()));

      // Spawn the task that handles the connection.
      tokio::spawn(task);
    }
  }
}

impl<T: Context<Response = Response> + Send> ThrusterServer for Server<T> {
  type Context = T;
  type Response = Response;
  type Request = Request;

  fn new(app: App<Self::Request, T>) -> Self {
    Server {
      app
    }
  }

  ///
  /// Alias for start_work_stealing_optimized
  ///
  fn start(mut self, host: &str, port: u16) {
    let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();

    self.app._route_parser.optimize();

    let listener = TcpListener::bind(&addr).unwrap();
    let arc_app = Arc::new(self.app);

    fn process<T: Context<Response = Response> + Send>(app: Arc<App<Request, T>>, socket: TcpStream) {
      let framed = Framed::new(socket, Http);
      let (tx, rx) = framed.split();

      let task = tx.send_all(rx.and_then(move |request: Request| {
            let matched = app.resolve_from_method_and_path(request.method(), request.path());
            app.resolve(request, matched)
          }))
          .then(|_| {
            future::ok(())
          });

      // Spawn the task that handles the connection.
      tokio::spawn(task);
    }

    let server = listener.incoming()
        .map_err(|e| println!("error = {:?}", e))
        .for_each(move |socket| {
            let _ = socket.set_nodelay(true);
            process(arc_app.clone(), socket);
            Ok(())
        });

    tokio::run(server);
  }
}
