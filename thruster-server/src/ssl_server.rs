use std::error::Error;
use std::net::ToSocketAddrs;
use std::sync::Arc;

use native_tls;
use native_tls::Identity;
use tokio;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio_tls;
use tokio_util::codec::Framed;

use thruster_app::app::App;
use thruster_core::context::Context;
use thruster_core::http::Http;
use thruster_core::request::Request;
use thruster_core::response::Response;

use crate::thruster_server::ThrusterServer;

pub struct SSLServer<T: 'static + Context<Response = Response> + Send> {
    app: App<Request, T>,
    cert: Option<Vec<u8>>,
    cert_pass: &'static str,
}

impl<T: 'static + Context<Response = Response> + Send> SSLServer<T> {
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

impl<T: Context<Response = Response> + Send> ThrusterServer for SSLServer<T> {
    type Context = T;
    type Response = Response;
    type Request = Request;

    fn new(app: App<Self::Request, T>) -> Self {
        SSLServer {
            app,
            cert: None,
            cert_pass: "",
        }
    }

    ///
    /// Alias for start_work_stealing_optimized
    ///
    fn start(mut self, host: &str, port: u16) {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if self.cert.is_none() {
                panic!(
                    "Cert is required to be set via SSLServer::cert() before starting the server"
                );
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
        });

        // TODO(trezm): Add an argument here for the tls_acceptor to be passed in.
        async fn process<T: Context<Response = Response> + Send>(
            app: Arc<App<Request, T>>,
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
