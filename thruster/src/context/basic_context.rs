use bytes::Bytes;
use std::collections::HashMap;
use std::str;

use crate::core::context::Context;
use crate::core::request::Request;
use crate::core::response::Response;

use crate::middleware::cookies::{Cookie, CookieOptions, HasCookies, SameSite};
use crate::middleware::query_params::HasQueryParams;

pub fn generate_context<S>(request: Request, _state: &S, _path: &str) -> BasicContext {
    let mut ctx = BasicContext::new();
    ctx.params = request.params().clone();
    ctx.request = request;

    ctx
}

#[derive(Default)]
pub struct BasicContext {
    response: Response,
    pub cookies: Vec<Cookie>,
    pub params: Option<HashMap<String, String>>,
    pub query_params: Option<HashMap<String, String>>,
    pub request: Request,
    pub status: u32,
    pub headers: HashMap<String, String>,
}

impl Clone for BasicContext {
    fn clone(&self) -> Self {
        warn!("You should not be calling this method -- it just returns a default context.");
        BasicContext::new()
    }
}

impl BasicContext {
    pub fn new() -> BasicContext {
        let mut ctx = BasicContext {
            response: Response::new(),
            cookies: Vec::new(),
            params: None,
            query_params: None,
            request: Request::new(),
            headers: HashMap::new(),
            status: 200,
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

        format!("{}={}; {}", name, value, pieces.join("; "))
    }
}

impl Context for BasicContext {
    type Response = Response;

    fn get_response(mut self) -> Self::Response {
        self.response.status_code(self.status, "");

        for (key, value) in self.headers {
            self.response.header(&key, &value);
        }

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

impl HasQueryParams for BasicContext {
    fn set_query_params(&mut self, query_params: HashMap<String, String>) {
        self.query_params = Some(query_params);
    }
}

impl HasCookies for BasicContext {
    fn set_cookies(&mut self, cookies: Vec<Cookie>) {
        self.cookies = cookies;
    }

    fn get_cookies(&self) -> Vec<String> {
        self.request
            .headers()
            .get("cookie")
            .map(|v| v.clone())
            .unwrap_or_else(|| vec![])
    }

    fn get_header(&self, key: &str) -> Vec<String> {
        match self.request.headers().get(key) {
            Some(v) => v.clone(),
            None => vec![],
        }
    }
}
