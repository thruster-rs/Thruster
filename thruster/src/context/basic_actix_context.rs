use actix_compat_bytes::Bytes;
use actix_web::http::{HeaderMap, HeaderName};
use http::header::SERVER;
use http::HeaderValue;
use std::collections::HashMap;
use std::str;

use crate::core::context::Context;
use crate::Response;

pub use crate::context::actix_request::ActixRequest;
use crate::middleware::query_params::HasQueryParams;

pub fn generate_context<S>(request: ActixRequest, _state: &S, _path: &str) -> BasicActixContext {
    BasicActixContext::new(request)
}

pub enum SameSite {
    Strict,
    Lax,
}

pub struct CookieOptions {
    pub domain: String,
    pub path: String,
    pub expires: u64,
    pub http_only: bool,
    pub max_age: u64,
    pub secure: bool,
    pub signed: bool,
    pub same_site: SameSite,
}

impl CookieOptions {
    pub fn default() -> CookieOptions {
        CookieOptions {
            domain: "".to_owned(),
            path: "/".to_owned(),
            expires: 0,
            http_only: false,
            max_age: 0,
            secure: false,
            signed: false,
            same_site: SameSite::Strict,
        }
    }
}

pub struct BasicActixContext {
    pub query_params: HashMap<String, String>,
    pub status: u16,
    pub actix_request: Option<ActixRequest>,
    response_body: Bytes,
    headers: HeaderMap,
}

impl Default for BasicActixContext {
    fn default() -> Self {
        BasicActixContext {
            query_params: HashMap::default(),
            status: 0,
            actix_request: None,
            response_body: Bytes::new(),
            headers: HeaderMap::new(),
        }
    }
}

impl Clone for BasicActixContext {
    fn clone(&self) -> Self {
        warn!("You should not be calling this method -- it just returns a default context.");
        BasicActixContext::default()
    }
}

const SERVER_HEADER_NAME: HeaderName = SERVER;
impl BasicActixContext {
    pub fn new(req: ActixRequest) -> BasicActixContext {
        let mut headers = HeaderMap::new();
        headers.insert(SERVER_HEADER_NAME, HeaderValue::from_static("thruster"));

        BasicActixContext {
            query_params: HashMap::new(),
            status: 200,
            actix_request: Some(req),
            response_body: Bytes::new(),
            headers,
        }
    }

    ///
    /// Set the body as a string
    ///
    pub fn body(&mut self, body_string: &str) {
        self.response_body = Bytes::from(body_string.as_bytes().to_vec());
    }

    ///
    /// Get the request body as a string
    ///
    pub async fn get_body(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let request = self.actix_request.as_ref().unwrap();

        let results = String::from_utf8(request.payload.clone()).unwrap();

        Ok(results)
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
            Some(val) => format!(
                "{}, {}",
                val.to_str().unwrap_or(""),
                self.cookify_options(name, value, &options)
            ),
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

        match options.same_site {
            SameSite::Strict => pieces.push("SameSite=Strict".to_owned()),
            SameSite::Lax => pieces.push("SameSite=Lax".to_owned()),
        };

        format!("{}={}; {}", name, value, pieces.join("; "))
    }

    // pub fn set_http2(&mut self) {
    //     self.http_version = hyper::Version::HTTP_2;
    // }

    // pub fn set_http11(&mut self) {
    //     self.http_version = hyper::Version::HTTP_11;
    // }

    // pub fn set_http10(&mut self) {
    //     self.http_version = hyper::Version::HTTP_10;
    // }
}

impl Context for BasicActixContext {
    type Response = Response;

    fn get_response(self) -> Self::Response {
        let mut response = Response::new();

        for (key, value) in self.headers.into_iter() {
            response.header(key.as_str(), value.to_str().unwrap());
        }

        response.body_bytes(&self.response_body);

        response
    }

    fn set_body(&mut self, body: Vec<u8>) {
        self.response_body = Bytes::from(body);
    }

    fn set_body_bytes(&mut self, bytes: bytes::Bytes) {
        self.response_body = Bytes::from(bytes.to_vec());
    }

    fn route(&self) -> &str {
        &self.actix_request.as_ref().unwrap().path
    }

    fn set(&mut self, key: &str, value: &str) {
        self.headers.append(
            HeaderName::from_bytes(key.as_bytes()).unwrap(),
            HeaderValue::from_str(value).unwrap(),
        );
    }

    fn remove(&mut self, key: &str) {
        self.headers.remove(key);
    }

    fn status(&mut self, code: u16) {
        self.status = code;
    }
}

impl HasQueryParams for BasicActixContext {
    fn set_query_params(&mut self, query_params: HashMap<String, String>) {
        self.query_params = query_params;
    }
}
