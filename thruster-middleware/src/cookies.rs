use std::collections::HashMap;

use thruster_core::context::Context;
use thruster_core::middleware::MiddlewareReturnValue;

#[derive(Debug)]
pub struct Cookie {
    pub key: String,
    pub value: String,
    pub options: CookieOptions,
}

#[derive(Debug, PartialEq)]
pub enum SameSite {
    Strict,
    Lax,
}

#[derive(Debug)]
pub struct CookieOptions {
    pub domain: String,
    pub path: String,
    pub expires: u64,
    pub http_only: bool,
    pub max_age: u64,
    pub secure: bool,
    pub signed: bool,
    pub same_site: Option<SameSite>,
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
            same_site: None,
        }
    }
}

pub trait HasCookies {
    fn set_cookies(&mut self, cookies: Vec<Cookie>);
    fn headers(&self) -> HashMap<String, String>;
}

pub fn cookies<T: 'static + Context + HasCookies + Send>(
    mut context: T,
    next: impl Fn(T) -> MiddlewareReturnValue<T> + Send,
) -> MiddlewareReturnValue<T> {
    let mut cookies = Vec::new();

    {
        let headers: HashMap<String, String> = context.headers();

        if let Some(val) = headers.get("set-cookie") {
            for cookie_string in val.split(',') {
                cookies.push(parse_string(cookie_string));
            }
        }
    }

    context.set_cookies(cookies);

    next(context)
}

fn parse_string(string: &str) -> Cookie {
    let mut options = CookieOptions::default();

    let mut pieces = string.split(';');

    let mut key_pair = pieces.next().unwrap().split('=');

    let key = key_pair.next().unwrap_or("").to_owned();
    let value = key_pair.next().unwrap_or("").to_owned();

    for option in pieces {
        let mut option_key_pair = option.split('=');

        if let Some(option_key) = option_key_pair.next() {
            match option_key.to_lowercase().trim() {
                "expires" => {
                    options.expires = option_key_pair
                        .next()
                        .unwrap_or("0")
                        .parse::<u64>()
                        .unwrap_or(0)
                }
                "max-age" => {
                    options.max_age = option_key_pair
                        .next()
                        .unwrap_or("0")
                        .parse::<u64>()
                        .unwrap_or(0)
                }
                "domain" => options.domain = option_key_pair.next().unwrap_or("").to_owned(),
                "path" => options.path = option_key_pair.next().unwrap_or("").to_owned(),
                "secure" => options.secure = true,
                "httponly" => options.http_only = true,
                "samesite" => {
                    if let Some(same_site_value) = option_key_pair.next() {
                        match same_site_value.to_lowercase().as_ref() {
                            "strict" => options.same_site = Some(SameSite::Strict),
                            "lax" => options.same_site = Some(SameSite::Lax),
                            _ => (),
                        };
                    }
                }
                _ => (),
            };
        }
    }

    Cookie {
        key,
        value,
        options,
    }
}
