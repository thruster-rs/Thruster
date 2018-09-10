use std::collections::HashMap;

use context::Context;
use response::Response;
use request::Request;
use builtins::retains_request::RetainsRequest;
use builtins::query_params::HasQueryParams;

pub enum SameSite {
  Strict,
  Lax
}

pub struct CookieOptions {
  pub domain: String,
  pub path: String,
  pub expires: u64,
  pub http_only: bool,
  pub max_age: u64,
  pub secure: bool,
  pub signed: bool,
  pub same_site: SameSite
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
      same_site: SameSite::Strict
    }
  }
}

pub fn generate_context(request: Request) -> BasicContext {
  let mut ctx = BasicContext::new();
  ctx.params = request.params().clone();
  ctx.request = request;

  ctx
}

pub struct BasicContext {
  pub body: String,
  pub params: HashMap<String, String>,
  pub query_params: HashMap<String, String>,
  pub request: Request,
  pub status: u32,
  pub headers: HashMap<String, String>
}

impl BasicContext {
  pub fn new() -> BasicContext {
    BasicContext {
      body: "".to_owned(),
      params: HashMap::new(),
      query_params: HashMap::new(),
      request: Request::new(),
      headers: HashMap::new(),
      status: 200
    }
  }

  ///
  /// Set a header on the response
  ///
  pub fn set(&mut self, key: &str, value: &str) {
    self.headers.insert(key.to_owned(), value.to_owned());
  }

  ///
  /// Remove a header on the response
  ///
  pub fn remove(&mut self, key: &str) {
    self.headers.remove(key);
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
  pub fn cookie(&mut self, name: &str, value: &str, options: CookieOptions) {
    let cookie_value = match self.headers.get("Set-Cookie") {
      Some(val) => format!("{}, {}", val, self.cookify_options(name, value, options)),
      None => self.cookify_options(name, value, options)
    };

    self.set("Set-Cookie", &cookie_value);
  }

  fn cookify_options(&self, name: &str, value: &str, options: CookieOptions) -> String {
    let mut pieces = vec![format!("Path={}", options.path)];

    if options.expires > 0 {
      pieces.push(format!("Expires={}", options.expires));
    }

    if options.max_age > 0 {
      pieces.push(format!("Max-Age={}", options.max_age));
    }

    if options.domain.len() > 0 {
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
      SameSite::Lax => pieces.push("SameSite=Lax".to_owned())
    };

    format!("{}={}; {}", name, value, pieces.join(", "))
  }
}

impl Context for BasicContext {
  fn get_response(self) -> Response {
    let mut response = Response::new();

    response.body(&self.body);

    for (key, val) in self.headers {
      response.header(&key, &val);
    }

    response.status_code(self.status, "");

    response
  }

  fn set_body(&mut self, body: String) {
    self.body = body;
  }
}

impl RetainsRequest for BasicContext {
  fn get_request<'a>(&'a self) -> &'a Request {
    &self.request
  }
}

impl HasQueryParams for BasicContext {
  fn set_query_params(&mut self, query_params: HashMap<String, String>) {
    self.query_params = query_params;
  }
}
