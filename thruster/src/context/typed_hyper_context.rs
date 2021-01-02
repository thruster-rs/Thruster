use bytes::Bytes;
use futures::stream::StreamExt;
use http::header::{HeaderMap, HeaderName, HeaderValue};
use http::request::Parts;
use hyper::{Body, Error, Response, StatusCode};
use std::collections::HashMap;
use std::convert::TryInto;
use std::str;
use thruster_proc::middleware_fn;

use crate::context::hyper_request::HyperRequest;
use crate::core::context::Context;
use crate::core::{MiddlewareNext, MiddlewareResult};
use crate::middleware::cookies::{Cookie, CookieOptions, HasCookies, SameSite};
use crate::middleware::query_params::HasQueryParams;

///
/// to_owned_request is needed in order to consume the body of a hyper
/// context. This request moves ownership of the parts and the body of
/// the raw hyper request into the context itself for easier access.
///
#[middleware_fn(_internal)]
pub async fn to_owned_request<T: 'static + Send>(
    context: TypedHyperContext<T>,
    next: MiddlewareNext<TypedHyperContext<T>>,
) -> MiddlewareResult<TypedHyperContext<T>> {
    let context = next(context.to_owned_request()).await?;

    Ok(context)
}

#[derive(Default)]
pub struct TypedHyperContext<S> {
    pub body: Body,
    pub query_params: HashMap<String, String>,
    pub status: u16,
    pub headers: HeaderMap,
    pub params: Option<HashMap<String, String>>,
    pub hyper_request: Option<HyperRequest>,
    pub extra: S,
    pub cookies: HashMap<String, Cookie>,
    http_version: hyper::Version,
    request_body: Option<Body>,
    request_parts: Option<Parts>,
}

impl<S> TypedHyperContext<S> {
    pub fn new(req: HyperRequest, extra: S) -> TypedHyperContext<S> {
        let params = req.params.clone();
        let mut ctx = TypedHyperContext {
            body: Body::empty(),
            query_params: HashMap::new(),
            headers: HeaderMap::new(),
            status: 200,
            params,
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
        let params = None;
        let mut ctx = TypedHyperContext {
            body: Body::empty(),
            query_params: HashMap::new(),
            headers: HeaderMap::new(),
            status: 200,
            params,
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
    pub async fn get_body(self) -> Result<(String, TypedHyperContext<S>), Error> {
        let ctx = match self.request_body {
            Some(_) => self,
            None => self.to_owned_request(),
        };

        let mut results = "".to_string();
        let mut unwrapped_body = ctx.request_body.unwrap();
        while let Some(chunk) = unwrapped_body.next().await {
            // TODO(trezm): Dollars to donuts this is pretty slow -- could make it faster with a
            // mutable byte buffer.
            results = format!("{}{}", results, String::from_utf8_lossy(chunk?.as_ref()));
        }

        Ok((
            results,
            TypedHyperContext {
                body: ctx.body,
                query_params: ctx.query_params,
                headers: ctx.headers,
                status: ctx.status,
                params: ctx.params,
                hyper_request: ctx.hyper_request,
                request_body: Some(Body::empty()),
                request_parts: ctx.request_parts,
                extra: ctx.extra,
                http_version: ctx.http_version,
                cookies: ctx.cookies,
            },
        ))
    }

    pub fn parts(&self) -> &Parts {
        self.request_parts
            .as_ref()
            .expect("Must call `to_owned_request` prior to getting parts")
    }

    pub fn to_owned_request(self) -> TypedHyperContext<S> {
        match self.hyper_request {
            Some(hyper_request) => {
                let (parts, body) = hyper_request.request.into_parts();

                TypedHyperContext {
                    body: self.body,
                    query_params: self.query_params,
                    headers: self.headers,
                    status: self.status,
                    params: hyper_request.params,
                    hyper_request: None,
                    request_body: Some(body),
                    request_parts: Some(parts),
                    extra: self.extra,
                    http_version: self.http_version,
                    cookies: self.cookies,
                }
            }
            None => self,
        }
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
                val.to_str().unwrap_or_else(|_| ""),
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

impl<S> Context for TypedHyperContext<S> {
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

impl<S> HasQueryParams for TypedHyperContext<S> {
    fn set_query_params(&mut self, query_params: HashMap<String, String>) {
        self.query_params = query_params;
    }
}

impl<S> HasCookies for TypedHyperContext<S> {
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
            .map(|v| v.to_str().unwrap_or_else(|_| "").to_string())
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
            .map(|v| v.to_str().unwrap_or_else(|_| "").to_string())
            .collect()
    }
}
