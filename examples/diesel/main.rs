extern crate futures;
extern crate serde;
extern crate serde_json;
extern crate smallvec;
extern crate tokio;
extern crate dotenv;

#[macro_use] extern crate diesel;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate thruster;

mod context;
mod schema;
mod content_model;

use futures::future;

use thruster::{App, MiddlewareChain, MiddlewareReturnValue};
use thruster::builtins::server::Server;
use thruster::server::ThrusterServer;
use crate::context::{Ctx, generate_context};
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use crate::schema::content::dsl::*;
use dotenv::dotenv;
use std::env;

lazy_static! {
    static ref psql: Pool<ConnectionManager<PgConnection>> = {
      dotenv().ok();

      let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

      let manager = ConnectionManager::<PgConnection>::new(database_url);
      Pool::new(manager).expect("Error creating db pool")
    };
}

fn fetch_value(mut context: Ctx, _next: impl Fn(Ctx) -> MiddlewareReturnValue<Ctx>  + Send + Sync) -> MiddlewareReturnValue<Ctx> {
  let conn = psql.get().unwrap();

  let results = content
    .limit(1)
    .load::<content_model::Content>(&conn)
    .unwrap();

  let result = results.get(0).unwrap();
  context.body = result.val.clone();

  Box::new(future::ok(context))
}

fn not_found_404(mut context: Ctx, _next: impl Fn(Ctx) -> MiddlewareReturnValue<Ctx>  + Send + Sync) -> MiddlewareReturnValue<Ctx> {
  context.body = "<html>
  ( ͡° ͜ʖ ͡°) What're you looking for here?
</html>".to_owned();
  context.set_header("Content-Type", "text/html");
  context.status_code = 404;

  Box::new(future::ok(context))
}

fn main() {
  println!("Starting server...");

  let app = App::create(generate_context);

  // app.get("/plaintext", middleware![Ctx => fetch_value]);
  // app.get("/*", middleware![Ctx => not_found_404]);

  let server = Server::new(app);
  server.start("0.0.0.0", 4321);
}
