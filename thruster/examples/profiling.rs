use log::info;
use std::time::Instant;
use thruster::{async_middleware, middleware_fn};
use thruster::{App, BasicContext as Ctx, Request, Server, ThrusterServer};
use thruster::{MiddlewareNext, MiddlewareResult};

#[middleware_fn]
async fn profiling(mut context: Ctx, next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let start_time = Instant::now();

    context = next(context).await?;

    let elapsed_time = start_time.elapsed();
    info!(
        "[{}μs] {} -- {}",
        elapsed_time.as_micros(),
        context.request.method(),
        context.request.path()
    );

    Ok(context)
}

#[middleware_fn]
async fn plaintext(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let val = "Hello, World!";
    context.body(val);
    Ok(context)
}

#[middleware_fn]
async fn test_fn_404(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    context.body("404");
    Ok(context)
}

fn main() {
    env_logger::init();
    info!("Starting server...");

    let mut app = App::<Request, Ctx, ()>::new_basic();

    app.use_middleware("/", async_middleware!(Ctx, [profiling]));
    app.get("/plaintext", async_middleware!(Ctx, [plaintext]));
    app.set404(async_middleware!(Ctx, [test_fn_404]));

    let server = Server::new(app);

    server.start("0.0.0.0", 4321);
}
