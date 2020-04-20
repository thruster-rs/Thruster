use std::error::Error;
use std::net::ToSocketAddrs;
use std::sync::Arc;

use async_trait::async_trait;
use futures::sink::SinkExt;
use native_tls::Identity;
use tokio::net::{TcpListener, TcpStream};
use tokio::stream::StreamExt;
use tokio_util::codec::Framed;

use crate::app::App;
use crate::core::context::Context;
use crate::core::http::Http;
use crate::core::request::Request;
use crate::core::response::Response;

use crate::server::ThrusterServer;

pub struct SSLServer<T: 'static + Context<Response = Response> + Send, S: Send> {
    app: App<Request, T, S>,
    cert: Option<Vec<u8>>,
    cert_pass: &'static str,
}

impl<T: 'static + Context<Response = Response> + Send, S: Send> SSLServer<T, S> {
    ///
    /// Sets the cert on the server
    ///
    pub fn cert(&mut self, cert: Vec<u8>) {
        self.cert = Some(cert);
    }

    pub fn cert_pass(&mut self, cert_pass: &'static str) {
        self.cert_pass = cert_pass;
    }
}

#[async_trait]
impl<T: Context<Response = Response> + Send, S: 'static + Send + Sync> ThrusterServer for SSLServer<T, S> {
    type Context = T;
    type Response = Response;
    type Request = Request;
    type State = S;

    fn new(app: App<Self::Request, T, Self::State>) -> Self {
        SSLServer {
            app,
            cert: None,
            cert_pass: "",
        }
    }

    ///
    /// Alias for start_work_stealing_optimized
    ///
    async fn build(mut self, host: &str, port: u16) {
        if self.cert.is_none() {
            panic!("Cert is required to be set via SSLServer::cert() before starting the server");
        }

        let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();

        self.app._route_parser.optimize();

        let mut listener = TcpListener::bind(&addr).await.unwrap();
        let mut incoming = listener.incoming();
        let arc_app = Arc::new(self.app);

        let cert = self.cert.unwrap().clone();
        let cert_pass = self.cert_pass;
        let cert = Identity::from_pkcs12(&cert, cert_pass).expect("Could not decrypt p12 file");
        let tls_acceptor = tokio_tls::TlsAcceptor::from(
            native_tls::TlsAcceptor::builder(cert)
                .build()
                .expect("Could not create TLS acceptor."),
        );
        let arc_acceptor = Arc::new(tls_acceptor);

        while let Some(Ok(stream)) = incoming.next().await {
            let cloned_app = arc_app.clone();
            let cloned_tls_acceptor = arc_acceptor.clone();
            tokio::spawn(async move {
                if let Err(e) = process(cloned_app, cloned_tls_acceptor, stream).await {
                    println!("failed to process connection; error = {}", e);
                }
            });
        }

        // TODO(trezm): Add an argument here for the tls_acceptor to be passed in.
        async fn process<T: Context<Response = Response> + Send, S: Send>(
            app: Arc<App<Request, T, S>>,
            tls_acceptor: Arc<tokio_tls::TlsAcceptor>,
            socket: TcpStream,
        ) -> Result<(), Box<dyn Error>> {
            let tls = tls_acceptor.accept(socket).await?;
            let mut framed = Framed::new(tls, Http);

            while let Some(request) = framed.next().await {
                match request {
                    Ok(request) => {
                        let matched =
                            app.resolve_from_method_and_path(request.method(), request.path());
                        let response = app.resolve(request, matched).await?;
                        framed.send(response).await?;
                    }
                    Err(e) => return Err(e.into()),
                }
            }

            Ok(())
        }
    }
}
