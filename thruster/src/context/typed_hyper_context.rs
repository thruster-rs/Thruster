use async_trait::async_trait;
use bytes::Bytes;
use http::header::{HeaderMap, HeaderName, HeaderValue};
use http::request::Parts;
use hyper::{Body, Response, StatusCode};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::str;

use crate::context::{context_ext::ContextExt, hyper_request::HyperRequest};
use crate::core::context::Context;
use crate::RequestWithParams;

use crate::middleware::cookies::{Cookie, CookieOptions, HasCookies, SameSite};
use crate::middleware::query_params::HasQueryParams;
use crate::parser::tree::Params;

pub struct TypedHyperContext<S: 'static + Send> {
    pub body: Body,
    pub query_params: HashMap<String, String>,
    pub status: u16,
    pub headers: HeaderMap,
    pub hyper_request: Option<HyperRequest>,
    pub extra: S,
    pub cookies: HashMap<String, Cookie>,
    http_version: hyper::Version,
    request_body: Option<Body>,
    request_parts: Option<Parts>,
}

impl<S: 'static + Send + Default> Default for TypedHyperContext<S> {
    fn default() -> Self {
        Self {
            body: Default::default(),
            query_params: Default::default(),
            status: 200,
            headers: Default::default(),
            hyper_request: Default::default(),
            extra: S::default(),
            cookies: Default::default(),
            http_version: Default::default(),
            request_body: Default::default(),
            request_parts: Default::default(),
        }
    }
}

impl<S: 'static + Send> Clone for TypedHyperContext<S> {
    fn clone(&self) -> Self {
        panic!("You should not be calling this method.");
    }
}

impl<S: 'static + Send> TypedHyperContext<S> {
    pub fn new(req: HyperRequest, extra: S) -> TypedHyperContext<S> {
        let mut ctx = TypedHyperContext {
            body: Body::empty(),
            query_params: HashMap::new(),
            headers: HeaderMap::new(),
            status: 200,
            hyper_request: Some(req),
            request_body: None,
            request_parts: None,
            extra,
            http_version: hyper::Version::HTTP_11,
            cookies: HashMap::new(),
        };

        ctx.set("Server", "Thruster");

        ctx
    }

    pub fn new_without_request(extra: S) -> TypedHyperContext<S> {
        let mut ctx = TypedHyperContext {
            body: Body::empty(),
            query_params: HashMap::new(),
            headers: HeaderMap::new(),
            status: 200,
            hyper_request: None,
            request_body: None,
            request_parts: None,
            extra,
            http_version: hyper::Version::HTTP_11,
            cookies: HashMap::new(),
        };

        ctx.set("Server", "Thruster");

        ctx
    }

    ///
    /// Set the body as a string
    ///
    pub fn body(&mut self, body_string: &str) {
        self.body = Body::from(body_string.to_string());
    }

    ///
    /// Get the body as a string
    ///
    pub async fn get_body(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let body = self.request_body.take().unwrap_or_else(|| {
            self.into_owned_request();
            self.request_body.take().unwrap()
        });

        Ok(String::from_utf8(
            hyper::body::to_bytes(body).await?.to_vec(),
        )?)
    }

    pub fn parts(&self) -> &Parts {
        self.request_parts
            .as_ref()
            .expect("Must call `to_owned_request` prior to getting parts")
    }

    pub fn into_owned_request(&mut self) {
        let hyper_request = self.hyper_request.take().expect(
            "`hyper_request` is None! That means `to_owned_request` has already been called",
        );
        let (parts, body) = hyper_request.request.into_parts();

        self.hyper_request = None;
        self.request_body = Some(body);
        self.request_parts = Some(parts);
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
            Some(SameSite::Strict) => pieces.push("SameSite=Strict".to_owned()),
            Some(SameSite::Lax) => pieces.push("SameSite=Lax".to_owned()),
            None => (),
        };

        format!("{}={}; {}", name, value, pieces.join("; "))
    }

    pub fn set_http2(&mut self) {
        self.http_version = hyper::Version::HTTP_2;
    }

    pub fn set_http11(&mut self) {
        self.http_version = hyper::Version::HTTP_11;
    }

    pub fn set_http10(&mut self) {
        self.http_version = hyper::Version::HTTP_10;
    }
}

impl<S: 'static + Send> Context for TypedHyperContext<S> {
    type Response = Response<Body>;

    fn get_response(self) -> Self::Response {
        let mut response = Response::new(self.body);

        *response.status_mut() = StatusCode::from_u16(self.status).unwrap();
        *response.headers_mut() = self.headers;
        *response.version_mut() = self.http_version;

        response
    }

    fn set_body(&mut self, body: Vec<u8>) {
        self.body = Body::from(body);
    }

    fn set_body_bytes(&mut self, bytes: Bytes) {
        self.body = Body::from(bytes);
    }

    fn route(&self) -> &str {
        let uri = self.hyper_request.as_ref().unwrap().request.uri();

        match uri.path_and_query() {
            Some(val) => val.as_str(),
            None => uri.path(),
        }
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

impl<S: 'static + Send> HasQueryParams for TypedHyperContext<S> {
    fn set_query_params(&mut self, query_params: HashMap<String, String>) {
        self.query_params = query_params;
    }
}

impl<S: 'static + Send> HasCookies for TypedHyperContext<S> {
    fn set_cookies(&mut self, cookies: Vec<Cookie>) {
        self.cookies.clear();
        for cookie in cookies {
            self.cookies.insert(cookie.key.clone(), cookie);
        }
    }

    fn get_cookies(&self) -> Vec<String> {
        let res: Vec<String> = self
            .headers
            .get_all("cookie")
            .iter()
            .map(|v| v.to_str().unwrap_or("").to_string())
            .collect();

        res
    }

    fn get_header(&self, key: &str) -> Vec<String> {
        self.hyper_request
            .as_ref()
            .unwrap()
            .request
            .headers()
            .get_all(key)
            .iter()
            .map(|v| v.to_str().unwrap_or("").to_string())
            .collect()
    }
}

#[async_trait]
impl<S: 'static + Send> ContextExt for TypedHyperContext<S> {
    fn get_params(&self) -> &Params {
        self.hyper_request.as_ref().unwrap().get_params()
    }

    fn json<T: serde::Serialize>(&mut self, body: &T) -> Result<(), Box<dyn std::error::Error>> {
        self.set("Content-Type", "application/json");
        self.body = Body::from(serde_json::to_vec::<T>(body)?);

        Ok(())
    }

    async fn get_json<T: DeserializeOwned>(&mut self) -> Result<T, Box<dyn std::error::Error>> {
        let body = self.request_body.take().unwrap_or_else(|| {
            self.into_owned_request();
            self.request_body.take().unwrap()
        });

        serde_json::from_slice::<T>(&hyper::body::to_bytes(body).await?)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    fn get_req_header<'a>(&'a self, header: &str) -> Option<&'a str> {
        self.parts()
            .headers
            .get(header)
            .and_then(|v| v.to_str().ok())
    }
}
