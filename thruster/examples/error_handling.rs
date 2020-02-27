use snafu::{ResultExt, Snafu};

use thruster::errors::ThrusterError;
use thruster::{async_middleware, middleware_fn};
use thruster::{map_try, App, BasicContext as Ctx, Request, Server, ThrusterServer};
use thruster::{MiddlewareNext, MiddlewareResult, MiddlewareReturnValue};

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("Could not parse id: {}", id))]
    InvalidId {
        id: String,
        source: std::num::ParseIntError,
    },

    #[snafu(display("Could not open config at {}: {}", filename.display(), source))]
    FileNotFound {
        filename: std::path::PathBuf,
        source: std::io::Error,
    },
}

trait ErrorExt {
    fn context(self, context: Ctx) -> ThrusterError<Ctx>;
}

impl<E: Into<Error>> ErrorExt for E {
    fn context(self, context: Ctx) -> ThrusterError<Ctx> {
        ThrusterError {
            context,
            message: "Failed to handle error".to_string(),
            status: 500,
            cause: Some(Box::new(self.into())),
        }
    }
}

#[middleware_fn]
async fn json_error_handler(context: Ctx, next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let res = next(context).await;

    // If there is not an error, return
    let mut err = match res {
        Ok(_) => return res,
        Err(err) => err,
    };

    // Generic handler, if we fail to downcast
    let e: Box<Error> = match err.cause.take().map(|cause| cause.downcast()) {
        Some(Ok(e)) => e,
        _ => {
            let mut context = err.context;

            context.body(&format!(
                "{{\"message\": \"{}\",\"success\":false}}",
                err.message
            ));
            context.status(err.status);

            return Ok(context);
        }
    };

    // Handle the Error variants
    let mut context = err.context;
    let status = match *e {
        Error::InvalidId { .. } => 400,
        Error::FileNotFound { .. } => 404,
    };

    context.status(status);
    context.body(&format!("{{\"message\": \"{}\",\"success\":false}}", e));

    Ok(context)
}

#[middleware_fn]
async fn error(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let id = String::from("Hello, world");
    let res = id.parse::<u32>().context(InvalidId { id });
    let non_existent_param = map_try!(res, Err(err) => err.context(context));

    context.body(&format!("{}", non_existent_param));

    Ok(context)
}

#[middleware_fn]
async fn four_oh_four(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    context.status(404);
    context.body("Whoops! That route doesn't exist!");
    Ok(context)
}

fn main() {
    println!("Starting server...");

    let mut app = App::<Request, Ctx>::new_basic();

    app.use_middleware("/", async_middleware!(Ctx, [json_error_handler]));

    app.get("/error", async_middleware!(Ctx, [error]));
    app.set404(async_middleware!(Ctx, [four_oh_four]));

    let server = Server::new(app);
    server.start("0.0.0.0", 4321);
}
