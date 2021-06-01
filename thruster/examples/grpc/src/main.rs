// use tonic::{Request, Response, Status};

// use http_body::Body as HttpBody;
// use hyper::Body;
// use log::info;
// use thruster::context::hyper_request::HyperRequest;
// use thruster::context::typed_hyper_context::TypedHyperContext;
// use thruster::hyper_server::HyperServer;
// use thruster::Context;
// use thruster::{async_middleware, middleware_fn};
// use thruster::{App, ThrusterServer};
// use thruster::{MiddlewareNext, MiddlewareResult};

// use tower::Service;

// use hello_world::greeter_server::{Greeter, GreeterServer};
// use hello_world::{HelloReply, HelloRequest};

// type Ctx = TypedHyperContext<MyGreeter>;

// pub mod hello_world {
//     tonic::include_proto!("helloworld");
// }

// #[derive(Default, Clone, Copy)]
// pub struct MyGreeter {}

// #[tonic::async_trait]
// impl Greeter for MyGreeter {
//     async fn say_hello(
//         &self,
//         request: Request<HelloRequest>,
//     ) -> Result<Response<HelloReply>, Status> {
//         let reply = hello_world::HelloReply {
//             message: format!("Hello {}!", request.into_inner().name),
//         };
//         Ok(Response::new(reply))
//     }
// }

// fn generate_context(request: HyperRequest, state: &MyGreeter, _path: &str) -> Ctx {
//     Ctx::new(request, *state)
// }

// #[middleware_fn]
// async fn rpc(context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
//     let hyper_request = context.hyper_request.unwrap().request;
//     let resp = GreeterServer::new(MyGreeter::default())
//         .call(hyper_request)
//         .await
//         .unwrap();

//     let mut context = Ctx::default();
//     for (key, value) in resp.headers() {
//         context.set(key.as_ref(), value.to_str().unwrap());
//     }
//     context.set("grpc-status", "0");
//     context.status(resp.status().as_u16() as u32);
//     context.set_http2();

//     let mut resp_body = resp.into_body();
//     let body = resp_body.data().await.unwrap().unwrap();

//     context.body = Body::from(body);

//     Ok(context)
// }

// fn main() {
//     env_logger::init();
//     info!("Starting server...");

//     let mut app =
//         App::<HyperRequest, Ctx, MyGreeter>::create(generate_context, MyGreeter::default());
//     app.post(
//         "/helloworld.Greeter/SayHello",
//         async_middleware!(Ctx, [rpc]),
//     );

//     let server = HyperServer::new(app);
//     server.start("0.0.0.0", 50051);
// }

fn main() {
    println!("This is disabled for now.");
}
