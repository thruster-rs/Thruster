use thruster_proc::middleware_fn;

use crate::core::context::Context;
use crate::core::{MiddlewareNext, MiddlewareResult};

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
    fn get_cookies(&self) -> Vec<String>;
    fn get_header(&self, key: &str) -> Vec<String>;
}

#[middleware_fn(_internal)]
pub async fn cookies<T: 'static + Context + HasCookies + Send + Sync>(
    mut context: T,
    next: MiddlewareNext<T>,
) -> MiddlewareResult<T> {
    let mut cookies = Vec::new();

    {
        for cookie_string in context.get_header("cookie") {
            cookies.append(&mut parse_string(&cookie_string));
        }
    }

    context.set_cookies(cookies);

    next(context).await
}

fn parse_string(string: &str) -> Vec<Cookie> {
    string
        .split(';')
        .map(|pair| {
            let mut key_pair = pair.split('=');

            let key = key_pair.next().unwrap_or("").to_owned().trim().to_string();
            let value = key_pair.next().unwrap_or("").to_owned().trim().to_string();
            let options = CookieOptions::default();

            Cookie {
                key,
                value,
                options,
            }
        })
        .collect()
}
