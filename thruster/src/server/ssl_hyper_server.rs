use futures::stream::StreamExt;
use std::net::ToSocketAddrs;

use async_trait::async_trait;
use hyper::server::conn::Http;
use hyper::server::Builder;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response};
use native_tls::Identity;
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::app::App;
use crate::context::basic_hyper_context::HyperRequest;
use crate::core::context::Context;

use crate::server::ThrusterServer;

pub struct SSLHyperServer<T: 'static + Context + Send, S: Send> {
    app: App<HyperRequest, T, S>,
    cert: Option<Vec<u8>>,
    cert_pass: &'static str,
}

impl<T: 'static + Context + Send, S: Send> SSLHyperServer<T, S> {
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
impl<T: Context<Response = Response<Body>> + Send, S: 'static + Send + Sync> ThrusterServer for SSLHyperServer<T, S> {
    type Context = T;
    type Response = Response<Body>;
    type Request = HyperRequest;
    type State = S;

    fn new(app: App<Self::Request, T, Self::State>) -> Self {
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
        let arc_acceptor = Arc::new(tls_acceptor);

        let service = make_service_fn(|_| {
            let app = arc_app.clone();

            async {
                Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                    let matched = app.resolve_from_method_and_path(
                        &req.method().to_string(),
                        &req.uri().path_and_query().unwrap().to_string(),
                    );
                    let req = HyperRequest::new(req);
                    app.resolve(req, matched)
                }))
            }
        });

        let mut listener = TcpListener::bind(&addr).await.unwrap();
        let incoming = listener.incoming();

        async {
            let server = Builder::new(
                hyper::server::accept::from_stream(incoming.filter_map(|socket| async {
                    match socket {
                        Ok(stream) => {
                            let timed_out = arc_acceptor.clone().accept(stream).await;

                            match timed_out {
                                Ok(val) => Some(Ok::<_, std::io::Error>(val)),
                                Err(e) => {
                                    println!("TLS error: {}", e);
                                    None
                                }
                            }
                        }
                        Err(e) => {
                            println!("TCP socket error: {}", e);
                            None
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
