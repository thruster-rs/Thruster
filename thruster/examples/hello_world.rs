use thruster::server::Server;
use thruster::thruster_proc::{async_middleware, middleware_fn};
use thruster::ThrusterServer;
use thruster::{App, BasicContext as Ctx, Request};
use thruster::{MiddlewareNext, MiddlewareReturnValue, MiddlewareResult};

#[middleware_fn]
async fn plaintext(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let val = "Hello, World!";
    context.body(val);
    Ok(context)
}

fn main() {
    println!("Starting server...");

    let mut app = App::<Request, Ctx>::new_basic();

    app.get("/plaintext", async_middleware!(Ctx, [plaintext]));

    let server = Server::new(app);
    server.start("0.0.0.0", 4321);
}
