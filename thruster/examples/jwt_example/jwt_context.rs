use bytes::Bytes;
use std::collections::HashMap;
use std::str;
use thruster::{
    middleware::{
        cookies::{Cookie, CookieOptions, HasCookies, SameSite},
        query_params::HasQueryParams
    },
    Context, Request, Response,
};

pub struct JwtContext {
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
