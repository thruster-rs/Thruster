use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

use crate::parser::tree::Params;

#[async_trait]
pub trait ContextExt {
    /// Get route params, e.g. /users/:id
    fn get_params<'a>(&'a self) -> &'a Params;

    /// Set the response as JSON.
    ///
    /// This method both sets the Content-Type header as well as the body as JSON.
    fn json<T: Serialize>(&mut self, body: &T) -> Result<(), Box<dyn std::error::Error>>;

    /// Gets the request body as JSON.
    async fn get_json<T: DeserializeOwned>(&mut self) -> Result<T, Box<dyn std::error::Error>>;

    fn get_req_header<'a>(&'a self, header: &str) -> Option<&'a str>;
}
