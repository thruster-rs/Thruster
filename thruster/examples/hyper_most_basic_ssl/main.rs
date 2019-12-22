use thruster::ssl_hyper_server::SSLHyperServer;
use thruster::thruster_context::basic_hyper_context::{
    generate_context, BasicHyperContext as Ctx, HyperRequest,
};
use thruster::thruster_proc::{async_middleware, middleware_fn};
use thruster::App;
use thruster::ThrusterServer;
use thruster::{MiddlewareNext, MiddlewareReturnValue};

#[middleware_fn]
async fn plaintext(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> Ctx {
    let val = "Hello, World!";
    context.body(val);
    context
}

#[middleware_fn]
async fn test_fn_404(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> Ctx {
    context.body("404");
    context
}

fn main() {
    println!("Starting server...");

    let mut app = App::<HyperRequest, Ctx>::create(generate_context);

    app.get("/plaintext", async_middleware!(Ctx, [plaintext]));
    app.set404(async_middleware!(Ctx, [test_fn_404]));

    let mut server = SSLHyperServer::new(app);
    server.cert(include_bytes!("identity.p12").to_vec());
    server.cert_pass("asdfasdfasdf");
    server.start("0.0.0.0", 4321);
}
