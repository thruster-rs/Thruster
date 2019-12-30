use async_trait::async_trait;
use thruster_app::app::App;
use thruster_core::context::Context;
use thruster_core::request::RequestWithParams;

#[async_trait]
pub trait ThrusterServer {
    type Context: Context + Send;
    type Response: Send;
    type Request: RequestWithParams + Send;

    fn new(_: App<Self::Request, Self::Context>) -> Self;
    async fn build(self, host: &str, port: u16);
    fn start(self, host: &str, port: u16)
    where
        Self: Sized,
    {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(self.build(host, port))
    }
}
