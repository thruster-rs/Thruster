#[macro_use]
extern crate serde_json;
extern crate frank_jwt;

use snafu::{ResultExt, Snafu};

use log::info;
use serde_json::{json, Value};
use thruster::core::response::Response;
use thruster::errors::ThrusterError;
use thruster::{async_middleware, middleware_fn};
use thruster::{map_try, App, Request, Server, ThrusterServer};
use thruster::{MiddlewareNext, MiddlewareResult};

struct JwtContext {
    response: Response,
    pub cookies: Vec<Cookie>,
    pub params: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub request: Request,
    pub status: u32,
    pub headers: HashMap<String, String>,
    pub user_id: String,
}

impl JwtContext {
    pub fn new() -> JwtContext {
        let mut ctx = JwtContext {
            response: Response::new(),
            cookies: Vec::new(),
            params: HashMap::new(),
            query_params: HashMap::new(),
            request: Request::new(),
            headers: HashMap::new(),
            status: 200,
            user_id: String::new(),
        };

        ctx.set("Server", "Thruster");

        ctx
    }

    ///
    /// Set the body as a string
    ///
    pub fn body(&mut self, body_string: &str) {
        self.response
            .body_bytes_from_vec(body_string.as_bytes().to_vec());
    }

    pub fn get_body(&self) -> String {
        str::from_utf8(&self.response.response)
            .unwrap_or("")
            .to_owned()
    }

    ///
    /// Set the response status code
    ///
    pub fn status(&mut self, code: u32) {
        self.status = code;
    }

    ///
    /// Set the response `Content-Type`. A shortcode for
    ///
    /// ```ignore
    /// ctx.set("Content-Type", "some-val");
    /// ```
    ///
    pub fn content_type(&mut self, c_type: &str) {
        self.set("Content-Type", c_type);
    }

    ///
    /// Set up a redirect, will default to 302, but can be changed after
    /// the fact.
    ///
    /// ```ignore
    /// ctx.set("Location", "/some-path");
    /// ctx.status(302);
    /// ```
    ///
    pub fn redirect(&mut self, destination: &str) {
        self.status(302);

        self.set("Location", destination);
    }

    ///
    /// Sets a cookie on the response
    ///
    pub fn cookie(&mut self, name: &str, value: &str, options: &CookieOptions) {
        let cookie_value = match self.headers.get("Set-Cookie") {
            Some(val) => format!("{}, {}", val, self.cookify_options(name, value, &options)),
            None => self.cookify_options(name, value, &options),
        };

        self.set("Set-Cookie", &cookie_value);
    }

    fn cookify_options(&self, name: &str, value: &str, options: &CookieOptions) -> String {
        let mut pieces = vec![format!("Path={}", options.path)];

        if options.expires > 0 {
            pieces.push(format!("Expires={}", options.expires));
        }

        if options.max_age > 0 {
            pieces.push(format!("Max-Age={}", options.max_age));
        }

        if !options.domain.is_empty() {
            pieces.push(format!("Domain={}", options.domain));
        }

        if options.secure {
            pieces.push("Secure".to_owned());
        }

        if options.http_only {
            pieces.push("HttpOnly".to_owned());
        }

        if let Some(ref same_site) = options.same_site {
            match same_site {
                SameSite::Strict => pieces.push("SameSite=Strict".to_owned()),
                SameSite::Lax => pieces.push("SameSite=Lax".to_owned()),
            };
        }

        format!("{}={}; {}", name, value, pieces.join(", "))
    }
}

impl Context for JwtContext {
    type Response = Response;

    fn get_response(mut self) -> Self::Response {
        self.response.status_code(self.status, "");

        self.response
    }

    fn set_body(&mut self, body: Vec<u8>) {
        self.response.body_bytes_from_vec(body);
    }

    fn set_body_bytes(&mut self, body_bytes: Bytes) {
        self.response.body_bytes(&body_bytes);
    }

    fn route(&self) -> &str {
        self.request.path()
    }

    fn set(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_owned(), value.to_owned());
    }

    fn remove(&mut self, key: &str) {
        self.headers.remove(key);
    }
}

impl HasQueryParams for JwtContext {
    fn set_query_params(&mut self, query_params: HashMap<String, String>) {
        self.query_params = query_params;
    }
}

impl HasCookies for JwtContext {
    fn set_cookies(&mut self, cookies: Vec<Cookie>) {
        self.cookies = cookies;
    }

    fn headers(&self) -> HashMap<String, String> {
        self.request.headers()
    }
}

#[middleware_fn]
async fn jwt_handler(
    mut context: JwtContext,
    next: MiddlewareNext<JwtContext>,
) -> MiddlewareResult<JwtContext> {
    let jwt = context.headers().get("Authorization");

    if jwt.is_none() {
        context.body("{{\"message\": \"Authorization header required\",\"success\":false}}");
        context.status(401); // Unauthorized

        return Ok(context);
    }
    let jwt = jwt.unwrap();

    let (_, payload) = frank_jwt::decode(
        &jwt,
        "secret123",
        frank_jwt::Algorithm::HS256,
        &frank_jwt::ValidationOptions::default(),
    );

    context.user_id = payload.get("userId").unwrap_or("undefined");

    next(context).await
}

#[middleware_fn]
async fn user_data(
    mut context: JwtContext,
    _next: MiddlewareNext<JwtContext>,
) -> MiddlewareResult<JwtContext> {
    context.body(&format!(
        "Hello there. Looks like you are user {}",
        context.user_id
    ));

    Ok(context)
}

#[middleware_fn]
async fn user_login(
    mut context: JwtContext,
    _next: MiddlewareNext<JwtContext>,
) -> MiddlewareResult<JwtContext> {
    let body: Value = serde_json::from_str(&context.get_body());

    // I hope to god this isn't your password. If it is, please reconsider your password policy
    if body.get("username") == Some("test_user") && body.get("password") == Some("pass123") {
        // Authentication  done. Generate JWT now

        // JWT generation taken from:
        // https://github.com/GildedHonour/frank_jwt

        let mut payload = json!({
            "userId": "test_user",
            "allowed_resources": ["test_values","print_hello","sleep","cry","repeat"]
        });
        let mut header = json!({});
        let secret = "secret123";
        let jwt = frank_jwt::encode(
            &header,
            secret.to_string(),
            &payload,
            frank_jwt::Algorithm::HS256,
        )
        .unwrap();

        context.body(&format!("{{\"success\": true, \"jwt\": \"{}\"}}",));

        Ok(context)
    } else {
        context.body(
            "{{\"success\": false, \"message\": \"Wrong credentials. Use test_user and pass123\"}}",
        );

        Ok(context)
    }
}

#[middleware_fn]
async fn four_oh_four(
    mut context: JwtContext,
    _next: MiddlewareNext<JwtContext>,
) -> MiddlewareResult<JwtContext> {
    context.status(404);
    context.body("Whoops! That route doesn't exist!");
    Ok(context)
}

fn main() {
    env_logger::init();
    info!("Starting server...");

    let mut app = App::<Request, JwtContext>::new_basic();

    app.use_middleware("/data", async_middleware!(JwtContext, [jwt_handler]));

    app.get("/data", async_middleware!(JwtContext, [user]));
    app.get("/login", async_middleware!(JwtContext, [login]));
    app.set404(async_middleware!(JwtContext, [four_oh_four]));

    let server = Server::new(app);
    server.start("0.0.0.0", 4321);
}
