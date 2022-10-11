use snafu::{ResultExt, Snafu};

use log::info;
use thruster::errors::ThrusterError;
use thruster::{m, middleware_fn, Context};
use thruster::{map_try, App, BasicContext as Ctx, Request, Server, ThrusterServer};
use thruster::{MiddlewareNext, MiddlewareResult};

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
    fn context(self, mut context: Ctx) -> ThrusterError<Ctx> {
        context.status(500);

        ThrusterError {
            context,
            message: "Failed to handle error".to_string(),
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

            return Ok(context);
        }
    };

    // Handle the Error variants
    let mut context = err.context;
    context.status(match *e {
        Error::InvalidId { .. } => 400,
        Error::FileNotFound { .. } => 404,
    });

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
    env_logger::init();
    info!("Starting server...");

    let app = App::<Request, Ctx, ()>::new_basic()
        .middleware("/", m![json_error_handler])
        .get("/error", m![error])
        .set404(m![four_oh_four]);

    let server = Server::new(app);
    server.start("0.0.0.0", 4321);
}
