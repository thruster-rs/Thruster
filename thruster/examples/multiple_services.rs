use thruster::server::Server;
use thruster::thruster_proc::{async_middleware, middleware_fn};
use thruster::ThrusterServer;
use thruster::{App, BasicContext as Ctx, Request};
use thruster::{MiddlewareNext, MiddlewareReturnValue};

#[middleware_fn]
async fn plaintext(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> Ctx {
    let val = "Hello, World!";
    context.body(val);
    context
}

#[tokio::main]
async fn main() {
    println!("Starting server...");

    let mut app = App::<Request, Ctx>::new_basic();

    app.get("/plaintext", async_middleware!(Ctx, [plaintext]));

    let server = Server::new(app);
    tokio::spawn(server.build("0.0.0.0", 4321));

    let mut healthcheck = App::<Request, Ctx>::new_basic();

    app.get("/healthz", async_middleware(Ctx, [|ctx, _| ctx]));

    tokio::spawn(server.build("0.0.0.0", 8080));

    futures::future::pending().await
}
