use bytes::Bytes;
use futures::stream::StreamExt;
use http::header::{HeaderMap, HeaderName, HeaderValue, SERVER};
use http::request::Parts;
use hyper::{Body, Error, Response, StatusCode};
use std::collections::HashMap;
use std::convert::TryInto;
use std::str;
use thruster_proc::middleware_fn;

pub use crate::context::hyper_request::HyperRequest;
use crate::core::context::Context;
use crate::core::{MiddlewareNext, MiddlewareResult};
use crate::middleware::query_params::HasQueryParams;

pub fn generate_context<S>(request: HyperRequest, _state: &S, _path: &str) -> BasicHyperContext {
    BasicHyperContext::new(request)
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

///
/// to_owned_request is needed in order to consume the body of a hyper
/// context. This request moves ownership of the parts and the body of
/// the raw hyper request into the context itself for easier access.
///
#[middleware_fn(_internal)]
pub async fn to_owned_request(
    context: BasicHyperContext,
    next: MiddlewareNext<BasicHyperContext>,
) -> MiddlewareResult<BasicHyperContext> {
    let context = next(context.to_owned_request()).await?;

    Ok(context)
}

#[derive(Default)]
pub struct BasicHyperContext {
    pub body: Body,
    pub query_params: HashMap<String, String>,
    pub status: u16,
    pub params: Option<HashMap<String, String>>,
    pub hyper_request: Option<HyperRequest>,
    request_body: Option<Body>,
    request_parts: Option<Parts>,
    http_version: hyper::Version,
    headers: HeaderMap,
}

const SERVER_HEADER_NAME: HeaderName = SERVER;
impl BasicHyperContext {
    pub fn new(req: HyperRequest) -> BasicHyperContext {
        let params = req.params.clone();
        let mut headers = HeaderMap::new();
        headers.insert(SERVER_HEADER_NAME, HeaderValue::from_static("thruster"));

        BasicHyperContext {
            body: Body::empty(),
            query_params: HashMap::new(),
            status: 200,
            params,
            hyper_request: Some(req),
            request_body: None,
            request_parts: None,
            http_version: hyper::Version::HTTP_11,
            headers,
        }
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
    pub async fn get_body(self) -> Result<(String, BasicHyperContext), Error> {
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
            BasicHyperContext {
                body: ctx.body,
                query_params: ctx.query_params,
                status: ctx.status,
                params: ctx.params,
                hyper_request: ctx.hyper_request,
                request_body: Some(Body::empty()),
                request_parts: ctx.request_parts,
                http_version: ctx.http_version,
                headers: ctx.headers,
            },
        ))
    }

    pub fn parts(&self) -> &Parts {
        self.request_parts
            .as_ref()
            .expect("Must call `to_owned_request` prior to getting parts")
    }

    pub fn to_owned_request(self) -> BasicHyperContext {
        let hyper_request = self.hyper_request.expect(
            "`hyper_request` is None! That means `to_owned_request` has already been called",
        );
        let (parts, body) = hyper_request.request.into_parts();

        BasicHyperContext {
            body: self.body,
            query_params: self.query_params,
            status: self.status,
            params: hyper_request.params,
            hyper_request: None,
            request_body: Some(body),
            request_parts: Some(parts),
            http_version: self.http_version,
            headers: self.headers,
        }
    }

    ///
    /// Set the response status code
    ///
    pub fn status(&mut self, code: u32) {
        self.status = code.try_into().unwrap();
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
            SameSite::Strict => pieces.push("SameSite=Strict".to_owned()),
            SameSite::Lax => pieces.push("SameSite=Lax".to_owned()),
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

impl Context for BasicHyperContext {
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

impl HasQueryParams for BasicHyperContext {
    fn set_query_params(&mut self, query_params: HashMap<String, String>) {
        self.query_params = query_params;
    }
}
