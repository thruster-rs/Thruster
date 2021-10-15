use crate::core::context::Context;
use crate::ReusableBoxFuture;
use crate::{app::App, core::request::ThrusterRequest};

pub trait ThrusterServer {
    type Context: Context + Clone + Send + Sync;
    type Response;
    type Request: ThrusterRequest;
    type State: Send;

    fn new(_: App<Self::Request, Self::Context, Self::State>) -> Self;
    fn build(self, host: &str, port: u16) -> ReusableBoxFuture<()>;
    fn start(self, host: &str, port: u16)
    where
        Self: Sized,
    {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build runtime")
            // tokio::runtime::Runtime::new()
            //     .unwrap()
            .block_on(self.build(host, port))
    }
}
