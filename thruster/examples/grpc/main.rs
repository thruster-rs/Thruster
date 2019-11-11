#![feature(proc_macro_hygiene)]
extern crate thruster;

mod context;
mod hyper_server;

use thruster::{MiddlewareNext, MiddlewareReturnValue, ThrusterServer};
use thruster::App;
use thruster::thruster_context::basic_hyper_context::{HyperRequest};
use thruster::thruster_proc::{async_middleware, middleware_fn};
use hyper::Body;
use tonic;
use tonic::codec::Codec;
use tonic::body::BoxBody;
use http;
use futures_util::{future, stream, TryStreamExt};

use context::{generate_context, BasicHyperContext as Ctx};
use hyper_server::HyperServer;

/// The request message containing the user's name.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HelloRequest {
    #[prost(string, tag = "1")]
    pub name: std::string::String,
}
/// The response message containing the greetings
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HelloReply {
    #[prost(string, tag = "1")]
    pub message: std::string::String,
}

#[middleware_fn]
async fn plaintext(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> Ctx {
  // let val = "Hello, World!";
  // context.body = Body::from(val);
  context
}

async fn decode_request<
  I: 'static + std::default::Default + prost::Message,
  O: 'static + std::default::Default + prost::Message
>(hyper_request: HyperRequest) -> I {
  let mut codec1: tonic::codec::ProstCodec<O, I> =
    tonic::codec::ProstCodec::default();
  let mut grpc = tonic::server::Grpc::new(codec1);
  let (parts, body) = hyper_request
    .request
    .into_parts();

  let mut codec2: tonic::codec::ProstCodec<O, I> =
    tonic::codec::ProstCodec::default();
  let mut stream =
    tonic::codec::Streaming::new_request(codec2.decoder(), body);

  futures_util::pin_mut!(stream);

  let message = stream.try_next().await.unwrap().unwrap();
  let mut req: tonic::Request<I> = tonic::Request::from_http_parts(parts, message);

  req.into_inner()
}

#[middleware_fn]
async fn run_grpc(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> Ctx {
  let mut req: HelloRequest = decode_request::<HelloRequest, HelloReply>(context.hyper_request.unwrap()).await;

  let r = tonic::Response::new(HelloReply {
      message: format!("This is a test response: {}", req.name)
    });

  let (mut parts, body) = r.into_http().into_parts();

  // Set the content type
  parts.headers.insert(
      http::header::CONTENT_TYPE,
      http::header::HeaderValue::from_static("application/grpc"),
  );

  let mut codec3: tonic::codec::ProstCodec<HelloReply, HelloRequest> =
    tonic::codec::ProstCodec::default();
  let body = tonic::codec::encode::encode_server(
      codec3.encoder(),
      stream::once(future::ok(body)));

  let mut response = Ctx::default();

  response.hyper_response = Some(http::Response::from_parts(parts, BoxBody::new(body)));

  response
}

fn main() {
  println!("Starting server...");

  let mut app = App::<HyperRequest, Ctx>::create(generate_context);
  app.get("/plaintext", async_middleware!(Ctx, [plaintext]));
  app.post("/helloworld.Greeter/SayHello", async_middleware!(Ctx, [run_grpc]));

  let server = HyperServer::new(app);
  server.start("0.0.0.0", 50051);
}
