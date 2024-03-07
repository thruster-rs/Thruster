use async_trait::async_trait;
use futures::future;
use futures::stream::StreamExt;
use futures::FutureExt;
use hyper::server::conn::Http;
use hyper::{Body, Response};
use log::error;
use pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::io::{self, BufReader};
use std::net::ToSocketAddrs;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::time::{timeout, Duration};
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::sync::ReusableBoxFuture;

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
    upgrade: bool,
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
            upgrade: true,
        }
    }

    fn build(mut self, host: &str, port: u16) -> ReusableBoxFuture<()> {
        let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();
        let upgrade = self.upgrade;
        let arc_app = Arc::new(self.app);

        let cert_u8: &[u8] = &self.cert.unwrap();
        let key_u8: &[u8] = &self.key.unwrap();
        let certs = certs(&mut BufReader::new(cert_u8))
            .expect("Could not read certs passed in")
            .into_iter()
            .map(Into::into)
            .collect();
        let mut key: PrivateKeyDer<'static> = pkcs8_private_keys(&mut BufReader::new(key_u8))
            .expect("Could not read private keys passed in")
            .into_iter()
            .next()
            .map(PrivatePkcs8KeyDer::from)
            .map(Into::into)
            .expect("Could not form private keys passed in");
        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .expect("Bad certificates");

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
                            Some(Err(e))
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
                        _ => {
                            continue;
                        }
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

                        if upgrade {
                            let _res = timeout(
                                Duration::from_millis(connection_timeout),
                                &mut http_future.with_upgrades(),
                            )
                            .await;
                        } else {
                            let _res = timeout(
                                Duration::from_millis(connection_timeout),
                                &mut http_future,
                            )
                            .await;
                        }
                    });
                }
            }
        });

        ReusableBoxFuture::new(listener_fut)
    }
}

impl<T: Context<Response = Response<Body>> + Clone + Send + Sync, S: 'static + Send + Sync>
    SSLHyperServer<T, S>
{
    pub fn with_upgrades(mut self, upgrade: bool) -> Self {
        self.upgrade = upgrade;

        self
    }
}
