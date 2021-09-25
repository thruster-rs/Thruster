use futures::future;
use futures::stream::StreamExt;
use futures::FutureExt;
use std::io::{self, BufReader};
use std::net::ToSocketAddrs;
use tokio::time::{timeout, Duration};
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::sync::ReusableBoxFuture;

use async_trait::async_trait;
use hyper::server::conn::Http;
use hyper::{Body, Response};
use log::error;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::rustls::internal::pemfile::{certs, pkcs8_private_keys};
use tokio_rustls::rustls::{NoClientAuth, ServerConfig};
use tokio_rustls::TlsAcceptor;

use crate::app::App;
use crate::context::basic_hyper_context::HyperRequest;
use crate::core::context::Context;

use crate::hyper_server::HyperService;
use crate::server::ThrusterServer;

/// Fake certs generated using
/// openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 3650 -nodes
pub struct SSLHyperServer<T: 'static + Context + Clone + Send + Sync, S: Send> {
    app: App<HyperRequest, T, S>,
    cert: Option<Vec<u8>>,
    key: Option<Vec<u8>>,
    tls_acceptor: Option<Arc<TlsAcceptor>>,
}

impl<T: 'static + Context + Clone + Send + Sync, S: Send> SSLHyperServer<T, S> {
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
impl<T: Context<Response = Response<Body>> + Clone + Send + Sync, S: 'static + Send + Sync>
    ThrusterServer for SSLHyperServer<T, S>
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
            tls_acceptor: None,
        }
    }

    fn build(mut self, host: &str, port: u16) -> ReusableBoxFuture<()> {
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

        self.tls_acceptor = Some(Arc::new(TlsAcceptor::from(Arc::new(config))));

        let arc_acceptor = self.tls_acceptor.as_ref().unwrap().clone();
        let listener_fut = TcpListener::bind(addr).then(move |listener| {
            let arc_acceptor = arc_acceptor.clone();

            let stream = TcpListenerStream::new(listener.unwrap());

            let mut hyper_stream = stream.filter_map(move |socket| match socket {
                Ok(stream) => {
                    let acceptor = arc_acceptor.clone();

                    let timed_out_fut = acceptor.accept(stream).map(|timed_out| match timed_out {
                        Ok(val) => Some(Ok::<_, std::io::Error>(val)),
                        Err(e) => {
                            error!("TLS error: {}", e);
                            None
                        }
                    });

                    ReusableBoxFuture::new(timed_out_fut)
                }
                Err(e) => {
                    error!("TCP socket error: {}", e);
                    ReusableBoxFuture::new(future::ready(None))
                }
            });

            async move {
                loop {
                    let stream = match hyper_stream.next().await {
                        Some(Ok(val)) => val,
                        _ => break,
                    };

                    // let ip = stream.peer_addr().map(|v| v.ip()).ok();
                    let arc_app = arc_app.clone();
                    let connection_timeout = arc_app.connection_timeout;

                    tokio::spawn(async move {
                        let mut http_future = Http::new().serve_connection(
                            stream,
                            HyperService::<T, S> {
                                ip: None,
                                app: arc_app,
                            },
                        );

                        let _res =
                            timeout(Duration::from_millis(connection_timeout), &mut http_future)
                                .await;
                    });
                }
            }
            // Builder::new(hyper_stream, Http::new()).serve(service)
        });

        ReusableBoxFuture::new(listener_fut)
    }
}
