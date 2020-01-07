use futures::stream::StreamExt;
use std::net::ToSocketAddrs;

use async_trait::async_trait;
use hyper::server::conn::Http;
use hyper::server::Builder;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response};
use std::sync::Arc;
use tokio::net::TcpListener;

use native_tls::Identity;
use thruster_app::app::App;
use thruster_context::basic_hyper_context::HyperRequest;
use thruster_core::context::Context;

use crate::thruster_server::ThrusterServer;

pub struct SSLHyperServer<T: 'static + Context + Send> {
    app: App<HyperRequest, T>,
    cert: Option<Vec<u8>>,
    cert_pass: &'static str,
}

impl<T: 'static + Context + Send> SSLHyperServer<T> {
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
impl<T: Context<Response = Response<Body>> + Send> ThrusterServer for SSLHyperServer<T> {
    type Context = T;
    type Response = Response<Body>;
    type Request = HyperRequest;

    fn new(app: App<Self::Request, T>) -> Self {
        SSLHyperServer {
            app,
            cert: None,
            cert_pass: "",
        }
    }

    async fn build(self, host: &str, port: u16) {
        let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();

        let arc_app = Arc::new(self.app);

        let cert = self.cert.unwrap();
        let cert_pass = self.cert_pass;
        let cert = Identity::from_pkcs12(&cert, cert_pass).expect("Could not decrypt p12 file");
        let tls_acceptor = tokio_tls::TlsAcceptor::from(
            native_tls::TlsAcceptor::builder(cert)
                .build()
                .expect("Could not create TLS acceptor."),
        );
        let _arc_acceptor = Arc::new(tls_acceptor);

        async {
            let service = make_service_fn(|_| {
                let app = arc_app.clone();

                async {
                    Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                        let matched = app.resolve_from_method_and_path(
                            &req.method().to_string(),
                            &req.uri().to_string(),
                        );
                        let req = HyperRequest::new(req);
                        app.resolve(req, matched)
                    }))
                }
            });

            let mut listener = TcpListener::bind(&addr).await.unwrap();
            let incoming = listener.incoming();
            let server = Builder::new(
                hyper::server::accept::from_stream(incoming.filter_map(|socket| {
                    async {
                        match socket {
                            Ok(stream) => match _arc_acceptor.clone().accept(stream).await {
                                Ok(val) => Some(Ok::<_, hyper::Error>(val)),
                                Err(e) => {
                                    println!("TLS error: {}", e);
                                    None
                                }
                            },
                            Err(e) => {
                                println!("TCP socket error: {}", e);
                                None
                            }
                        }
                    }
                })),
                Http::new(),
            )
            .serve(service);

            server.await?;

            Ok::<_, hyper::Error>(())
        }
        .await
        .expect("hyper server failed");
    }
}
