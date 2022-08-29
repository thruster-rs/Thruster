use std::time::SystemTime;

use log::info;
use thruster::{m, middleware_fn, Context};
use thruster::{App, BasicContext as Ctx, Request, Server, ThrusterServer};
use thruster::{MiddlewareNext, MiddlewareResult};

#[middleware_fn]
async fn home(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    context.body(
        "<html>
<body>
    <div id=\"output\">Waiting for result...</div>

    <script>
        const output = document.getElementById('output');

        setInterval(async () => {
            output.textContent = (await tryFetch()).time;
        }, 1000);

        async function tryFetch() {
            const response = await fetch('http://localhost:8080/time', {
                method: 'GET',
                mode: 'cors', // no-cors, *cors, same-origin
            });

            return response.json();
        }
    </script>
</body>
</html>",
    );
    context.set("Content-Type", "text/html");
    Ok(context)
}

#[middleware_fn]
async fn time(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    context.body(&format!(
        "{{\"time\":{:?}}}",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    ));
    context.set("Content-Type", "application/json");

    Ok(context)
}

#[middleware_fn]
pub async fn cors(mut context: Ctx, next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let origin_env = std::env::var("CORS_ORIGIN").unwrap_or_else(|_| "*".to_string());

    // If the env var is a comma separated list, then we grab the origin header, and parse the env var
    // into a vec. If the origin matches any of the vec, then we set that as the CORS header.
    //
    // If the env var is a single value, then we simply use that as the header.
    let origin = if origin_env.contains(',') {
        // Only one origin header is allowed
        let header = context
            .request
            .headers()
            .get("Origin")
            .and_then(|header_values| header_values.first())
            .map(Clone::clone)
            .unwrap_or_else(|| "*".to_string());
        origin_env
            .split(',')
            .find(|v| v == &&header)
            .unwrap_or_else(|| "*")
    } else {
        &origin_env
    };

    context.set("Access-Control-Allow-Origin", &origin);
    context.set("Access-Control-Allow-Headers", "*");
    context.set(
        "Access-Control-Allow-Methods",
        "GET, POST, PUT, DELETE, OPTIONS",
    );

    next(context).await
}

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Starting server...");

    let app = App::<Request, Ctx, ()>::new_basic().get("/", m![home]);

    let server = Server::new(app);
    tokio::spawn(server.build("0.0.0.0", 4321));

    let healthcheck = App::<Request, Ctx, ()>::new_basic().get("/time", m![cors, time]);

    let healthcheck_server = Server::new(healthcheck);
    tokio::spawn(healthcheck_server.build("0.0.0.0", 8080));

    futures::future::pending().await
}
