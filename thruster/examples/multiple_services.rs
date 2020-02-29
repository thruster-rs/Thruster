use thruster::{async_middleware, middleware_fn};
use thruster::{App, BasicContext as Ctx, Request, Server, ThrusterServer};
use thruster::{MiddlewareNext, MiddlewareResult};

#[middleware_fn]
async fn plaintext(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let val = "Hello, World!";
    context.body(val);
    Ok(context)
}

#[middleware_fn]
async fn noop(context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    Ok(context)
}

#[tokio::main]
async fn main() {
    println!("Starting server...");

    let mut app = App::<Request, Ctx>::new_basic();

    app.get("/plaintext", async_middleware!(Ctx, [plaintext]));

    let server = Server::new(app);
    tokio::spawn(server.build("0.0.0.0", 4321));

    let mut healthcheck = App::<Request, Ctx>::new_basic();

    healthcheck.get("/healthz", async_middleware!(Ctx, [noop]));

    let healthcheck_server = Server::new(healthcheck);
    tokio::spawn(healthcheck_server.build("0.0.0.0", 8080));

    futures::future::pending().await
}
