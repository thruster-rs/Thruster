use futures::stream::StreamExt;
use std::io::{self, BufReader};
use std::net::ToSocketAddrs;

use async_trait::async_trait;
use hyper::server::conn::Http;
use hyper::server::Builder;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response};
use log::error;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::rustls::internal::pemfile::{certs, pkcs8_private_keys};
use tokio_rustls::rustls::{NoClientAuth, ServerConfig};
use tokio_rustls::TlsAcceptor;

use crate::app::App;
use crate::context::basic_hyper_context::HyperRequest;
use crate::core::context::Context;

use crate::server::ThrusterServer;

pub struct SSLHyperServer<T: 'static + Context + Send + Sync, S: Send> {
    app: App<HyperRequest, T, S>,
    cert: Option<Vec<u8>>,
    key: Option<Vec<u8>>,
}

impl<T: 'static + Context + Send + Sync, S: Send> SSLHyperServer<T, S> {
    ///
    /// Sets the cert on the server
    ///
    pub fn cert(&mut self, cert: Vec<u8>) {
        self.cert = Some(cert);
    }

    /// Sets the key for the server.
    pub fn key(&mut self, key: Vec<u8>) {
        self.key = Some(key);
    }
}

#[async_trait]
impl<T: Context<Response = Response<Body>> + Send + Sync, S: 'static + Send> ThrusterServer
    for SSLHyperServer<T, S>
{
    type Context = T;
    type Response = Response<Body>;
    type Request = HyperRequest;
    type State = S;

    fn new(mut app: App<Self::Request, T, Self::State>) -> Self {
        app = app.commit();

        SSLHyperServer {
            app,
            cert: None,
            key: None,
        }
    }

    async fn build(self, host: &str, port: u16) {
        let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();

        let arc_app = Arc::new(self.app);

        let cert_u8: &[u8] = &self.cert.unwrap();
        let key_u8: &[u8] = &self.key.unwrap();
        let certs = certs(&mut BufReader::new(cert_u8))
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cert"))
            .unwrap();
        let mut keys = pkcs8_private_keys(&mut BufReader::new(key_u8))
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))
            .unwrap();
        let mut config = ServerConfig::new(NoClientAuth::new());
        config
            .set_single_cert(certs, keys.remove(0))
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))
            .unwrap();
        let tls_acceptor = TlsAcceptor::from(Arc::new(config));
        // let cert = self.cert.unwrap();
        // let cert_pass = self.cert_pass;
        // let cert = Identity::from_pkcs12(&cert, cert_pass).expect("Could not decrypt p12 file");
        // let tls_acceptor = tokio_native_tls::TlsAcceptor::from(
        //     native_tls::TlsAcceptor::builder(cert)
        //         .build()
        //         .expect("Could not create TLS acceptor."),
        // );

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
                // Why did we use a filter_map here? Basically because I was too
                // lazy to implement and wrapper and Accept trait for that class
                // in order to wrap the TlsAcceptor correctly. This probably has
                // a performance hit and should be fixed someday.
                hyper::server::accept::from_stream(incoming.filter_map(|socket| async {
                    match socket {
                        Ok(stream) => {
                            let timed_out = arc_acceptor.clone().accept(stream).await;

                            match timed_out {
                                Ok(val) => Some(Ok::<_, std::io::Error>(val)),
                                Err(e) => {
                                    error!("TLS error: {}", e);
                                    None
                                }
                            }
                        }
                        Err(e) => {
                            error!("TCP socket error: {}", e);
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
