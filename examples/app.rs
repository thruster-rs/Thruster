extern crate env_logger;
extern crate futures;
extern crate fanta;
extern crate tokio_proto;
extern crate tokio_service;

use fanta::{App, BasicContext, MiddlewareChain, Http};
use tokio_proto::TcpServer;

fn index(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> BasicContext {
  BasicContext {
    body: "Hello world".to_string(),
    _identifier: 0
  }
}

fn main() {
  println!("Starting server...");

  drop(env_logger::init());
  let addr = "0.0.0.0:8080".parse().unwrap();

  TcpServer::new(Http, addr)
      .serve(|| {
        let mut app = App::<BasicContext>::new();

        app.get("/index.html", vec![index]);

        Ok(app)
      });
}
