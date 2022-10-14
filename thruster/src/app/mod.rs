mod thruster_app;

#[cfg(not(feature = "hyper_server"))]
pub mod testing_async;

#[cfg(feature = "hyper_server")]
mod testing_hyper_async;

#[cfg(feature = "hyper_server")]
pub mod testing_async {
    pub use super::testing_hyper_async::*;
}

use async_trait::async_trait;
pub use httparse::Header;
use hyper::Body;
pub use thruster_app::*;

use self::testing_async::TestResponse;

#[async_trait]
pub trait Testable {
    async fn get(
        &self,
        route: &str,
        headers: Vec<(String, String)>,
    ) -> Result<TestResponse, Box<dyn std::error::Error>>;
    async fn option(
        &self,
        route: &str,
        headers: Vec<(String, String)>,
    ) -> Result<TestResponse, Box<dyn std::error::Error>>;
    async fn post(
        &self,
        route: &str,
        headers: Vec<(String, String)>,
        body: Body,
    ) -> Result<TestResponse, Box<dyn std::error::Error>>;
    async fn put(
        &self,
        route: &str,
        headers: Vec<(String, String)>,
        body: Body,
    ) -> Result<TestResponse, Box<dyn std::error::Error>>;
    async fn delete(
        &self,
        route: &str,
        headers: Vec<(String, String)>,
    ) -> Result<TestResponse, Box<dyn std::error::Error>>;
    async fn patch(
        &self,
        route: &str,
        headers: Vec<(String, String)>,
        body: Body,
    ) -> Result<TestResponse, Box<dyn std::error::Error>>;
}
